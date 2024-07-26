use crate::info::AppInfo;
use ash::vk;
use winit::{dpi::LogicalSize, event_loop::EventLoop, window};

pub struct Window {
    window: window::Window,
    width: i32,
    height: i32,
    framebuffer_resized: bool,
}

impl Window {
    pub fn new<T>(
        event_loop: &EventLoop<T>,
        width: i32,
        height: i32,
        app_info: &AppInfo,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            window: Self::init_window(event_loop, width, height, app_info)?,
            width,
            height,
            framebuffer_resized: false,
        })
    }

    #[inline]
    pub const fn window(&self) -> &window::Window {
        &self.window
    }

    #[inline]
    pub const fn width(&self) -> i32 {
        self.width
    }

    #[inline]
    pub const fn height(&self) -> i32 {
        self.height
    }

    #[inline]
    pub const fn was_window_resized(&self) -> bool {
        self.framebuffer_resized
    }

    pub fn extent(&self) -> anyhow::Result<vk::Extent2D> {
        Ok(vk::Extent2D {
            width: self.width.try_into()?,
            height: self.height.try_into()?,
        })
    }

    #[inline]
    pub fn reset_window_resize_flag(&mut self) {
        self.framebuffer_resized = false;
    }

    #[inline]
    pub fn framebuffer_resized(&mut self, width: i32, height: i32) {
        self.framebuffer_resized = true;
        self.width = width;
        self.height = height;
    }

    /* Private */
    fn init_window<T>(
        event_loop: &EventLoop<T>,
        width: i32,
        height: i32,
        app_info: &AppInfo,
    ) -> anyhow::Result<window::Window> {
        let name = app_info.name().to_str()?;

        Ok(window::WindowBuilder::new()
            .with_resizable(false)
            .with_inner_size(LogicalSize::new(width, height))
            .with_title(name)
            .build(event_loop)?)
    }
}
