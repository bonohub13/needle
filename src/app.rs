// Copyright 2025 Kensuke Saito
// SPDX-License-Identifier: MIT

use super::Needle;
use anyhow::Result;
use needle_core::NeedleConfig;
use std::{cell::RefCell, rc::Rc};
use winit::event_loop::{ControlFlow, EventLoop};

pub fn run(config: Rc<RefCell<NeedleConfig>>) -> Result<()> {
    let event_loop = EventLoop::new()?;
    let mut app = Needle::default();

    app.set_config(config)?;
    event_loop.set_control_flow(ControlFlow::Poll);
    match event_loop.run_app(&mut app) {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("{e}");

            Err(e)
        }
    }?;

    Ok(())
}
