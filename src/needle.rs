// Copyright 2025 Kensuke Saito
// SPDX-License-Identifier: MIT

use anyhow::Result;
use needle_core::{
    NeedleConfig, NeedleErr, NeedleError, NeedleLabel, OpMode, Renderer, ShaderRenderer,
    ShaderRendererDescriptor, State, TextRenderer, Texture, Time, Vertex,
};
use std::{
    fs::{self, OpenOptions},
    io::copy,
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

pub struct NeedleBase<'a> {
    window: Arc<Window>,
    state: State<'a>,
    depth_texture: Texture,
    background_renderer: ShaderRenderer,
    time_renderer: TextRenderer,
    fps_renderer: TextRenderer,
    clock_info: Time,
    current_frame: u64,
    next_frame: Instant,
    fps_update: Instant,
    fps_limit: Duration,
    fps_update_limit: Duration,
}

pub struct Needle<'window> {
    base: Option<NeedleBase<'window>>,
    config: Option<Arc<NeedleConfig>>,
}

impl<'a> NeedleBase<'a> {
    fn new(event_loop: &ActiveEventLoop, config: Arc<NeedleConfig>) -> Result<Self> {
        let window = {
            let attr = Window::default_attributes()
                .with_title(Needle::APP_NAME)
                .with_resizable(true)
                .with_transparent(true);
            let window = event_loop.create_window(attr)?;

            Arc::new(window)
        };
        let state = pollster::block_on(State::new(window.clone()))?;
        let depth_texture = Texture::create_depth_texture(
            state.device(),
            state.surface_config(),
            NeedleLabel::Texture("Depth"),
        );
        let (background, time, fps) =
            Self::create_renderers(window.clone(), config.clone(), &state)?;

        Ok(Self {
            window,
            state,
            depth_texture,
            background_renderer: background,
            time_renderer: time,
            fps_renderer: fps,
            clock_info: Time::new(config.time.format),
            current_frame: 0,
            next_frame: Instant::now(),
            fps_limit: Duration::from_secs_f64(1.0 / config.fps.frame_limit as f64),
            fps_update_limit: Duration::from_secs_f64(1.0),
            fps_update: Instant::now(),
        })
    }

    fn set_mode(&mut self, mode: OpMode) {
        self.clock_info.set_mode(mode);
    }

    fn start_clock(&mut self) -> NeedleErr<()> {
        match self.clock_info.mode() {
            OpMode::Clock => Err(NeedleError::TimerStartFailure),
            OpMode::CountDownTimer(_) | OpMode::CountUpTimer => {
                self.clock_info.toggle_timer();
                Ok(())
            }
        }
    }

    fn resize(&mut self, size: &winit::dpi::PhysicalSize<u32>) {
        if (size.width > 0) && (size.height > 0) {
            self.state.resize(size);
            self.depth_texture = Texture::create_depth_texture(
                self.state.device(),
                self.state.surface_config(),
                NeedleLabel::Texture("Depth"),
            );
            self.time_renderer.resize(size);
            self.fps_renderer.resize(size);
        }
    }

    fn update(&mut self, config: Arc<NeedleConfig>) -> Result<()> {
        self.time_renderer.set_text(&self.clock_info.current_time());
        self.time_renderer
            .update(self.state.queue(), self.state.surface_config());
        self.time_renderer
            .prepare(5.0, self.state.device(), self.state.queue())?;

        if config.fps.enable {
            self.fps_renderer.set_text(&format!(
                "{:.3}",
                config.fps.frame_limit as f64 - 1.0 / self.current_frame as f64
            ));
            self.fps_renderer
                .update(self.state.queue(), self.state.surface_config());
            self.fps_renderer
                .prepare(5.0, self.state.device(), self.state.queue())?;
        }

        Ok(())
    }

    fn render(&mut self) -> NeedleErr<()> {
        let color = wgpu::Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        };

        self.state.render(|current_texture, encoder| {
            let view = current_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&NeedleLabel::RenderPass("").to_string()),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(color),
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

            self.background_renderer.render(&mut render_pass)?;
            self.time_renderer.render(&mut render_pass)?;
            self.fps_renderer.render(&mut render_pass)?;

            Ok(())
        })
    }

    fn create_renderers(
        window: Arc<Window>,
        config: Arc<NeedleConfig>,
        state: &State,
    ) -> Result<(ShaderRenderer, TextRenderer, TextRenderer)> {
        let window_size = window.inner_size();
        let window_scale_factor = window.scale_factor();
        let depth_stencil_state = Texture::default_depth_stencil();
        let (background_vertices, indices) =
            Vertex::indexed_rectangle([1.0, 1.0], [0.0, 0.0], 0.1, &config.background_color);
        let background_vertex_buffer =
            state.create_vertex_buffer("Background", &background_vertices);
        let background_index_buffer = state.create_index_buffer("Background", &indices);
        let background_renderer = {
            let desc = ShaderRendererDescriptor {
                vert_shader_path: NeedleConfig::config_path(
                    false,
                    Some(Needle::VERTEX_SHADER_DEFAULT_PATH),
                )?,
                frag_shader_path: NeedleConfig::config_path(
                    false,
                    Some(Needle::FRAGMENT_SHADER_DEFAULT_PATH),
                )?,
                vertex_buffers: &[background_vertex_buffer],
                vertex_buffer_layouts: &[Vertex::buffer_layout()],
                indices: Some((0, indices)),
                index_buffers: Some(background_index_buffer),
                depth_stencil: Some(depth_stencil_state.clone()),
                label: Some("Background"),
            };

            ShaderRenderer::new(state, &desc)?
        };
        let time_renderer = TextRenderer::new(
            state,
            &config.time.config,
            config.time.font.clone(),
            &window_size,
            window_scale_factor,
            state.surface_config().format,
            Some(depth_stencil_state.clone()),
        )?;
        let fps_renderer = TextRenderer::new(
            state,
            &config.fps.config,
            None,
            &window_size,
            window_scale_factor,
            state.surface_config().format,
            Some(depth_stencil_state.clone()),
        )?;

        Ok((background_renderer, time_renderer, fps_renderer))
    }
}

impl Needle<'_> {
    const APP_NAME: &'static str = env!("CARGO_PKG_NAME");
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    const VERTEX_SHADER_DEFAULT_PATH: &'static str = "shaders/spv/shader.vert.spv";
    const FRAGMENT_SHADER_DEFAULT_PATH: &'static str = "shaders/spv/shader.frag.spv";

    pub fn set_config(&mut self, config: Arc<NeedleConfig>) -> Result<()> {
        let shader_path = NeedleConfig::config_path(false, Some("shaders/spv"))?;
        let vert_shader_path =
            NeedleConfig::config_path(false, Some(Self::VERTEX_SHADER_DEFAULT_PATH))?;
        let frag_shader_path =
            NeedleConfig::config_path(false, Some(Self::FRAGMENT_SHADER_DEFAULT_PATH))?;

        if !(vert_shader_path.exists() && frag_shader_path.exists()) {
            if !shader_path.exists() {
                fs::create_dir_all(shader_path)?;
            }

            Self::download_shader()?;
        }

        self.config = Some(config);

        Ok(())
    }

    fn download_shader() -> Result<()> {
        let vert_shader = "shader.vert.spv";
        let frag_shader = "shader.frag.spv";

        Self::write(vert_shader)?;
        Self::write(frag_shader)?;

        Ok(())
    }

    fn write(path: &str) -> Result<()> {
        let release_url_base = "https://github.com/bonohub13/needle/releases/download";
        let write_path =
            match NeedleConfig::config_path(false, Some(&format!("shaders/spv/{}", path))) {
                Ok(p) => Ok(p),
                Err(_) => Err(NeedleError::InvalidPath),
            }?;
        let src_url = format!("{}/{}/{}", release_url_base, Self::VERSION, path);
        let mut file = match OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(write_path)
        {
            Ok(file) => Ok(file),
            Err(_) => Err(NeedleError::InvalidPath),
        }?;
        let resp = reqwest::blocking::get(&src_url)?;
        let content = resp.bytes()?;

        log::debug!("URL : {}", src_url);
        copy(&mut content.as_ref(), &mut file)?;

        Ok(())
    }
}

impl<'a> ApplicationHandler for Needle<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.base.is_none() {
            if let Some(config) = self.config.as_ref() {
                match NeedleBase::new(event_loop, config.clone()) {
                    Ok(base) => self.base = Some(base),
                    Err(e) => panic!("{}", e),
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let (Some(base), Some(config)) = (self.base.as_mut(), self.config.as_mut()) {
            base.current_frame += 1;
            match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            ..
                        },
                    ..
                } => {
                    event_loop.exit();
                }
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::Space),
                            ..
                        },
                    ..
                } => {
                    if let Err(e) = base.start_clock() {
                        log::error!("{}", e);
                        event_loop.exit();
                    }
                }
                WindowEvent::Resized(physical_size) => {
                    base.resize(&physical_size);
                }
                WindowEvent::RedrawRequested => {
                    /* Check for window has been done in the if statement above */
                    base.window.request_redraw();
                    if let Err(e) = base.update(config.clone()) {
                        log::error!("{}", e);
                        event_loop.exit();
                    }
                    match base.render() {
                        Ok(_) => {
                            base.next_frame += base.fps_limit;

                            if (base.fps_update - std::time::Instant::now()) > base.fps_update_limit
                            {
                                base.fps_update = std::time::Instant::now();
                                base.current_frame = 0;
                            }
                            std::thread::sleep(base.next_frame - std::time::Instant::now());
                        }
                        Err(e) => match e {
                            NeedleError::Lost | NeedleError::Outdated => {
                                let size = base.window.inner_size();

                                base.resize(&size);
                            }
                            NeedleError::OutOfMemory | NeedleError::RemovedFromAtlas => {
                                log::error!("{}", NeedleError::OutOfMemory);
                                event_loop.exit();
                            }
                            NeedleError::Timeout => {
                                log::warn!("{}", NeedleError::Timeout);
                            }
                            NeedleError::Other => {
                                log::error!("{}", NeedleError::Other);
                            }
                            _ => (),
                        },
                    }
                }
                _ => (),
            }
        }
    }
}

impl Default for Needle<'_> {
    fn default() -> Self {
        Self {
            base: None,
            config: None,
        }
    }
}
