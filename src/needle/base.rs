// Copyright 2025 Kensuke Saito
// SPDX-License-Identifier: MIT

use anyhow::Result;
use imgui::Condition;
use needle_core::{
    Buffer, FontTypes, ImguiMode, ImguiState, NeedleConfig, NeedleErr, NeedleError, NeedleLabel,
    OpMode, Position, Renderer, ShaderRenderer, ShaderRendererDescriptor, State, TextRenderer,
    Texture, Time, Vertex,
};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{event_loop::ActiveEventLoop, window::Window};

pub struct NeedleBase<'a> {
    pub window: Arc<Window>,
    state: State<'a>,
    pub imgui_state: ImguiState,
    depth_texture: Texture,
    background_renderer: ShaderRenderer,
    time_renderer: TextRenderer,
    fps_renderer: TextRenderer,
    clock_info: Time,
    pub current_frame: u64,
    pub next_frame: Instant,
    pub fps_update: Instant,
    pub fps_limit: Duration,
    pub fps_update_limit: Duration,
}

impl<'a> NeedleBase<'a> {
    // Imgui Tags
    const NEEDLE_IMGUI_SAVE_COUNT: usize = 2;
    const NEEDLE_IMGUI_DESCRIPTION_COUNT: usize = 4;
    //  - Background
    const BACKGROUND_COLOR_COUNT: usize = 3;
    //  - Clock Timer
    const CLOCK_TIMER_FONT_ROWS: usize = 5;
    const CLOCK_TIMER_FONT_COLOR_COUNT: usize = 3;
    const CLOCK_TIMER_POSITION_COUNT: usize = 9;
    //  - FPS
    const FPS_FONT_COLOR_COUNT: usize = 3;
    const FPS_POSITION_COUNT: usize = 4;

    /// Create new instance of new Needle primary application logic
    pub fn new(
        event_loop: &ActiveEventLoop,
        config: Rc<RefCell<NeedleConfig>>,
        title: &str,
        vert_shader_path: &str,
        frag_shader_path: &str,
    ) -> Result<Self> {
        let window = {
            let attr = Window::default_attributes()
                .with_title(title)
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
        let (background, time, fps) = Self::create_renderers(
            window.clone(),
            config.clone(),
            &state,
            vert_shader_path,
            frag_shader_path,
        )?;

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

    /// Start count down/count up timer.
    /// If clock mode is set to clock, this fails.
    pub fn start_clock(&mut self) -> NeedleErr<()> {
        match self.clock_info.mode() {
            OpMode::Clock => Err(NeedleError::TimerStartFailure),
            OpMode::CountDownTimer(_) | OpMode::CountUpTimer => {
                self.clock_info.toggle_timer();
                Ok(())
            }
        }
    }

    /// Resize render surface to new window size
    pub fn resize(&mut self, size: &winit::dpi::PhysicalSize<u32>) {
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

    /// Render single frame of all objects in needle
    pub fn render(&mut self, config: &mut NeedleConfig) -> Result<()> {
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

    /// Update render content for new frame
    fn update(&mut self, config: &NeedleConfig) -> NeedleErr<()> {
        const TEXT_RENDERER_MARGIN: f32 = 5.0;

        let (background_vertices, indices) =
            Vertex::indexed_rectangle([1.0, 1.0], [0.0, 0.0], 0.1, &config.background_color);
        let background_buffer = Buffer::new(
            &self.state,
            NeedleLabel::Buffer("Background"),
            &background_vertices,
            0,
            Some(&indices),
        );

        self.background_renderer.set_buffer(background_buffer)?;
        self.time_renderer.set_text(&self.clock_info.current_time());
        self.time_renderer.set_config(&config.time.config);
        self.time_renderer
            .update(self.state.queue(), self.state.surface_config());
        self.time_renderer.prepare(
            TEXT_RENDERER_MARGIN,
            self.state.device(),
            self.state.queue(),
        )?;

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
        self.fps_renderer.prepare(
            TEXT_RENDERER_MARGIN,
            self.state.device(),
            self.state.queue(),
        )?;

        Ok(())
    }

    /// Update Imgui UI for needle
    fn update_imgui(&mut self, config: &mut NeedleConfig) -> NeedleErr<()> {
        // Imgui Tags
        const NEEDLE_IMGUI_WINDOW_TITLE: &str = "Needle Settings";
        const NEEDLE_IMGUI_WINDOW_SIZE: [f32; 2] = [800.0, 600.0];
        const NEEDLE_IMGUI_SETTINGS: &str = "Settings";
        const NEEDLE_IMGUI_SAVE: &str = "Save";
        //  - Background
        const BACKGROUND_COLOR: &str = "Color:";
        //  - Clock Timer
        const CLOCK_TIMER_FONT: &str = "Font";
        const CLOCK_TIMER_FONT_COLOR: &str = "Font Color:";
        const CLOCK_TIMER_FONT_SCALE: &str = "Font Scale";
        const CLOCK_TIMER_POSITION: &str = "Clock Position";
        const CLOCK_TIMER_MODE: &str = "Mode:";
        const CLOCK_TIMER_FORMAT_MODE: &str = "Format Mode";
        const CLOCK_TIMER_CLOCK_MODE: &str = "Clock Mode";
        const CLOCK_TIMER_CLOCK_MODE_INFO: &str = "Press \"SPACE\" to start/stop timer";
        const CLOCK_TIMER_CLOCK_MODE_DURATION: &str = "Countdown Duration";
        //  - FPS
        const FPS_VISUALIZATION: &str = "Toggle FPS visualization";
        const FPS_FONT_COLOR: &str = "Font Color:";
        const FPS_POSITION: &str = "FPS Position";

        self.imgui_state.setup(&self.window, |ui, settings_mode| {
            let window = ui.window(NEEDLE_IMGUI_WINDOW_TITLE);
            let mut mode: u8 = u8::from(*settings_mode);
            let mut save_result: NeedleErr<()> = Ok(());

            window
                .size(NEEDLE_IMGUI_WINDOW_SIZE, Condition::FirstUseEver)
                .build(|| {
                    // --- Mode Selection ---
                    if ui
                        .slider_config(
                            NEEDLE_IMGUI_SETTINGS,
                            ImguiMode::Background.into(),
                            ImguiMode::Fps.into(),
                        )
                        .display_format(format!("{settings_mode}"))
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

                            ui.text(BACKGROUND_COLOR);
                            Self::background_color()
                                .iter()
                                .enumerate()
                                .for_each(|(i, tag)| {
                                    if ui.slider(tag, 0, 255, &mut background_color[i]) {
                                        config.background_color[i] =
                                            background_color[i] as f32 / 255.0;
                                    };
                                });
                        }
                        ImguiMode::ClockTimer => {
                            // --- Font selection ---
                            let fonts = self.time_renderer.fonts_mut();
                            let font_names = fonts.font_names().unwrap_or([].into());
                            let font_names = font_names
                                .iter()
                                .map(|font| font.as_str())
                                .collect::<Vec<_>>();
                            let mut clock_font = font_names
                                .iter()
                                .enumerate()
                                .find(|(_, font)| {
                                    **font == config.time.font.clone().unwrap_or("".to_string())
                                })
                                .map(|(idx, _)| idx as i32)
                                .unwrap_or(0);

                            if ui.list_box(
                                CLOCK_TIMER_FONT,
                                &mut clock_font,
                                font_names.as_ref(),
                                Self::CLOCK_TIMER_FONT_ROWS as i32,
                            ) {
                                let font = &fonts.available_fonts()[clock_font as usize];

                                config.time.font = Some(font.font.to_string());
                                if let Err(e) = self.time_renderer.set_font(&font.font) {
                                    log::error!("{font:?}");
                                    log::error!("{e}");
                                }
                            }
                            ui.separator();

                            // --- Font color ---
                            ui.text(CLOCK_TIMER_FONT_COLOR);
                            Self::clock_font_color()
                                .iter()
                                .enumerate()
                                .for_each(|(i, tag)| {
                                    ui.slider(tag, 0, 255, &mut config.time.config.color[i]);
                                });

                            // --- Font scale ---
                            let mut clock_scale = (config.time.config.scale * 100.0) as u8;
                            if ui.slider(CLOCK_TIMER_FONT_SCALE, 1, u8::MAX, &mut clock_scale) {
                                config.time.config.scale = clock_scale as f32 / 50.0;
                            }
                            ui.separator();

                            // --- Clock position ---
                            let mut clock_position = config.time.config.position.into();

                            if ui.list_box(
                                CLOCK_TIMER_POSITION,
                                &mut clock_position,
                                &Self::clock_position(),
                                Self::CLOCK_TIMER_POSITION_COUNT as i32,
                            ) {
                                let position = Position::from(clock_position);

                                if config.fps.config.position != position {
                                    config.time.config.position = position;
                                }
                            }
                            ui.separator();
                            // --- Format Mode ---
                            let mut view_mode: u8 = config.time.format.into();

                            ui.text(CLOCK_TIMER_MODE);
                            if ui
                                .slider_config(CLOCK_TIMER_FORMAT_MODE, 0, 1)
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
                                .slider_config(CLOCK_TIMER_CLOCK_MODE, 0, 2)
                                .display_format(format!("{}", self.clock_info.mode()))
                                .build(&mut clock_mode)
                            {
                                match clock_mode.into() {
                                    OpMode::Clock => {
                                        self.clock_info.set_mode(OpMode::Clock);
                                    }
                                    OpMode::CountUpTimer => {
                                        self.clock_info.set_mode(OpMode::CountUpTimer);
                                        ui.text(CLOCK_TIMER_CLOCK_MODE_INFO);
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

                                    ui.text(CLOCK_TIMER_CLOCK_MODE_INFO);
                                    if ui
                                        .input_int(
                                            CLOCK_TIMER_CLOCK_MODE_DURATION,
                                            &mut countdown_sec,
                                        )
                                        .build()
                                    {
                                        countdown_duration = Duration::new(countdown_sec as u64, 0)
                                    }
                                    self.clock_info
                                        .set_mode(OpMode::CountDownTimer(countdown_duration));
                                }
                                OpMode::CountUpTimer => {
                                    ui.text(CLOCK_TIMER_CLOCK_MODE_INFO);
                                }
                                _ => (),
                            }
                        }
                        ImguiMode::Fps => {
                            // --- Enable/Disable FPS visualization ---
                            let mut fps_enable = if config.fps.enable { 1 } else { 0 };

                            if ui
                                .slider_config(FPS_VISUALIZATION, 0, 1)
                                .display_format(Self::fps_enable(config.fps.enable))
                                .build(&mut fps_enable)
                            {
                                config.fps.enable = fps_enable % 2 == 1;
                            }
                            ui.separator();

                            // FPS font color
                            ui.text(FPS_FONT_COLOR);
                            Self::fps_font_color()
                                .iter()
                                .enumerate()
                                .for_each(|(i, tag)| {
                                    ui.slider(tag, 0, u8::MAX, &mut config.fps.config.color[i]);
                                });
                            ui.separator();

                            // --- FPS text position ---
                            let mut fps_position: i32 = config.fps.config.position.into();

                            if ui.list_box(
                                FPS_POSITION,
                                &mut fps_position,
                                &Self::fps_position(),
                                Self::FPS_POSITION_COUNT as i32,
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
                    Self::save().iter().for_each(|tag| {
                        ui.text(tag);
                    });
                    if ui.button(NEEDLE_IMGUI_SAVE) {
                        save_result = config.save_config();
                    }

                    // Description
                    ui.separator();
                    Self::description().iter().for_each(|tag| ui.text(tag));
                });

            save_result
        })
    }

    /// Render single frame for needle
    fn render_needle(&mut self, view: &wgpu::TextureView) -> NeedleErr<()> {
        let color = wgpu::Color::TRANSPARENT;

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

    #[inline]
    const fn background_color<'color>() -> [&'color str; NeedleBase::BACKGROUND_COLOR_COUNT] {
        [
            "red (background)",
            "green (background)",
            "blue (background)",
        ]
    }

    #[inline]
    const fn clock_font_color<'color>() -> [&'color str; NeedleBase::CLOCK_TIMER_FONT_COLOR_COUNT] {
        ["red (text)", "green (text)", "blue (text)"]
    }

    #[inline]
    const fn clock_position<'position>() -> [&'position str; NeedleBase::CLOCK_TIMER_POSITION_COUNT]
    {
        [
            "Top Left",
            "Top",
            "Top Right",
            "Left",
            "Center",
            "Right",
            "Bottom Left",
            "Bottom",
            "Bottom Right",
        ]
    }

    #[inline]
    const fn fps_enable<'enable>(enable: bool) -> &'enable str {
        if enable {
            "Enable"
        } else {
            "Disable"
        }
    }

    #[inline]
    const fn fps_font_color<'color>() -> [&'color str; NeedleBase::FPS_FONT_COLOR_COUNT] {
        ["red (fps)", "green (fps)", "blue (fps)"]
    }

    #[inline]
    const fn fps_position<'position>() -> [&'position str; NeedleBase::FPS_POSITION_COUNT] {
        ["Top Left", "Top Right", "Bottom Left", "Bottom Right"]
    }

    #[inline]
    const fn save<'save>() -> [&'save str; NeedleBase::NEEDLE_IMGUI_SAVE_COUNT] {
        ["Press \"INSERT\" to toggle menu.", "Save config:"]
    }

    #[inline]
    const fn description<'desc>() -> [&'desc str; NeedleBase::NEEDLE_IMGUI_DESCRIPTION_COUNT] {
        [
            "Repository:",
            "  - https://github.com/bonohub13/needle",
            "License:",
            "  - MIT",
        ]
    }

    fn create_renderers(
        window: Arc<Window>,
        config: Rc<RefCell<NeedleConfig>>,
        state: &State,
        vert_shader_path: &str,
        frag_shader_path: &str,
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
        let background_buffer = Buffer::new(
            state,
            NeedleLabel::Buffer("Background"),
            &background_vertices,
            0,
            Some(&indices),
        );
        let background_renderer = {
            let desc = ShaderRendererDescriptor {
                vert_shader_path: NeedleConfig::config_path(false, Some(vert_shader_path))?,
                frag_shader_path: NeedleConfig::config_path(false, Some(frag_shader_path))?,
                buffer: background_buffer,
                vertex_buffer_layouts: Vertex::buffer_layout(),
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
}
