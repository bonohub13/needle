// Copyright 2025 Kensuke Saito
// SPDX-License-Identifier: MIT

mod base;

use anyhow::Result;
use base::NeedleBase;
use needle_core::{NeedleConfig, NeedleError};
use std::{
    cell::RefCell,
    fs::{self, OpenOptions},
    io::copy,
    rc::Rc,
    time::Instant,
};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

#[derive(Default)]
pub struct Needle<'window> {
    base: Option<NeedleBase<'window>>,
    config: Option<Rc<RefCell<NeedleConfig>>>,
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

    /// Download shader from Github
    fn download_shader() -> Result<()> {
        let vert_shader = "shader.vert.spv";
        let frag_shader = "shader.frag.spv";

        Self::write(vert_shader)?;
        Self::write(frag_shader)?;

        Ok(())
    }

    /// Download specified shader
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
                match NeedleBase::new(
                    event_loop,
                    config.clone(),
                    Self::APP_NAME,
                    Self::VERTEX_SHADER_DEFAULT_PATH,
                    Self::FRAGMENT_SHADER_DEFAULT_PATH,
                ) {
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
