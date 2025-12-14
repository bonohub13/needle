// Copyright 2025 Kensuke Saito
// SPDX-License-Identifier: MIT

use imgui::Condition;
use needle_core::{ImguiMode, NeedleConfig, NeedleErr, OpMode, Position, TimeFormat};
use std::time::Duration;

impl super::NeedleBase<'_> {
    /// Update Imgui UI for needle
    pub(crate) fn update_imgui(&mut self, config: &mut NeedleConfig) -> NeedleErr<()> {
        self.imgui_state.setup(&self.window, |ui, settings_mode| {
            let window = ui.window(Self::WINDOW_TITLE);
            let mut mode: i8 = i8::from(*settings_mode);
            let mut save_result: NeedleErr<()> = Ok(());

            window
                .size(Self::WINDOW_SIZE, Condition::FirstUseEver)
                .build(|| {
                    // --- Mode Selection ---
                    if ui
                        .slider_config(Self::SETTINGS_TAG, ImguiMode::BACKGROUND, ImguiMode::MAX)
                        .display_format(format!("{settings_mode}"))
                        .build(&mut mode)
                    {
                        *settings_mode = mode.into();
                    }
                    ui.separator();

                    match settings_mode {
                        ImguiMode::Background => {
                            ui.text(Self::BACKGROUND_COLOR_TAG);
                            Self::background_color()
                                .iter()
                                .enumerate()
                                .for_each(|(i, tag)| {
                                    if ui.slider(
                                        tag,
                                        Self::BACKGROUND_COLOR_RANGE[0],
                                        Self::BACKGROUND_COLOR_RANGE[1],
                                        &mut config.background_color[i],
                                    ) {};
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
                                    **font == config.time.font.clone().unwrap_or_default()
                                })
                                .map(|(idx, _)| idx as i32)
                                .unwrap_or(0);

                            if ui.list_box(
                                Self::CLOCK_TIMER_FONT_TAG,
                                &mut clock_font,
                                font_names.as_ref(),
                                Self::CLOCK_TIMER_LIST_ROW_LENGTH,
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
                            ui.text(Self::CLOCK_TIMER_FONT_COLOR_TAG);
                            Self::clock_font_color()
                                .iter()
                                .enumerate()
                                .for_each(|(i, tag)| {
                                    ui.slider(tag, 0, 255, &mut config.time.config.color[i]);
                                });

                            // --- Font scale ---
                            let mut clock_scale = (config.time.config.scale * 100.0) as u8;
                            if ui.slider(
                                Self::CLOCK_TIMER_FONT_SCALE_TAG,
                                Self::CLOCK_TIMER_FONT_SCALE_RANGE[0],
                                Self::CLOCK_TIMER_FONT_SCALE_RANGE[1],
                                &mut clock_scale,
                            ) {
                                config.time.config.scale = clock_scale as f32 / 50.0;
                            }
                            ui.separator();

                            // --- Clock position ---
                            let mut clock_position = config.time.config.position.into();

                            if ui.list_box(
                                Self::CLOCK_TIMER_POSITION_TAG,
                                &mut clock_position,
                                &Self::clock_position(),
                                Self::CLOCK_TIMER_LIST_ROW_LENGTH,
                            ) {
                                let position = Position::from(clock_position);

                                if config.fps.config.position != position {
                                    config.time.config.position = position;
                                }
                            }
                            ui.separator();
                            // --- Format Mode ---
                            let mut view_mode: i8 = config.time.format.into();

                            ui.text(Self::CLOCK_TIMER_MODE_TAG);
                            if ui
                                .slider_config(
                                    Self::CLOCK_TIMER_FORMAT_MODE_TAG,
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
                                .slider_config(
                                    Self::CLOCK_TIMER_CLOCK_MODE_TAG,
                                    OpMode::CLOCK,
                                    OpMode::MAX,
                                )
                                .display_format(format!("{}", self.clock_info.mode()))
                                .build(&mut clock_mode)
                            {
                                match clock_mode.into() {
                                    OpMode::Clock => {
                                        self.clock_info.set_mode(OpMode::Clock);
                                    }
                                    OpMode::CountUpTimer => {
                                        self.clock_info.set_mode(OpMode::CountUpTimer);
                                        ui.text(Self::CLOCK_TIMER_CLOCK_MODE_INFO);
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

                                    ui.text(Self::CLOCK_TIMER_CLOCK_MODE_INFO);
                                    if ui
                                        .input_int(
                                            Self::CLOCK_TIMER_CLOCK_MODE_DURATION_TAG,
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
                                    ui.text(Self::CLOCK_TIMER_CLOCK_MODE_INFO);
                                }
                                _ => (),
                            }
                        }
                        ImguiMode::Fps => {
                            // --- Enable/Disable FPS visualization ---
                            let mut fps_enable = if config.fps.enable { 1 } else { 0 };

                            if ui
                                .slider_config(
                                    Self::FPS_VISUALIZATION_TAG,
                                    Self::FPS_VISUALIZATION_RANGE[0],
                                    Self::FPS_VISUALIZATION_RANGE[1],
                                )
                                .display_format(Self::fps_enable(config.fps.enable))
                                .build(&mut fps_enable)
                            {
                                config.fps.enable = fps_enable % 2 == 1;
                            }
                            ui.separator();

                            // FPS font color
                            ui.text(Self::FPS_FONT_COLOR_TAG);
                            Self::fps_font_color()
                                .iter()
                                .enumerate()
                                .for_each(|(i, tag)| {
                                    ui.slider(
                                        tag,
                                        Self::FPS_FONT_COLOR_RANGE[0],
                                        Self::FPS_FONT_COLOR_RANGE[1],
                                        &mut config.fps.config.color[i],
                                    );
                                });
                            ui.separator();

                            // --- FPS text position ---
                            let mut fps_position: i32 = config.fps.config.position.into();

                            if ui.list_box(
                                Self::FPS_POSITION_TAG,
                                &mut fps_position,
                                &Self::fps_position(),
                                Self::FPS_LIST_ROW_LENGTH,
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
                    if ui.button(Self::SAVE_TAG) {
                        save_result = config.save_config();
                    }

                    // Description
                    ui.separator();
                    Self::description().iter().for_each(|tag| ui.text(tag));
                });

            save_result
        })
    }
}
