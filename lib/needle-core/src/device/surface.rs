use crate::window::Window;
use anyhow::Result;
use ash::{khr::surface, vk};
#[allow(deprecated)]
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

pub struct Surface {
    instance: surface::Instance,
    surface: vk::SurfaceKHR,
}

pub struct SwapchainSupportDetails {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl Surface {
    #[allow(deprecated)]
    pub fn new(entry: &ash::Entry, instance: &ash::Instance, window: &Window) -> Result<Self> {
        let raw_display_handle = window.window().raw_display_handle()?;
        let raw_window_handle = window.window().raw_window_handle()?;
        let surface = unsafe {
            ash_window::create_surface(entry, instance, raw_display_handle, raw_window_handle, None)
        }?;
        let instance = surface::Instance::new(entry, instance);

        Ok(Self { instance, surface })
    }

    pub unsafe fn destroy(&self) {
        self.instance.destroy_surface(self.surface, None);
    }

    #[inline]
    pub fn surface(&self) -> &vk::SurfaceKHR {
        &self.surface
    }

    #[inline]
    pub unsafe fn get_physical_device_surface_support(
        &self,
        physical_device: &vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> Result<bool> {
        Ok(self.instance.get_physical_device_surface_support(
            *physical_device,
            queue_family_index,
            self.surface,
        )?)
    }

    #[inline]
    pub unsafe fn get_physical_device_surface_formats(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> Result<Vec<vk::SurfaceFormatKHR>> {
        Ok(self
            .instance
            .get_physical_device_surface_formats(*physical_device, self.surface)?)
    }

    #[inline]
    pub unsafe fn get_physical_device_surface_capabilities(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> Result<vk::SurfaceCapabilitiesKHR> {
        Ok(self
            .instance
            .get_physical_device_surface_capabilities(*physical_device, self.surface)?)
    }

    #[inline]
    pub unsafe fn get_physical_device_surface_present_modes(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> Result<Vec<vk::PresentModeKHR>> {
        Ok(self
            .instance
            .get_physical_device_surface_present_modes(*physical_device, self.surface)?)
    }

    pub unsafe fn query_swapchain_support(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> Result<SwapchainSupportDetails> {
        Ok(SwapchainSupportDetails {
            capabilities: self.get_physical_device_surface_capabilities(physical_device)?,
            formats: self.get_physical_device_surface_formats(physical_device)?,
            present_modes: self.get_physical_device_surface_present_modes(physical_device)?,
        })
    }
}
