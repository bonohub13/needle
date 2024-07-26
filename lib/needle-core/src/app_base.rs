use crate::{device::Device, info::AppInfo, renderer::Renderer, window::Window};
use anyhow::Result;
use winit::event_loop::EventLoop;

pub struct AppBase {
    app_info: AppInfo,
    window: Window,
    device: Device,
    renderer: Renderer,
}

impl AppBase {
    pub fn new<T>(event_loop: &EventLoop<T>, app_info: &AppInfo) -> Result<Self> {
        let window = Window::new(event_loop, 1920, 1080, &app_info)?;
        let device = Device::new(&window, app_info)?;
        let renderer = Renderer::new(&window, &device)?;

        Ok(Self {
            app_info: app_info.clone(),
            window,
            device,
            renderer,
        })
    }
}

impl Drop for AppBase {
    fn drop(&mut self) {
        self.renderer.destroy(&self.device);
        self.device.destroy();
    }
}
