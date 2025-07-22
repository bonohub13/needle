// Copyright 2025 Kensuke Saito
// SPDX-License-Identifier: MIT

use crate::needle::{mode::ImguiMode, NeedleLabel};
use anyhow::Result;
use imgui::{Condition, Context, FontConfig, FontSource, MouseCursor};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use needle_core::{NeedleConfig, OpMode, Position, State, TextRenderer, Time};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};
use winit::window::{Window, WindowId};

pub struct ImguiState {
    context: Context,
    platform: WinitPlatform,
    renderer: imgui_wgpu::Renderer,
    last_cursor: Option<MouseCursor>,
    last_frame: Instant,
    show_imgui: bool,
    settings_mode: ImguiMode,
}

impl ImguiState {
    const NEEDLE_IMGUI_WINDOW_TITLE: &'static str = "Needle Settings";
    const NEEDLE_IMGUI_WINDOW_SIZE: [f32; 2] = [800.0, 600.0];

    pub fn new(window: Arc<Window>, config: Rc<RefCell<NeedleConfig>>, state: &State) -> Self {
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

    pub fn update(&mut self, new_frame: Instant) {
        self.context
            .io_mut()
            .update_delta_time(new_frame - self.last_frame);
        self.last_frame = new_frame;
    }

    pub fn setup(
        &mut self,
        window: &Window,
        config: &mut NeedleConfig,
        clock_info: &mut Time,
        time_renderer: &mut TextRenderer,
    ) -> Result<()> {
        self.platform.prepare_frame(self.context.io_mut(), window)?;
        let ui = self.context.new_frame();

        if self.show_imgui {
            let window = ui.window(Self::NEEDLE_IMGUI_WINDOW_TITLE);
            let mut mode: u8 = self.settings_mode.into();
            let mut save_result = Ok(());

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
                        .display_format(match self.settings_mode {
                            ImguiMode::Background => "Background",
                            ImguiMode::ClockTimer => "Clock/Timer",
                            ImguiMode::Fps => "FPS",
                            _ => "",
                        })
                        .build(&mut mode)
                    {
                        self.settings_mode = mode.into();
                    }
                    ui.separator();

                    match self.settings_mode {
                        ImguiMode::Background => {
                            let mut background_color = config
                                .background_color
                                .iter()
                                .map(|val| (*val * 255.0) as u8)
                                .collect::<Vec<_>>();

                            ui.text("Color:");
                            if ui.slider("red (background)", 0, 255, &mut background_color[0]) {
                                config.background_color[0] = background_color[0] as f32 / 255.0;
                            };
                            if ui.slider("green (background)", 0, 255, &mut background_color[1]) {
                                config.background_color[1] = background_color[1] as f32 / 255.0;
                            };
                            if ui.slider("blue (background)", 0, 255, &mut background_color[2]) {
                                config.background_color[2] = background_color[2] as f32 / 255.0;
                            };
                        }
                        ImguiMode::ClockTimer => {
                            // --- Font selection ---
                            let fonts = time_renderer.fonts_mut();
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
                                if let Err(e) = time_renderer.set_font(&font.font) {
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
                                clock_info.set_format(config.time.format);
                            }
                            ui.separator();

                            // --- Clock Mode ---
                            let mut clock_mode: u8 = clock_info.mode().into();
                            let mut countdown_duration =
                                if let OpMode::CountDownTimer(duration) = clock_info.mode() {
                                    duration
                                } else {
                                    Duration::new(0, 0)
                                };

                            if ui
                                .slider_config("Clock Mode", 0, 2)
                                .display_format(format!("{}", clock_info.mode()))
                                .build(&mut clock_mode)
                            {
                                match clock_mode.into() {
                                    OpMode::Clock => {
                                        clock_info.set_mode(OpMode::Clock);
                                    }
                                    OpMode::CountUpTimer => {
                                        clock_info.set_mode(OpMode::CountUpTimer);
                                        ui.text("Press \"SPACE\" to start/stop timer");
                                    }
                                    OpMode::CountDownTimer(_) => {
                                        clock_info
                                            .set_mode(OpMode::CountDownTimer(countdown_duration));
                                    }
                                }
                            }

                            match clock_info.mode() {
                                OpMode::CountDownTimer(_) => {
                                    let mut countdown_sec = 0;

                                    ui.text("Press \"SPACE\" to start/stop timer");
                                    if ui
                                        .input_int("Countdown Duration", &mut countdown_sec)
                                        .build()
                                    {
                                        countdown_duration = Duration::new(countdown_sec as u64, 0)
                                    }
                                    clock_info.set_mode(OpMode::CountDownTimer(countdown_duration));
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

            save_result?;
        }

        if self.last_cursor != ui.mouse_cursor() {
            self.last_cursor = ui.mouse_cursor();
            self.platform.prepare_render(ui, window);
        }

        Ok(())
    }

    pub fn render(&mut self, state: &State, view: &wgpu::TextureView) -> Result<()> {
        let mut encoder = state.device().create_command_encoder(&Default::default());
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(&NeedleLabel::ImguiWindow("").to_string()),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
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

        self.renderer.render(
            self.context.render(),
            state.queue(),
            state.device(),
            &mut render_pass,
        )?;

        drop(render_pass);

        state.queue().submit(Some(encoder.finish()));

        Ok(())
    }

    pub fn handle_event(
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

    pub fn toggle_imgui(&mut self) {
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
