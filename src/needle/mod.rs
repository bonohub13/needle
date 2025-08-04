// Copyright 2025 Kensuke Saito
// SPDX-License-Identifier: MIT

use anyhow::Result;
use imgui::Condition;
use needle_core::{
    FontTypes, ImguiMode, ImguiState, NeedleConfig, NeedleErr, NeedleError, NeedleLabel, OpMode,
    Position, Renderer, ShaderRenderer, ShaderRendererDescriptor, State, TextRenderer, Texture,
    Time, Vertex,
};
use std::{
    cell::RefCell,
    fs::{self, OpenOptions},
    io::copy,
    rc::Rc,
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
    imgui_state: ImguiState,
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

#[derive(Default)]
pub struct Needle<'window> {
    base: Option<NeedleBase<'window>>,
    config: Option<Rc<RefCell<NeedleConfig>>>,
}

impl<'a> NeedleBase<'a> {
    const NEEDLE_IMGUI_WINDOW_TITLE: &'static str = "Needle Settings";
    const NEEDLE_IMGUI_WINDOW_SIZE: [f32; 2] = [800.0, 600.0];

    fn new(event_loop: &ActiveEventLoop, config: Rc<RefCell<NeedleConfig>>) -> Result<Self> {
        let window = {
            let attr = Window::default_attributes()
                .with_title(Needle::APP_NAME)
                .with_resizable(true)
                .with_transparent(true);
            let window = event_loop.create_window(attr)?;

            Arc::new(window)
        };
        let state = pollster::block_on(State::new(window.clone()))?;
        let imgui_state = ImguiState::new(window.clone(), config.clone(), &state);
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
            imgui_state,
            depth_texture,
            background_renderer: background,
            time_renderer: time,
            fps_renderer: fps,
            clock_info: Time::new(config.borrow().time.format),
            current_frame: 0,
            next_frame: Instant::now(),
            fps_limit: Duration::from_secs_f64(1.0 / config.borrow().fps.frame_limit as f64),
            fps_update_limit: Duration::from_secs_f64(1.0),
            fps_update: Instant::now(),
        })
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

    fn update(&mut self, config: &NeedleConfig) -> NeedleErr<()> {
        let (background_vertices, _) =
            Vertex::indexed_rectangle([1.0, 1.0], [0.0, 0.0], 0.1, &config.background_color);
        let background_vertex_buffer = self
            .state
            .create_vertex_buffer("Background", &background_vertices);

        self.background_renderer
            .set_vertex_buffer(&[background_vertex_buffer])?;
        self.time_renderer.set_text(&self.clock_info.current_time());
        self.time_renderer.set_config(&config.time.config);
        self.time_renderer
            .update(self.state.queue(), self.state.surface_config());
        self.time_renderer
            .prepare(5.0, self.state.device(), self.state.queue())?;

        if config.fps.enable {
            self.fps_renderer.set_text(&format!(
                "{:.3}",
                config.fps.frame_limit as f64 - 1.0 / self.current_frame as f64
            ));
        } else {
            self.fps_renderer.set_text("");
        }
        self.fps_renderer.set_config(&config.fps.config);
        self.fps_renderer
            .update(self.state.queue(), self.state.surface_config());
        self.fps_renderer
            .prepare(5.0, self.state.device(), self.state.queue())?;

        Ok(())
    }

    fn update_imgui(&mut self, config: &mut NeedleConfig) -> NeedleErr<()> {
        self.imgui_state.setup(&self.window, |ui, settings_mode| {
            let window = ui.window(Self::NEEDLE_IMGUI_WINDOW_TITLE);
            let mut mode: u8 = u8::from(*settings_mode);
            let mut save_result: NeedleErr<()> = Ok(());

            window
                .size(Self::NEEDLE_IMGUI_WINDOW_SIZE, Condition::FirstUseEver)
                .build(|| {
                    // --- Mode Selection ---
                    if ui
                        .slider_config(
                            "Settings",
                            ImguiMode::Background.into(),
                            ImguiMode::Fps.into(),
                        )
                        .display_format(match settings_mode {
                            ImguiMode::Background => "Background",
                            ImguiMode::ClockTimer => "Clock/Timer",
                            ImguiMode::Fps => "FPS",
                            _ => "",
                        })
                        .build(&mut mode)
                    {
                        *settings_mode = mode.into();
                    }
                    ui.separator();

                    match settings_mode {
                        ImguiMode::Background => {
                            let mut background_color = config
                                .background_color
                                .iter()
                                .map(|val| (*val * 255.0) as u8)
                                .collect::<Vec<_>>();

                            ui.text("Color:");
                            background_color
                                .iter_mut()
                                .zip(Self::background_color_tag())
                                .enumerate()
                                .for_each(|(i, (color, tag))| {
                                    if ui.slider(tag, 0, 255, color) {
                                        config.background_color[i] = *color as f32 / 255.0;
                                    }
                                });
                        }
                        ImguiMode::ClockTimer => {
                            // --- Font selection ---
                            let fonts = self.time_renderer.fonts_mut();
                            let available_fonts = fonts.available_fonts();
                            let font_names = fonts.font_names().unwrap_or([].into());
                            let font_names = font_names
                                .iter()
                                .map(|font| font.as_str())
                                .collect::<Vec<_>>();
                            let mut clock_font = font_names
                                .iter()
                                .enumerate()
                                .find(|(_, font)| {
                                    font.to_string()
                                        == config.time.font.clone().unwrap_or("".to_string())
                                })
                                .map(|(idx, _)| idx as i32)
                                .unwrap_or(0);

                            if ui.list_box("Font:", &mut clock_font, font_names.as_ref(), 5) {
                                let font = &available_fonts[clock_font as usize];

                                config.time.font = Some(font.font.to_string());
                                if let Err(e) = self.time_renderer.set_font(&font.font) {
                                    log::error!("{font:?}");
                                    log::error!("{e}");
                                }
                            }
                            ui.separator();

                            // --- Font color ---
                            ui.text("Text Color:");
                            ui.slider("red (text)", 0, 255, &mut config.time.config.color[0]);
                            ui.slider("green (text)", 0, 255, &mut config.time.config.color[1]);
                            ui.slider("blue (text)", 0, 255, &mut config.time.config.color[2]);

                            // --- Font scale ---
                            let mut clock_scale = (config.time.config.scale * 100.0) as u8;
                            if ui.slider("Text Scale", 0, u8::MAX, &mut clock_scale) {
                                config.time.config.scale = if clock_scale > 0 {
                                    clock_scale as f32 / 50.0
                                } else {
                                    1.0 / 50.0
                                };
                            }
                            ui.separator();

                            // --- Clock position ---
                            let mut clock_position = config.time.config.position.into();

                            if ui.list_box(
                                "Clock Position",
                                &mut clock_position,
                                &[
                                    "Top Left",
                                    "Top",
                                    "Top Right",
                                    "Left",
                                    "Center",
                                    "Right",
                                    "Bottom Left",
                                    "Bottom",
                                    "Bottom Right",
                                ],
                                9,
                            ) {
                                let position = Position::from(clock_position);

                                if config.fps.config.position != position {
                                    config.time.config.position = position;
                                }
                            }
                            ui.separator();
                            // --- Format Mode ---
                            let mut view_mode: u8 = config.time.format.into();

                            ui.text("Mode:");
                            if ui
                                .slider_config("Format Mode", 0, 1)
                                .display_format(format!("{}", config.time.format))
                                .build(&mut view_mode)
                            {
                                config.time.format = view_mode.into();
                                self.clock_info.set_format(config.time.format);
                            }
                            ui.separator();

                            // --- Clock Mode ---
                            let mut clock_mode: u8 = self.clock_info.mode().into();
                            let mut countdown_duration =
                                if let OpMode::CountDownTimer(duration) = self.clock_info.mode() {
                                    duration
                                } else {
                                    Duration::new(0, 0)
                                };

                            if ui
                                .slider_config("Clock Mode", 0, 2)
                                .display_format(format!("{}", self.clock_info.mode()))
                                .build(&mut clock_mode)
                            {
                                match clock_mode.into() {
                                    OpMode::Clock => {
                                        self.clock_info.set_mode(OpMode::Clock);
                                    }
                                    OpMode::CountUpTimer => {
                                        self.clock_info.set_mode(OpMode::CountUpTimer);
                                        ui.text("Press \"SPACE\" to start/stop timer");
                                    }
                                    OpMode::CountDownTimer(_) => {
                                        self.clock_info
                                            .set_mode(OpMode::CountDownTimer(countdown_duration));
                                    }
                                }
                            }

                            match self.clock_info.mode() {
                                OpMode::CountDownTimer(_) => {
                                    let mut countdown_sec = 0;

                                    ui.text("Press \"SPACE\" to start/stop timer");
                                    if ui
                                        .input_int("Countdown Duration", &mut countdown_sec)
                                        .build()
                                    {
                                        countdown_duration = Duration::new(countdown_sec as u64, 0)
                                    }
                                    self.clock_info
                                        .set_mode(OpMode::CountDownTimer(countdown_duration));
                                }
                                OpMode::CountUpTimer => {
                                    ui.text("Press \"SPACE\" to start/stop timer");
                                }
                                _ => (),
                            }
                        }
                        ImguiMode::Fps => {
                            // --- Enable/Disable FPS visualization ---
                            let mut fps_enable = if config.fps.enable { 1 } else { 0 };

                            if ui.slider("Toggle FPS visualization", 0, 1, &mut fps_enable) {
                                config.fps.enable = fps_enable % 2 == 1;
                            }
                            ui.separator();

                            // FPS text color
                            ui.text("Text Color:");
                            ui.slider("red (fps):", 0, 255, &mut config.fps.config.color[0]);
                            ui.slider("green (fps):", 0, 255, &mut config.fps.config.color[1]);
                            ui.slider("blue (fps):", 0, 255, &mut config.fps.config.color[2]);
                            ui.separator();

                            // --- FPS text position ---
                            let mut fps_position: i32 = config.fps.config.position.into();

                            if ui.list_box(
                                "FPS Position",
                                &mut fps_position,
                                &["Top Left", "Top Right", "Bottom Left", "Bottom Right"],
                                4,
                            ) {
                                let position = match fps_position {
                                    0 => Position::TopLeft,
                                    1 => Position::TopRight,
                                    2 => Position::BottomLeft,
                                    _ => Position::BottomRight,
                                };

                                if config.time.config.position != position {
                                    config.fps.config.position = position;
                                }
                            }
                        }
                        _ => (),
                    }

                    // Save current settings
                    ui.separator();
                    ui.text("Press \"INSERT\" to toggle menu.");
                    ui.text("Save config:");
                    if ui.button("Save") {
                        save_result = config.save_config();
                    }

                    // Description
                    //  - Repository
                    //  - License
                    ui.separator();
                    ui.text("Repository:");
                    ui.text("https://github.com/bonohub13/needle");
                    ui.text("License: MIT");
                });

            save_result
        })
    }

    fn render_needle(&mut self, view: &wgpu::TextureView) -> NeedleErr<()> {
        self.state.render(|encoder| {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&NeedleLabel::RenderPass("").to_string()),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
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

    fn render(&mut self, config: &mut NeedleConfig) -> Result<()> {
        let texture = self.state.get_current_texture()?;
        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.update_imgui(config)?;
        self.update(config)?;
        self.window.pre_present_notify();
        if let Err(err) = self.render_needle(&view) {
            match err {
                NeedleError::Lost | NeedleError::Outdated => {
                    let size = self.window.inner_size();

                    self.resize(&size);
                }
                NeedleError::OutOfMemory | NeedleError::RemovedFromAtlas => {
                    log::error!("{}", NeedleError::OutOfMemory);

                    return Err(err.into());
                }
                NeedleError::Timeout => log::warn!("{err}"),
                NeedleError::Other => log::error!("{err}"),
                _ => (),
            }
        }

        self.imgui_state.render(&self.state, &view)?;

        texture.present();

        Ok(())
    }

    fn create_renderers(
        window: Arc<Window>,
        config: Rc<RefCell<NeedleConfig>>,
        state: &State,
    ) -> Result<(ShaderRenderer, TextRenderer, TextRenderer)> {
        let window_size = window.inner_size();
        let window_scale_factor = window.scale_factor();
        let depth_stencil_state = Texture::default_depth_stencil();
        let (background_vertices, indices) = Vertex::indexed_rectangle(
            [1.0, 1.0],
            [0.0, 0.0],
            0.1,
            &config.borrow().background_color,
        );
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
        let mut time_renderer = TextRenderer::new(
            state,
            &config.borrow().time.config,
            config.borrow().time.font.clone(),
            &window_size,
            window_scale_factor,
            state.surface_config().format,
            Some(depth_stencil_state.clone()),
        )?;
        let fps_renderer = TextRenderer::new(
            state,
            &config.borrow().fps.config,
            None,
            &window_size,
            window_scale_factor,
            state.surface_config().format,
            Some(depth_stencil_state.clone()),
        )?;

        time_renderer
            .fonts_mut()
            .query_fonts(Some(FontTypes::Monospace))?;

        Ok((background_renderer, time_renderer, fps_renderer))
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

    const fn background_color_tag<'tag>() -> [&'tag str; 4] {
        [
            "red (background)",
            "green (background)",
            "blue (background)",
            "alpha (background)",
        ]
    }
}

impl Needle<'_> {
    const APP_NAME: &'static str = env!("CARGO_PKG_NAME");
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    const VERTEX_SHADER_DEFAULT_PATH: &'static str = "shaders/spv/shader.vert.spv";
    const FRAGMENT_SHADER_DEFAULT_PATH: &'static str = "shaders/spv/shader.frag.spv";
    const RELEASE_URL: &'static str = "https://github.com/bonohub13/needle/releases/download";

    pub fn set_config(&mut self, config: Rc<RefCell<NeedleConfig>>) -> Result<()> {
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
        let write_path =
            match NeedleConfig::config_path(false, Some(&format!("shaders/spv/{path}"))) {
                Ok(p) => Ok(p),
                Err(_) => Err(NeedleError::InvalidPath),
            }?;
        let src_url = format!("{}/{}/{}", Self::RELEASE_URL, Self::VERSION, path);
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

        log::debug!("URL : {src_url}");
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
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let (Some(base), Some(config)) = (self.base.as_mut(), self.config.as_ref()) {
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
                        log::error!("{e}");
                        event_loop.exit();
                    }
                }
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::Insert),
                            ..
                        },
                    ..
                } => {
                    base.imgui_state.toggle_imgui();
                }
                WindowEvent::Resized(physical_size) => {
                    base.resize(&physical_size);
                }
                WindowEvent::RedrawRequested => {
                    /* Check for window has been done in the if statement above */
                    base.window.request_redraw();
                    let frame_time = Instant::now();
                    base.imgui_state.update(frame_time);
                    if let Err(err) = base.render(&mut config.borrow_mut()) {
                        log::error!("{err}");

                        event_loop.exit();
                    }
                    base.next_frame += base.fps_limit;

                    if (base.fps_update - frame_time) > base.fps_update_limit {
                        base.fps_update = frame_time;
                        base.current_frame = 0;
                    }
                    std::thread::sleep(base.next_frame - frame_time);
                }
                _ => (),
            }

            base.imgui_state
                .handle_event(&base.window, window_id, event);
        }
    }
}
