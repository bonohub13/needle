use crate::{NeedleErr, NeedleError, NeedleLabel, Time, TimeFormat};
use anyhow::{Context, Result};
use glyphon::{Buffer, FontSystem, SwashCache, TextAtlas, TextRenderer, Viewport};
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use winit::{dpi::PhysicalSize, window::Window};

pub struct State<'a> {
    window: &'a Window,
    time_formatter: Time,
    size: PhysicalSize<u32>,
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,

    font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: Viewport,
    atlas: TextAtlas,
    text_renderer: glyphon::TextRenderer,
    text_buffer: glyphon::Buffer,
    text_scale: f32,
}

impl<'a> State<'a> {
    #[allow(dead_code)]
    const VERTEX_SHADER_MAIN: &'static str = "vs_main";

    #[allow(dead_code)]
    const FRAGMENT_SHADER_MAIN: &'static str = "fs_main";

    pub async fn new(window: &'a Window, time_format: TimeFormat, text_scale: f32) -> Result<Self> {
        let time_formatter = Time::new(time_format);
        let size = window.inner_size();
        let scale_factor = window.scale_factor();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        // Surface
        let surface = instance.create_surface(window)?;

        // Device and Queue
        let adapters = instance.enumerate_adapters(wgpu::Backends::all());
        let adapter = adapters
            .iter()
            .filter(|adapter| adapter.is_surface_supported(&surface))
            .next()
            .context("Failed to find valid adapter")?;
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    ..Default::default()
                },
                None,
            )
            .await?;

        // Config
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        // Text Rendering System
        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = glyphon::Cache::new(&device);
        let viewport = Viewport::new(&device, &cache);
        let mut atlas = TextAtlas::new(&device, &queue, &cache, surface_format);
        let text_renderer =
            TextRenderer::new(&mut atlas, &device, wgpu::MultisampleState::default(), None);
        let mut text_buffer = Buffer::new(&mut font_system, glyphon::Metrics::new(30.0, 42.0));
        let physical_width = (size.width as f64 * scale_factor) as f32;
        let physical_height = (size.height as f64 * scale_factor) as f32;

        text_buffer.set_size(
            &mut font_system,
            Some(physical_width),
            Some(physical_height),
        );
        text_buffer.shape_until_scroll(&mut font_system, false);

        Ok(Self {
            window,
            time_formatter,
            size,
            surface,
            device,
            queue,
            config,

            font_system,
            swash_cache,
            viewport,
            text_buffer,
            atlas,
            text_renderer,
            text_scale,
        })
    }

    pub const fn window(&self) -> &Window {
        &self.window
    }

    pub const fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    pub fn text_size(&self) -> (f32, f32) {
        let (width, total_lines) = self
            .text_buffer
            .layout_runs()
            .fold((0.0, 0usize), |(width, total_lines), run| {
                (run.line_w.max(width), total_lines + 1)
            });

        (
            width,
            total_lines as f32 * self.text_buffer.metrics().line_height,
        )
    }

    pub fn resize(&mut self, size: &PhysicalSize<u32>) {
        if (size.width > 0) && (size.height > 0) {
            self.size = *size;
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn update(&mut self) -> Result<()> {
        let local = chrono::Local::now();

        self.text_buffer.set_text(
            &mut self.font_system,
            &self.time_formatter.time_to_str(&local),
            glyphon::Attrs::new().family(glyphon::Family::Serif),
            glyphon::Shaping::Advanced,
        );
        self.viewport.update(
            &self.queue,
            glyphon::Resolution {
                width: self.config.width,
                height: self.config.height,
            },
        );

        let (text_width, text_height) = self.text_size();
        let center = [
            (self.size.width as f32 - text_width * self.text_scale) / 2.0,
            (self.size.height as f32 - text_height * self.text_scale) / 2.0,
        ];
        self.text_renderer.prepare(
            &self.device,
            &self.queue,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            [glyphon::TextArea {
                buffer: &self.text_buffer,
                left: center[0],
                top: center[1],
                scale: self.text_scale,
                bounds: glyphon::TextBounds {
                    left: 0,
                    top: 0,
                    right: self.size.width as i32,
                    bottom: self.size.height as i32,
                },
                default_color: glyphon::Color::rgb(255, 255, 255),
                custom_glyphs: &[],
            }],
            &mut self.swash_cache,
        )?;

        Ok(())
    }

    pub fn render(&mut self) -> NeedleErr<()> {
        let output = match self.surface.get_current_texture() {
            Ok(texture) => Ok(texture),
            Err(err) => {
                let err = match err {
                    wgpu::SurfaceError::Timeout => NeedleError::Timeout,
                    wgpu::SurfaceError::Outdated => NeedleError::Outdated,
                    wgpu::SurfaceError::Lost => NeedleError::Lost,
                    wgpu::SurfaceError::OutOfMemory => NeedleError::OutOfMemory,
                };

                Err(err)
            }
        }?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some(NeedleLabel::CommandEncoder("").as_str()),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(NeedleLabel::RenderPass("").as_str()),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            match self
                .text_renderer
                .render(&self.atlas, &self.viewport, &mut render_pass)
            {
                Ok(_) => (),
                Err(err) => {
                    return match err {
                        glyphon::RenderError::RemovedFromAtlas => {
                            Err(NeedleError::RemovedFromAtlas)
                        }
                        glyphon::RenderError::ScreenResolutionChanged => {
                            Err(NeedleError::ScreenResolutionChanged)
                        }
                    }
                }
            }
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
