// Copyright 2025 Kensuke Saito
// SPDX-License-Identifier: MIT

mod params;
mod ui;

use crate::needle::renderer::NeedleRenderer;
use anyhow::Result;
use needle_core::{
    ImguiState, NeedleConfig, NeedleErr, NeedleError, OpMode, ShaderDescriptor, State, Time,
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
    /// Create new instance of new Needle primary application logic
    pub fn new(
        event_loop: &ActiveEventLoop,
        config: Rc<RefCell<NeedleConfig>>,
        title: &str,
        background_shader_desc: &ShaderDescriptor,
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
            background_shader_desc,
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
}
