use anyhow::Result;
use needle_core::{
    AppBase, ImguiState, NeedleConfig, NeedleErr, NeedleError, NeedleLabel, ShaderRenderer,
    TextRenderer, Texture, Time, Vertex,
};
use std::{
    fs::{self, OpenOptions},
    io::copy,
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, ElementState, Event, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

pub struct Needle<'a> {
    window: Option<Arc<Window>>,
    imgui: Option<ImguiState>,
    app_base: Option<AppBase<'a>>,
    config: Option<Arc<NeedleConfig>>,
    depth_texture: Option<Texture>,
    background_renderer: Option<ShaderRenderer>,
    time_renderer: Option<TextRenderer>,
    fps_renderer: Option<TextRenderer>,
    current_frame: u64,
    next_frame: Instant,
    fps_update: Instant,
    fps_limit: Duration,
    fps_update_limit: Duration,
}

impl Needle<'_> {
    const APP_NAME: &'static str = env!("CARGO_PKG_NAME");
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    const VERTEX_SHADER_DEFAULT_PATH: &'static str = "shaders/spv/shader.vert.spv";
    const FRAGMENT_SHADER_DEFAULT_PATH: &'static str = "shaders/spv/shader.frag.spv";
    const CONFIG_WINDOW_SUFFIX: &'static str = "config";

    pub fn set_config(&mut self, config: Arc<NeedleConfig>) -> Result<()> {
        let shader_path = NeedleConfig::config_path(false, Some("shaders/spv"))?;
        let vert_shader_path =
            NeedleConfig::config_path(false, Some(Self::VERTEX_SHADER_DEFAULT_PATH))?;
        let frag_shader_path =
            NeedleConfig::config_path(false, Some(Self::FRAGMENT_SHADER_DEFAULT_PATH))?;

        self.config = Some(config.clone());
        self.fps_limit = Duration::from_secs_f64(1.0 / config.fps.frame_limit as f64);
        self.fps_update_limit = Duration::from_secs_f64(1.0);

        if !(vert_shader_path.exists() && frag_shader_path.exists()) {
            if !shader_path.exists() {
                fs::create_dir_all(shader_path)?;
            }

            Self::download_shader()?;
        }

        Ok(())
    }

    pub fn download_shader() -> Result<()> {
        let vert_shader = "shader.vert.spv";
        let frag_shader = "shader.frag.spv";

        Self::write(vert_shader)?;
        Self::write(frag_shader)?;

        Ok(())
    }

    fn create_renderers(&mut self) -> Result<()> {
        if let Some(config) = self.config.as_ref() {
            let app_base = self
                .app_base
                .as_ref()
                .expect("needle_core::AppBase for Needle not available");
            let window = self.window.as_ref().expect("Window not initialized");
            let window_size = window.inner_size();
            let window_scale_factor = window.scale_factor();
            let depth_stencil_state = Texture::default_depth_stencil();
            let (background_vertices, indices) =
                Vertex::indexed_rectangle([1.0, 1.0], [0.0, 0.0], 0.1, &config.background_color);
            let background_vertex_buffer =
                app_base.create_vertex_buffer("Background", &background_vertices);
            let background_index_buffer = app_base.create_index_buffer("Background", &indices);
            let background_renderer = ShaderRenderer::new(
                app_base.device(),
                app_base.surface_config(),
                NeedleConfig::config_path(false, Some(Self::VERTEX_SHADER_DEFAULT_PATH))
                    .expect("Failed to find vertex shader"),
                NeedleConfig::config_path(false, Some(Self::FRAGMENT_SHADER_DEFAULT_PATH))
                    .expect("Failed to find fragment shader"),
                vec![background_vertex_buffer],
                vec![Vertex::buffer_layout()],
                Some((0, indices)),
                Some(background_index_buffer),
                Some(depth_stencil_state.clone()),
                Some("Background"),
            )?;
            let time_renderer = TextRenderer::new(
                &config.time.config,
                config.time.font.clone(),
                &window_size,
                window_scale_factor,
                app_base.device(),
                app_base.queue(),
                app_base.surface_config().format,
                Some(depth_stencil_state.clone()),
            )?;
            let fps_renderer = TextRenderer::new(
                &config.fps.config,
                None,
                &window_size,
                window_scale_factor,
                app_base.device(),
                app_base.queue(),
                app_base.surface_config().format,
                Some(depth_stencil_state.clone()),
            )?;

            self.background_renderer = Some(background_renderer);
            self.time_renderer = Some(time_renderer);
            self.fps_renderer = Some(fps_renderer);

            Ok(())
        } else {
            Err(NeedleError::ConfigNonExistant("Config not available".into()).into())
        }
    }

    fn resize(&mut self, size: &winit::dpi::PhysicalSize<u32>) {
        if (size.width > 0) && (size.height > 0) {
            if let (Some(window), Some(imgui), Some(app_base), Some(time), Some(fps)) = (
                self.window.as_ref(),
                self.imgui.as_mut(),
                self.app_base.as_mut(),
                self.time_renderer.as_mut(),
                self.fps_renderer.as_mut(),
            ) {
                imgui.resize(window.scale_factor());
                app_base.resize(size);
                self.depth_texture = Some(Texture::create_depth_texture(
                    app_base.device(),
                    app_base.surface_config(),
                    NeedleLabel::Texture("Depth"),
                ));
                time.resize(size);
                fps.resize(size);
            }
        }
    }

    fn update(&mut self) -> Result<Instant> {
        let app_base = self
            .app_base
            .as_ref()
            .expect("needle_core::AppBase not available");
        let imgui = self.imgui.as_mut().expect("ImguiState not available");
        let config = self.config.as_ref().expect("NeedleConfig not available");
        let time = self
            .time_renderer
            .as_mut()
            .expect("Time Renderer not available");
        let fps = self
            .fps_renderer
            .as_mut()
            .expect("FPS Renderer not available");
        let now = Instant::now();
        let local = chrono::Local::now();
        let time_formatter = Time::new(config.time.format);

        imgui.update(now);
        time.set_text(&time_formatter.time_to_str(&local));
        time.update(app_base.queue(), app_base.surface_config());
        time.prepare(5.0, app_base.device(), app_base.queue())?;

        if config.fps.enable {
            fps.set_text(&format!(
                "{:.3}",
                config.fps.frame_limit as f64 - 1.0 / self.current_frame as f64
            ));
            fps.update(app_base.queue(), app_base.surface_config());
            fps.prepare(5.0, app_base.device(), app_base.queue())?;
        }

        Ok(now)
    }

    fn render(&mut self) -> NeedleErr<()> {
        let window = self.window.as_ref().expect("Windows not available");
        let imgui = self.imgui.as_mut().expect("ImguiState not available");
        let app_base = self
            .app_base
            .as_mut()
            .expect("needle_core::AppBase not available");
        let depth_texture = self
            .depth_texture
            .as_ref()
            .expect("Depth Texture not available");
        let background_renderer = self
            .background_renderer
            .as_mut()
            .expect("Background Renderer not available");
        let time_renderer = self
            .time_renderer
            .as_mut()
            .expect("Time Renderer not available");
        let fps_renderer = self
            .fps_renderer
            .as_mut()
            .expect("FPS Renderer not available");
        let color = wgpu::Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        };

        imgui.setup_ui(window, |ui| {
            let window_name = format!("{}: {}", Self::APP_NAME, Self::CONFIG_WINDOW_SUFFIX);
            let window = ui.window(&window_name);

            window
                .size([300.0, 100.0], imgui::Condition::FirstUseEver)
                .build(|| {
                    ui.text("configuration");
                    ui.separator();
                });
        })?;
        app_base.render(|current_texture, encoder| {
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
                    view: depth_texture.view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            background_renderer.render(&mut render_pass);
            time_renderer.render(&mut render_pass)?;
            fps_renderer.render(&mut render_pass)?;

            Ok(())
        })
    }

    fn handle_events(&mut self, event: &Event<()>) {
        if let (Some(window), Some(imgui)) = (self.window.as_ref(), self.imgui.as_mut()) {
            imgui.handle_event(window, event)
        }
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

impl ApplicationHandler for Needle<'_> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let window_attr = Window::default_attributes()
                .with_title(Self::APP_NAME)
                .with_resizable(true)
                .with_transparent(true);
            let window = Arc::new(
                event_loop
                    .create_window(window_attr)
                    .expect("Failed to create window."),
            );
            let imgui = ImguiState::new(window.scale_factor(), &window);
            let app_base = pollster::block_on(AppBase::new(window.clone()))
                .expect("Failed to create needle_core::AppBase");
            let depth_texture = Texture::create_depth_texture(
                app_base.device(),
                app_base.surface_config(),
                NeedleLabel::Texture("Depth"),
            );

            self.window = Some(window.clone());
            self.imgui = Some(imgui);
            self.app_base = Some(app_base);
            self.depth_texture = Some(depth_texture);
            if let Err(e) = self.create_renderers() {
                panic!("{}", e);
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        self.current_frame += 1;
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
            WindowEvent::Resized(physical_size) => {
                self.resize(&physical_size);
            }
            WindowEvent::RedrawRequested => {
                if self.window.is_some()
                    && self.imgui.is_some()
                    && self.app_base.is_some()
                    && self.config.is_some()
                    && self.depth_texture.is_some()
                    && self.background_renderer.is_some()
                    && self.time_renderer.is_some()
                    && self.fps_renderer.is_some()
                {
                    /* Check for window has been done in the if statement above */
                    self.window.as_ref().unwrap().request_redraw();
                    let now = match self.update() {
                        Err(e) => {
                            log::error!("{}", e);
                            event_loop.exit();
                            None
                        }
                        Ok(now) => Some(now),
                    }
                    .unwrap();
                    match self.render() {
                        Ok(_) => {
                            self.next_frame += self.fps_limit;

                            if (self.fps_update - now) > self.fps_update_limit {
                                self.fps_update = now;
                                self.current_frame = 0;
                            }
                            std::thread::sleep(self.next_frame - std::time::Instant::now());
                        }
                        Err(e) => match e {
                            NeedleError::Lost | NeedleError::Outdated => {
                                if let Some(window) = self.window.as_ref() {
                                    let size = window.inner_size();

                                    self.resize(&size);
                                }
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
            }
            _ => (),
        }
        self.handle_events(&winit::event::Event::WindowEvent { window_id, event });
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: ()) {
        self.handle_events(&Event::UserEvent(event))
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        self.handle_events(&Event::DeviceEvent { device_id, event })
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.handle_events(&Event::AboutToWait)
    }
}

impl Default for Needle<'_> {
    fn default() -> Self {
        Self {
            window: None,
            imgui: None,
            app_base: None,
            config: None,
            depth_texture: None,
            background_renderer: None,
            time_renderer: None,
            fps_renderer: None,
            next_frame: Instant::now(),
            fps_update: Instant::now(),
            current_frame: 0,
            fps_limit: Duration::from_secs_f64(1.0),
            fps_update_limit: Duration::default(),
        }
    }
}
