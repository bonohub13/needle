// Copyright 2025 Kensuke Saito
// SPDX-License-Identifier: MIT

mod mode;

use anyhow::Result;
use imgui::{Condition, Context, FontConfig, FontSource, MouseCursor};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use mode::ImguiMode;
use needle_core::{
    FontTypes, NeedleConfig, NeedleErr, NeedleError, NeedleLabel, OpMode, Position, Renderer,
    ShaderRenderer, ShaderRendererDescriptor, State, TextRenderer, Texture, Time, Vertex,
};
use std::{
    cell::{RefCell, RefMut},
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
    window::{Window, WindowId},
};

pub struct ImguiState {
    context: Context,
    platform: WinitPlatform,
    renderer: imgui_wgpu::Renderer,
    last_cursor: Option<MouseCursor>,
    last_frame: Instant,
    show_imgui: bool,
    settings_mode: ImguiMode,
}

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

impl ImguiState {
    const NEEDLE_IMGUI_WINDOW_TITLE: &'static str = "Needle Settings";
    const NEEDLE_IMGUI_WINDOW_SIZE: [f32; 2] = [800.0, 600.0];

    fn new(window: Arc<Window>, config: Rc<RefCell<NeedleConfig>>, state: &State) -> Self {
        let mut context = Self::create_context(window.clone(), config.clone());
        let platform = Self::create_platform(window.clone(), &mut context);
        let renderer = Self::create_renderer(&mut context, state);

        Self {
            context,
            platform,
            renderer,
            last_cursor: None,
            last_frame: Instant::now(),
            show_imgui: true,
            settings_mode: ImguiMode::Background,
        }
    }

    fn update(&mut self, new_frame: Instant) {
        self.context
            .io_mut()
            .update_delta_time(new_frame - self.last_frame);
        self.last_frame = new_frame;
    }

    fn handle_event(
        &mut self,
        window: &Window,
        window_id: WindowId,
        event: winit::event::WindowEvent,
    ) {
        self.platform.handle_event::<()>(
            self.context.io_mut(),
            window,
            &winit::event::Event::WindowEvent { event, window_id },
        )
    }

    fn toggle_imgui(&mut self) {
        self.show_imgui = !self.show_imgui;
    }

    fn create_context(window: Arc<Window>, _config: Rc<RefCell<NeedleConfig>>) -> Context {
        let mut context = Context::create();
        let hidpi_factor = window.scale_factor();
        let font_size = (13.0 * hidpi_factor) as f32;

        context.set_ini_filename(None);
        context.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        context.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);

        context
    }

    fn create_platform(window: Arc<Window>, context: &mut Context) -> WinitPlatform {
        let mut platform = WinitPlatform::new(context);

        platform.attach_window(context.io_mut(), &window, HiDpiMode::Default);

        platform
    }

    fn create_renderer(context: &mut Context, state: &State) -> imgui_wgpu::Renderer {
        let config = imgui_wgpu::RendererConfig {
            texture_format: state.surface_config().format,
            ..Default::default()
        };

        imgui_wgpu::Renderer::new(context, state.device(), state.queue(), config)
    }
}

impl<'a> NeedleBase<'a> {
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

    fn update(&mut self, config: RefMut<NeedleConfig>) -> NeedleErr<()> {
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

    fn render_needle(&mut self, view: &wgpu::TextureView) -> NeedleErr<()> {
        let color = wgpu::Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        };

        self.state.render(|encoder| {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&NeedleLabel::RenderPass("").to_string()),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
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

    fn render(&mut self, mut config: RefMut<NeedleConfig>) -> Result<()> {
        let texture = self.state.get_current_texture()?;
        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.imgui_state
            .platform
            .prepare_frame(self.imgui_state.context.io_mut(), &self.window)?;

        let ui = self.imgui_state.context.frame();

        if self.imgui_state.show_imgui {
            let window = ui.window(ImguiState::NEEDLE_IMGUI_WINDOW_TITLE);
            let mut mode: u8 = self.imgui_state.settings_mode.into();
            let mut save_result = Ok(());

            window
                .size(
                    ImguiState::NEEDLE_IMGUI_WINDOW_SIZE,
                    Condition::FirstUseEver,
                )
                .build(|| {
                    ui.slider_config(
                        "Settings",
                        ImguiMode::Background.into(),
                        ImguiMode::Fps.into(),
                    )
                    .display_format(match self.imgui_state.settings_mode {
                        ImguiMode::Background => "Background",
                        ImguiMode::ClockTimer => "Clock/Timer",
                        ImguiMode::Fps => "FPS",
                        _ => "",
                    })
                    .build(&mut mode);
                    self.imgui_state.settings_mode = mode.into();
                    ui.separator();
                    match self.imgui_state.settings_mode {
                        ImguiMode::Background => {
                            let mut background_color = config
                                .background_color
                                .iter()
                                .map(|val| (*val * 255.0) as u8)
                                .collect::<Vec<_>>();
                            let mut update_flg = 0;

                            ui.text("Color:");
                            if ui.slider("red (background)", 0, 255, &mut background_color[0]) {
                                update_flg += 1
                            };
                            if ui.slider("green (background)", 0, 255, &mut background_color[1]) {
                                update_flg += 1
                            };
                            if ui.slider("blue (background)", 0, 255, &mut background_color[2]) {
                                update_flg += 1
                            };

                            if update_flg > 0 {
                                config.background_color[0] = background_color[0] as f32 / 255.0;
                                config.background_color[1] = background_color[1] as f32 / 255.0;
                                config.background_color[2] = background_color[2] as f32 / 255.0;
                                config.background_color[3] = background_color[3] as f32 / 255.0;
                            }
                        }
                        ImguiMode::ClockTimer => {
                            let mut clock_scale = (config.time.config.scale * 100.0) as u8;
                            let mut clock_position = config.time.config.position.into();
                            let mut view_mode: u8 = config.time.format.into();
                            let mut clock_mode: u8 = self.clock_info.mode().into();
                            let mut countdown_duration =
                                if let OpMode::CountDownTimer(duration) = self.clock_info.mode() {
                                    duration
                                } else {
                                    Duration::new(0, 0)
                                };
                            let available_fonts = self.time_renderer.fonts_mut().available_fonts();
                            let font_names = self
                                .time_renderer
                                .fonts_mut()
                                .font_names()
                                .unwrap_or([].into());
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
                                    log::error!("{:?}", font);
                                    log::error!("{}", e);
                                }
                            }
                            ui.text("Text Color:");
                            ui.slider("red (text)", 0, 255, &mut config.time.config.color[0]);
                            ui.slider("green (text)", 0, 255, &mut config.time.config.color[1]);
                            ui.slider("blue (text)", 0, 255, &mut config.time.config.color[2]);
                            if ui.slider("Text Scale", 0, u8::MAX, &mut clock_scale) {
                                config.time.config.scale = if clock_scale > 0 {
                                    clock_scale as f32 / 50.0
                                } else {
                                    1.0 / 50.0
                                };
                            }
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
                            ui.text("Mode:");
                            if ui
                                .slider_config("Format Mode", 0, 1)
                                .display_format(format!("{}", config.time.format))
                                .build(&mut view_mode)
                            {
                                config.time.format = view_mode.into();
                                self.clock_info.set_format(config.time.format);
                            }
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
                            let mut fps_enable = if config.fps.enable { 1 } else { 0 };
                            let mut fps_position: i32 = config.fps.config.position.into();

                            if ui.slider("Toggle FPS visualization", 0, 1, &mut fps_enable) {
                                config.fps.enable = fps_enable % 2 == 1;
                            }
                            ui.text("Text Color:");
                            ui.slider("red (fps):", 0, 255, &mut config.fps.config.color[0]);
                            ui.slider("green (fps):", 0, 255, &mut config.fps.config.color[1]);
                            ui.slider("blue (fps):", 0, 255, &mut config.fps.config.color[2]);
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
                    ui.separator();
                    ui.text("Press \"INSERT\" to toggle menu.");
                    ui.text("Save config:");
                    if ui.button("Save") {
                        save_result = config.save_config();
                    }
                    ui.separator();
                    ui.text("Repository:");
                    ui.text("https://github.com/bonohub13/needle");
                    ui.text("License: MIT");
                });

            save_result?;
        }

        if self.imgui_state.last_cursor != ui.mouse_cursor() {
            self.imgui_state.last_cursor = ui.mouse_cursor();
            self.imgui_state.platform.prepare_render(ui, &self.window);
        }

        self.update(config)?;
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
                NeedleError::Timeout => log::warn!("{}", err),
                NeedleError::Other => log::error!("{}", err),
                _ => (),
            }
        }

        let mut encoder = self
            .state
            .device()
            .create_command_encoder(&Default::default());
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(&NeedleLabel::ImguiWindow("").to_string()),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        self.imgui_state.renderer.render(
            self.imgui_state.context.render(),
            self.state.queue(),
            self.state.device(),
            &mut render_pass,
        )?;

        drop(render_pass);

        self.state.queue().submit(Some(encoder.finish()));

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
            None,
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
            match NeedleConfig::config_path(false, Some(&format!("shaders/spv/{}", path))) {
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
                        log::error!("{}", e);
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
                    if let Err(err) = base.render(config.borrow_mut()) {
                        log::error!("{}", err);

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
