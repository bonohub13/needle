// Copyright 2025 Kensuke Saito
// SPDX-License-Identifier: MIT

use crate::needle::renderer::NeedleRenderer;
use anyhow::Result;
use imgui::Condition;
use needle_core::{
    ImguiMode, ImguiState, NeedleConfig, NeedleErr, NeedleError, OpMode, Position, State, Time,
    TimeFormat,
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
    renderer: NeedleRenderer,
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
    const BACKGROUND_COLOR_COUNT: usize = 4;
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
        let renderer = NeedleRenderer::new(
            window.clone(),
            config.clone(),
            &state,
            vert_shader_path,
            frag_shader_path,
            None,
            None,
        )?;

        Ok(Self {
            window,
            state,
            imgui_state,
            renderer,
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
            self.renderer.resize(&self.state, size);
        }
    }

    /// Render single frame of all objects in needle
    pub fn render(&mut self, config: &mut NeedleConfig) -> Result<()> {
        self.state.device().poll(wgpu::PollType::Wait)?;
        let texture = self.state.get_current_texture()?;
        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.update_imgui(config)?;
        self.update(config)?;
        self.window.pre_present_notify();
        if let Err(err) = self.renderer.render(&mut self.state, &view) {
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
        self.state.device().poll(wgpu::PollType::Wait)?;
        texture.present();

        Ok(())
    }

    /// Update render content for new frame
    fn update(&mut self, config: &NeedleConfig) -> NeedleErr<()> {
        self.renderer
            .update(&self.state, config, &self.clock_info, self.current_frame)?;

        let event = self.state.queue().submit([]);

        match self
            .state
            .device()
            .poll(wgpu::PollType::WaitForSubmissionIndex(event))
        {
            Ok(_) => Ok(()),
            Err(_) => Err(NeedleError::Other),
        }
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
            let mut mode: i8 = i8::from(*settings_mode);
            let mut save_result: NeedleErr<()> = Ok(());

            window
                .size(NEEDLE_IMGUI_WINDOW_SIZE, Condition::FirstUseEver)
                .build(|| {
                    // --- Mode Selection ---
                    if ui
                        .slider_config(NEEDLE_IMGUI_SETTINGS, ImguiMode::BACKGROUND, ImguiMode::MAX)
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
                            let fonts = self.renderer.clock.fonts_mut();
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
                                if let Err(e) = self.renderer.clock.set_font(&font.font) {
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
                            let mut view_mode: i8 = config.time.format.into();

                            ui.text(CLOCK_TIMER_MODE);
                            if ui
                                .slider_config(
                                    CLOCK_TIMER_FORMAT_MODE,
                                    TimeFormat::HOUR_MIN_SEC,
                                    TimeFormat::MAX,
                                )
                                .display_format(format!("{}", config.time.format))
                                .build(&mut view_mode)
                            {
                                config.time.format = view_mode.into();
                                self.clock_info.set_format(config.time.format);
                            }
                            ui.separator();

                            // --- Clock Mode ---
                            let mut clock_mode: i8 = self.clock_info.mode().into();
                            let mut countdown_duration =
                                if let OpMode::CountDownTimer(duration) = self.clock_info.mode() {
                                    duration
                                } else {
                                    Duration::new(0, 0)
                                };

                            if ui
                                .slider_config(CLOCK_TIMER_CLOCK_MODE, OpMode::CLOCK, OpMode::MAX)
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
                                const OFFSET: i32 = Position::TOP_LEFT as i32;
                                const TOP_LEFT: i32 = Position::TOP_LEFT as i32 - OFFSET;
                                const TOP_RIGHT: i32 = Position::TOP_RIGHT as i32 - OFFSET;
                                const BOTTOM_LEFT: i32 = Position::BOTTOM_LEFT as i32 - OFFSET;
                                const BOTTOM_RIGHT: i32 = Position::BOTTOM_RIGHT as i32 - OFFSET;

                                let position = match fps_position {
                                    TOP_LEFT => Position::TopLeft,
                                    TOP_RIGHT => Position::TopRight,
                                    BOTTOM_LEFT => Position::BottomLeft,
                                    BOTTOM_RIGHT => Position::BottomRight,
                                    _ => config.fps.config.position,
                                };

                                if config.time.config.position != position {
                                    config.fps.config.position = position;
                                }
                            }
                        }
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

    #[inline]
    const fn background_color<'color>() -> [&'color str; NeedleBase::BACKGROUND_COLOR_COUNT] {
        [
            "red (background)",
            "green (background)",
            "blue (background)",
            "alpha (background)",
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
            "Center",
            "Top",
            "Bottom",
            "Left",
            "Right",
            "Top Left",
            "Top Right",
            "Bottom Left",
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
}
