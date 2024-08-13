/* Hide console when running a release build under Windows */
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use needle_core;
use std::time::Instant;
use winit::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

const APP_NAME: &'static str = "needle";

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    let app_info = needle_core::AppInfo::new("needle", 1);
    let app_base = needle_core::AppBase::new(&event_loop, &app_info)?;
    let mut current_time = Instant::now();

    let result = event_loop.run(move |event, event_loop_window_target| {
        event_loop_window_target.set_control_flow(ControlFlow::Poll);

        let new_time = Instant::now();
        let frame_time = (new_time - current_time).as_secs_f32();

        current_time = new_time;

        match event {
            Event::WindowEvent {
                window_id,
                ref event,
            } if window_id == app_base.window_id() => match event {
                WindowEvent::CloseRequested => {
                    event_loop_window_target.exit();
                }
                _ => (),
            },
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {}
            _ => (),
        }

        if event_loop_window_target.exiting() {
            app_base.wait_after_single_render_loop(event_loop_window_target);
        }

        app_base.wait_after_single_render_loop(event_loop_window_target);
    });

    match result {
        Ok(_) => Ok(()),
        Err(err) => Err(err.into()),
    }
}
