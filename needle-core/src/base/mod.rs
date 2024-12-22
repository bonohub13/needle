use crate::{NeedleConfig, NeedleErr, NeedleError, NeedleLabel};
use anyhow::{Context, Result};
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use winit::{dpi::PhysicalSize, window::Window};

pub struct AppBase<'a> {
    window: &'a Window,
    app_config: NeedleConfig,
    size: PhysicalSize<u32>,
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
}

impl<'a> AppBase<'a> {
    pub async fn new(window: &'a Window, config: &NeedleConfig) -> Result<Self> {
        let size = window.inner_size();
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

        Ok(Self {
            window,
            app_config: *config,
            size,
            surface,
            device,
            queue,
            config: surface_config,
        })
    }

    #[inline]
    pub const fn window(&self) -> &Window {
        &self.window
    }

    #[inline]
    pub const fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    #[inline]
    pub const fn config(&self) -> &NeedleConfig {
        &self.app_config
    }

    #[inline]
    pub const fn device(&self) -> &Device {
        &self.device
    }

    #[inline]
    pub const fn queue(&self) -> &Queue {
        &self.queue
    }

    #[inline]
    pub const fn surface_config(&self) -> &SurfaceConfiguration {
        &self.config
    }

    pub fn resize(&mut self, size: &PhysicalSize<u32>) {
        if (size.width > 0) && (size.height > 0) {
            self.size = *size;
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render<F: FnOnce(&wgpu::SurfaceTexture, &mut wgpu::CommandEncoder) -> NeedleErr<()>>(
        &mut self,
        render_func: F,
    ) -> NeedleErr<()> {
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
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some(&NeedleLabel::CommandEncoder("").to_string()),
            });

        render_func(&output, &mut encoder)?;

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}