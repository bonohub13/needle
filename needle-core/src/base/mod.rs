use crate::{NeedleErr, NeedleError, NeedleLabel, Vertex};
use anyhow::{Context, Result};
use std::{sync::Arc, time::Instant};
use wgpu::{util::DeviceExt, Device, Queue, Surface, SurfaceConfiguration};
use winit::{dpi::PhysicalSize, window::Window};

pub struct ImguiState {
    pub context: imgui::Context,
    pub platform: imgui_winit_support::WinitPlatform,
    hidpi_factor: f64,
    last_frame: Instant,
}

pub struct AppBase<'a> {
    size: PhysicalSize<u32>,
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
}

impl ImguiState {
    pub fn new(hidpi_factor: f64, window: &winit::window::Window) -> Self {
        let mut context = imgui::Context::create();
        let mut platform = imgui_winit_support::WinitPlatform::new(&mut context);
        let font_size = (13.0 * hidpi_factor) as f32;

        platform.attach_window(
            context.io_mut(),
            window,
            imgui_winit_support::HiDpiMode::Default,
        );
        context.set_ini_filename(None);
        context.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        context
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData {
                config: Some(imgui::FontConfig {
                    oversample_h: 1,
                    pixel_snap_h: true,
                    size_pixels: font_size,
                    ..Default::default()
                }),
            }]);

        Self {
            context,
            platform,
            hidpi_factor,
            last_frame: Instant::now(),
        }
    }

    pub fn resize(&mut self, hidpi_factor: f64) {
        self.hidpi_factor = hidpi_factor;
    }

    pub fn update(&mut self, now: Instant) {
        self.context
            .io_mut()
            .update_delta_time(now - self.last_frame);
        self.last_frame = now;
    }

    pub fn handle_event(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::Event<()>,
    ) {
        self.platform
            .handle_event::<()>(self.context.io_mut(), window, event)
    }

    pub fn setup_ui<F: FnOnce(&mut imgui::Ui)>(
        &mut self,
        window: &winit::window::Window,
        func: F,
    ) -> NeedleErr<()> {
        match self.platform.prepare_frame(self.context.io_mut(), window) {
            Ok(_) => Ok(()),
            Err(err) => {
                println!("{} ({})", NeedleError::FailedToPrepareFrame, err);

                Err(NeedleError::FailedToPrepareFrame)
            }
        }?;

        func(self.context.frame());

        Ok(())
    }
}

impl AppBase<'_> {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        // Surface
        let surface = instance.create_surface(window)?;

        // Device and Queue
        let adapters = instance.enumerate_adapters(wgpu::Backends::all());
        let adapter = adapters
            .iter()
            .find(|adapter| adapter.is_surface_supported(&surface))
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
        let surface_caps = surface.get_capabilities(adapter);
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
            size,
            surface,
            device,
            queue,
            config: surface_config,
        })
    }

    #[inline]
    pub const fn size(&self) -> PhysicalSize<u32> {
        self.size
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
                    wgpu::SurfaceError::Other => NeedleError::Other,
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

    pub fn create_vertex_buffer(&self, label: &str, vertices: &[Vertex]) -> wgpu::Buffer {
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&NeedleLabel::VertexBuffer(label).to_string()),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            })
    }

    pub fn create_index_buffer(&self, label: &str, indices: &[u16]) -> wgpu::Buffer {
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&NeedleLabel::IndexBuffer(label).to_string()),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            })
    }
}
