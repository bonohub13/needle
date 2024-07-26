use crate::{device::Device, swapchain::Swapchain, window::Window};
use anyhow::{bail, Result};
use ash::vk;
use winit::event_loop::ControlFlow;

pub struct Renderer {
    swapchain: Box<Swapchain>,
    command_buffers: Vec<vk::CommandBuffer>,
    current_image_index: usize,
    current_frame_index: usize,
    frame_started: bool,
}

impl Renderer {
    pub fn new(window: &Window, device: &Device) -> Result<Self> {
        let swapchain = Self::recreate_swapchain(window, device, None, None)?;
        let command_buffers = Self::create_command_buffers(device)?;

        Ok(Self {
            swapchain,
            command_buffers,
            current_frame_index: 0,
            current_image_index: 0,
            frame_started: false,
        })
    }

    pub fn destroy(&mut self, device: &Device) {
        unsafe {
            device
                .device()
                .free_command_buffers(*device.command_pool(), &self.command_buffers);
        }
        self.command_buffers.clear();
        self.swapchain.destroy(device);
    }

    /* Private functions */

    fn recreate_swapchain(
        window: &Window,
        device: &Device,
        old_swapchain: Option<&Swapchain>,
        mut control_flow: Option<&mut ControlFlow>,
    ) -> Result<Box<Swapchain>> {
        let device_ref = device.device();
        let mut extent = window.extent()?;

        while extent.width == 0 || extent.height == 0 {
            extent = window.extent()?;

            if let Some(ref mut control_flow_mut_ref) = control_flow {
                **control_flow_mut_ref = ControlFlow::Wait;
            }
        }
        // Wait until current swapchain is out of use
        unsafe { device_ref.device_wait_idle() }?;

        let swapchain = if let Some(old_swapchain) = old_swapchain {
            let swapchain = Swapchain::create_from_previous_swapchain(
                device,
                extent,
                old_swapchain.swapchain(),
            )?;

            if !old_swapchain.compare_swap_formats(&swapchain) {
                bail!("Swapchain image or depth format has changed!");
            }

            swapchain
        } else {
            Swapchain::new(device, extent)?
        };

        Ok(Box::new(swapchain))
    }

    fn create_command_buffers(device: &Device) -> Result<Vec<vk::CommandBuffer>> {
        let allocate_info = vk::CommandBufferAllocateInfo::default()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(*device.command_pool())
            .command_buffer_count(Swapchain::MAX_FRAMES_IN_FLIGHT as u32);
        let command_buffers = unsafe { device.device().allocate_command_buffers(&allocate_info) }?;

        Ok(command_buffers)
    }
}
