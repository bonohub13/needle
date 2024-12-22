use crate::{
    NeedleConfig, NeedleErr, NeedleError, NeedleLabel, ShaderRenderer, TextRenderer, Texture, Time,
};
use anyhow::{Context, Result};
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use winit::{dpi::PhysicalSize, window::Window};

pub struct State<'a> {
    window: &'a Window,
    app_config: NeedleConfig,
    size: PhysicalSize<u32>,
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    depth_texture: Texture,
    text_renderer: TextRenderer,
    fps_renderer: TextRenderer,
    background_renderer: ShaderRenderer,
}

impl<'a> State<'a> {
    pub async fn new(window: &'a Window, config: &NeedleConfig) -> Result<Self> {
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
                    required_features: wgpu::Features::SPIRV_SHADER_PASSTHROUGH,
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
        let surface_config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let depth_texture =
            Texture::create_depth_texture(&device, &surface_config, NeedleLabel::Texture("Depth"));
        let depth_stencil = wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        };

        // Text Rendering System
        let text_renderer = TextRenderer::new(
            &config.time.config,
            &size,
            scale_factor,
            &device,
            &queue,
            surface_format,
            Some(depth_stencil.clone()),
        );

        // Fps Rendering System
        let fps_renderer = TextRenderer::new(
            &config.fps.config,
            &size,
            scale_factor,
            &device,
            &queue,
            surface_format,
            Some(depth_stencil.clone()),
        );

        let background_renderer = ShaderRenderer::new(
            &device,
            &surface_config,
            "shaders/spv/shader.vert.spv",
            "shaders/spv/shader.frag.spv",
            vec![],
            vec![],
            vec![],
            Some(depth_stencil),
            Some("Backgroun Render"),
        )?;

        Ok(Self {
            window,
            app_config: *config,
            size,
            surface,
            device,
            queue,
            config: surface_config,
            depth_texture,
            text_renderer,
            fps_renderer,
            background_renderer,
        })
    }

    pub const fn window(&self) -> &Window {
        &self.window
    }

    pub const fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    pub const fn config(&self) -> &NeedleConfig {
        &self.app_config
    }

    pub fn resize(&mut self, size: &PhysicalSize<u32>) {
        if (size.width > 0) && (size.height > 0) {
            self.size = *size;
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = Texture::create_depth_texture(
                &self.device,
                &self.config,
                NeedleLabel::Texture("Depth"),
            );
            self.text_renderer.resize(size);
            self.fps_renderer.resize(size);
        }
    }

    pub fn update(&mut self, fps: f32) -> Result<()> {
        let local = chrono::Local::now();
        let time_formatter = Time::new(self.app_config.time.format);

        self.text_renderer
            .set_text(&time_formatter.time_to_str(&local));
        self.fps_renderer.set_text(&format!("FPS: {:.3}", fps));
        self.text_renderer.update(&self.queue, &self.config);
        self.text_renderer.prepare(5.0, &self.device, &self.queue)?;
        if self.app_config.fps.enable {
            self.fps_renderer.update(&self.queue, &self.config);
            self.fps_renderer.prepare(0.0, &self.device, &self.queue)?;
        }

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
                label: Some(&NeedleLabel::CommandEncoder("").to_string()),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&NeedleLabel::RenderPass("").to_string()),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.app_config.background_color[0],
                            g: self.app_config.background_color[1],
                            b: self.app_config.background_color[2],
                            a: self.app_config.background_color[3],
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: self.depth_texture.view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            self.background_renderer.render(&mut render_pass);
            self.text_renderer.render(&mut render_pass)?;
            if self.app_config.fps.enable {
                self.fps_renderer.render(&mut render_pass)?;
            }
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        self.text_renderer.trim();
        self.fps_renderer.trim();

        Ok(())
    }
}
