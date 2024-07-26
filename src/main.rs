/* Hide console when running a release build under Windows */
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use needle_core;
use winit::event_loop::EventLoop;

const APP_NAME: &'static str = "needle";

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    let app_info = needle_core::AppInfo::new("needle", 1);
    let app_base = needle_core::AppBase::new(&event_loop, &app_info);

    Ok(())
}
