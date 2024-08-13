use crate::{device::Device, info::AppInfo, renderer::Renderer, window::Window};
use anyhow::Result;
use winit::{
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::WindowId,
};

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

    pub fn window_id(&self) -> WindowId {
        self.window.window().id()
    }

    pub fn wait_after_single_render_loop<T>(&self, event_loop: &EventLoopWindowTarget<T>) -> () {
        match unsafe { self.device.device().device_wait_idle() } {
            Ok(_) => (),
            Err(err) => {
                eprintln!("Device failed to wait idle!");
                eprintln!("\tError: {:?}", err);

                event_loop.exit();
            }
        }
    }
}

impl Drop for AppBase {
    fn drop(&mut self) {
        self.renderer.destroy(&self.device);
        self.device.destroy();
    }
}
