mod config;

pub use config::*;

use crate::{renderer::TextRenderer, NeedleErr, NeedleError, NeedleLabel, Time};
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
    text_renderer: TextRenderer,
}

impl<'a> State<'a> {
    #[allow(dead_code)]
    const VERTEX_SHADER_MAIN: &'static str = "vs_main";

    #[allow(dead_code)]
    const FRAGMENT_SHADER_MAIN: &'static str = "fs_main";

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

        // Text Rendering System
        let text_renderer = TextRenderer::new(
            &config.text,
            &size,
            scale_factor,
            &device,
            &queue,
            surface_format,
        );

        Ok(Self {
            window,
            app_config: *config,
            size,
            surface,
            device,
            queue,
            config: surface_config,
            text_renderer,
        })
    }

    pub const fn window(&self) -> &Window {
        &self.window
    }

    pub const fn size(&self) -> PhysicalSize<u32> {
        self.size
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
        let time_formatter = Time::new(*self.text_renderer.format());

        self.text_renderer
            .set_text(&time_formatter.time_to_str(&local));
        self.text_renderer.update(&self.queue, &self.config);

        let (text_width, text_height) = self.text_renderer.text_size();
        let center = [
            (self.size.width as f32 - text_width * self.text_renderer.scale()) / 2.0,
            (self.size.height as f32 - text_height * self.text_renderer.scale()) / 2.0,
        ];
        self.text_renderer
            .prepare(&self.device, &self.queue, &self.size, center[0], center[1])?;

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
                            r: self.app_config.background_color[0],
                            g: self.app_config.background_color[1],
                            b: self.app_config.background_color[2],
                            a: self.app_config.background_color[3],
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            self.text_renderer.render(&mut render_pass)?;
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
