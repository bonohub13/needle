pub mod debug;
pub mod query_family;
pub mod surface;

use crate::{info::AppInfo, utils::is_debug_build, window::Window};
use anyhow::{bail, Context, Result};
use ash::{ext::debug_utils, khr::swapchain, vk};
#[allow(deprecated)]
use raw_window_handle::HasRawDisplayHandle;
use std::ffi::CStr;

pub struct Device {
    entry: ash::Entry,
    instance: ash::Instance,
    debug_messenger: debug::DebugUtilsMessenger,
    surface: surface::Surface,
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    command_pool: vk::CommandPool,
}

impl Device {
    const VALIDATION_LAYERS: [*const i8; 1] =
        [
            unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0") }
                .as_ptr(),
        ];
    const DEVICE_EXTENSIONS: [*const i8; 1] = [swapchain::NAME.as_ptr()];

    pub fn new(window: &Window, app_info: &AppInfo) -> anyhow::Result<Self> {
        let entry = unsafe { ash::Entry::load() }?;
        let instance = Self::create_instance(&entry, window, app_info)?;
        let debug_messenger = if is_debug_build() {
            debug::DebugUtilsMessenger::new(&entry, &instance)?
        } else {
            debug::DebugUtilsMessenger::null(&entry, &instance)
        };
        let surface = surface::Surface::new(&entry, &instance, window)?;
        let (_, physical_device) = Self::pick_physical_device(&instance, &surface)?;
        let (device, graphics_queue, present_queue) =
            Self::create_device(&instance, &surface, &physical_device)?;
        let command_pool =
            Self::create_command_pool(&instance, &surface, &physical_device, &device)?;

        Ok(Self {
            entry,
            instance,
            debug_messenger,
            surface,
            physical_device,
            device,
            graphics_queue,
            present_queue,
            command_pool,
        })
    }

    pub fn destroy(&self) {
        unsafe {
            self.device.destroy_command_pool(self.command_pool, None);
            self.device.destroy_device(None);
            self.surface.destroy();
            if is_debug_build() {
                self.debug_messenger.destroy();
            }
            self.instance.destroy_instance(None);
        }
    }

    #[inline]
    pub fn surface(&self) -> &surface::Surface {
        &self.surface
    }

    #[inline]
    pub fn device(&self) -> &ash::Device {
        &self.device
    }

    #[inline]
    pub fn command_pool(&self) -> &vk::CommandPool {
        &self.command_pool
    }

    #[inline]
    pub fn find_physical_queue_families(&self) -> Result<query_family::QueryFamilyIndices> {
        Self::find_queue_families(&self.instance, &self.surface, &self.physical_device)
    }

    pub fn find_supported_format(
        &self,
        candidates: &[vk::Format],
        tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags,
    ) -> Result<vk::Format> {
        let format = candidates
            .iter()
            .filter(|format| {
                let properties = unsafe {
                    self.instance
                        .get_physical_device_format_properties(self.physical_device, **format)
                };

                match tiling {
                    vk::ImageTiling::LINEAR => {
                        (properties.linear_tiling_features & features) == features
                    }
                    vk::ImageTiling::OPTIMAL => {
                        (properties.optimal_tiling_features & features) == features
                    }
                    _ => false,
                }
            })
            .map(|format| *format)
            .next()
            .context("Failed to find supported format")?;

        Ok(format)
    }

    #[inline]
    pub fn swapchain_device(&self) -> swapchain::Device {
        swapchain::Device::new(&self.instance, &self.device)
    }

    #[inline]
    pub unsafe fn swapchain_support(&self) -> Result<surface::SwapchainSupportDetails> {
        self.surface.query_swapchain_support(&self.physical_device)
    }

    pub fn create_image_with_info(
        &self,
        create_info: &vk::ImageCreateInfo,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<(vk::Image, vk::DeviceMemory)> {
        let image = unsafe { self.device.create_image(create_info, None) }?;
        let mem_requirements = unsafe { self.device.get_image_memory_requirements(image) };
        let allocate_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(
                self.find_memory_type(mem_requirements.memory_type_bits, properties)?,
            );
        let image_memory = unsafe { self.device.allocate_memory(&allocate_info, None) }?;

        unsafe { self.device.bind_image_memory(image, image_memory, 0) }?;

        Ok((image, image_memory))
    }

    /* Private */
    fn create_instance(
        entry: &ash::Entry,
        window: &Window,
        app_info: &AppInfo,
    ) -> anyhow::Result<ash::Instance> {
        assert!(is_debug_build() && Self::check_validation_layer_support(entry)?);

        let app_info = vk::ApplicationInfo::default()
            .application_name(app_info.name())
            .application_version(app_info.version())
            .engine_name(app_info.engine_name())
            .engine_version(app_info.engine_version())
            .api_version(app_info.api_version());
        let extensions = Self::get_required_extensions(window)?;
        let layers = Self::VALIDATION_LAYERS.to_vec();
        let mut debug_create_info =
            debug::DebugUtilsMessenger::populate_debug_message_create_info();
        let create_info = if is_debug_build() {
            vk::InstanceCreateInfo::default()
                .application_info(&app_info)
                .enabled_extension_names(&extensions)
                .enabled_layer_names(&layers)
                .push_next(&mut debug_create_info)
        } else {
            vk::InstanceCreateInfo::default()
                .application_info(&app_info)
                .enabled_extension_names(&extensions)
                .enabled_layer_names(&layers)
        };
        let instance = unsafe { entry.create_instance(&create_info, None) }?;

        Self::has_required_instance_extensions(window, entry)?;

        Ok(instance)
    }

    fn pick_physical_device(
        instance: &ash::Instance,
        surface: &surface::Surface,
    ) -> Result<(vk::PhysicalDeviceProperties, vk::PhysicalDevice)> {
        let physical_devices = unsafe { instance.enumerate_physical_devices() }?;

        if physical_devices.is_empty() {
            bail!("Failed to find a suitable GPU");
        }

        println!("Device count: {}", physical_devices.len());
        let physical_device = {
            let mut physical_device = None;

            for device in physical_devices.iter() {
                if Self::is_device_suitable(instance, surface, device)? {
                    physical_device = Some(*device);

                    break;
                }
            }

            physical_device
        }
        .context("Failed to find a suitable GPU")?;
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };

        println!("physical device: {:?}", unsafe {
            CStr::from_ptr(properties.device_name.as_ptr())
        });

        Ok((properties, physical_device))
    }

    fn create_device(
        instance: &ash::Instance,
        surface: &surface::Surface,
        physical_device: &vk::PhysicalDevice,
    ) -> Result<(ash::Device, vk::Queue, vk::Queue)> {
        let indices = Self::find_queue_families(instance, surface, physical_device)?;
        let queue_priority = 1.0f32;
        let queue_create_info = {
            indices
                .unique_queue_families()?
                .iter()
                .map(|queue_family| {
                    vk::DeviceQueueCreateInfo::default()
                        .queue_family_index(*queue_family)
                        .queue_priorities(std::slice::from_ref(&queue_priority))
                })
                .collect::<Vec<_>>()
        };
        let create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_info)
            .enabled_extension_names(&Self::DEVICE_EXTENSIONS);
        let device = unsafe { instance.create_device(*physical_device, &create_info, None) }?;
        let graphics_queue = unsafe {
            device.get_device_queue(
                indices
                    .graphics_family
                    .context("Failed to get graphics queue")?,
                0,
            )
        };
        let present_queue = unsafe {
            device.get_device_queue(
                indices
                    .present_family
                    .context("Failed to get present queue")?,
                0,
            )
        };

        Ok((device, graphics_queue, present_queue))
    }

    fn create_command_pool(
        instance: &ash::Instance,
        surface: &surface::Surface,
        physical_device: &vk::PhysicalDevice,
        device: &ash::Device,
    ) -> Result<vk::CommandPool> {
        let queue_family_indices = Self::find_queue_families(instance, surface, physical_device)?;
        let create_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(
                queue_family_indices
                    .graphics_family
                    .context("Failed to get graphics queue family")?,
            )
            .flags(
                vk::CommandPoolCreateFlags::TRANSIENT
                    | vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            );
        let command_pool = unsafe { device.create_command_pool(&create_info, None) }?;

        Ok(command_pool)
    }

    fn find_memory_type(
        &self,
        type_filter: u32,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<u32> {
        let mem_properties = unsafe {
            self.instance
                .get_physical_device_memory_properties(self.physical_device)
        };

        for i in 0..mem_properties.memory_type_count {
            if (type_filter & (1 << i)) != 0
                && (mem_properties.memory_types[i as usize].property_flags & properties)
                    == properties
            {
                return Ok(i);
            }
        }

        bail!("Failed to find suitable memory type");
    }

    /// Performs a check if all required validation layers are supported.
    /// Returns true only if all required validation layers are supported.
    fn check_validation_layer_support(entry: &ash::Entry) -> anyhow::Result<bool> {
        println!("Requested validation layers:");
        let validation_layers = Self::VALIDATION_LAYERS
            .iter()
            .map(|layer| {
                let layer_name = unsafe { CStr::from_ptr(*layer) };

                println!("\t{:?}", layer_name);

                layer_name
            })
            .collect::<Vec<_>>();

        println!("Available layers:");
        let layers_found = unsafe { entry.enumerate_instance_layer_properties() }?
            .iter()
            .filter(|layer| {
                let layer_name = unsafe { CStr::from_ptr((*layer).layer_name.as_ptr()) };

                println!("\t{:?}", layer_name);

                validation_layers.contains(&layer_name)
            })
            .count();

        Ok(layers_found == validation_layers.len())
    }

    #[allow(deprecated)]
    fn get_required_extensions(window: &Window) -> anyhow::Result<Vec<*const i8>> {
        let mut extensions =
            ash_window::enumerate_required_extensions(window.window().raw_display_handle()?)?
                .to_vec();

        if is_debug_build() {
            extensions.push(debug_utils::NAME.as_ptr());
        }

        Ok(extensions)
    }

    fn has_required_instance_extensions(window: &Window, entry: &ash::Entry) -> Result<()> {
        println!("Available extensions:");
        let available = unsafe { entry.enumerate_instance_extension_properties(None) }?
            .iter()
            .map(|extension| {
                let extension_name = unsafe { CStr::from_ptr(extension.extension_name.as_ptr()) };

                println!("\t{:?}", extension_name);

                extension_name
            })
            .collect::<Vec<_>>();

        println!("Required extensions:");
        let required_extensions = Self::get_required_extensions(window)?;
        let contained_required_extensions = required_extensions
            .iter()
            .filter(|extension| {
                let extension_name = unsafe { CStr::from_ptr(**extension) };

                println!("\t{:?}", extension_name);

                available.contains(&extension_name)
            })
            .count();

        if contained_required_extensions != required_extensions.len() {
            bail!("Missing required extension")
        }

        Ok(())
    }

    fn is_device_suitable(
        instance: &ash::Instance,
        surface: &surface::Surface,
        physical_device: &vk::PhysicalDevice,
    ) -> Result<bool> {
        let indices = Self::find_queue_families(instance, surface, physical_device)?;
        let extensions_supported = Self::check_device_extension_support(instance, physical_device)?;
        let swapchain_adequate = if extensions_supported {
            let swapchain_support_details =
                unsafe { surface.query_swapchain_support(physical_device) }?;

            !swapchain_support_details.formats.is_empty()
                && !swapchain_support_details.present_modes.is_empty()
        } else {
            false
        };
        let supported_features = unsafe { instance.get_physical_device_features(*physical_device) };

        Ok(indices.is_complete()
            && extensions_supported
            && swapchain_adequate
            && supported_features.sampler_anisotropy != 0)
    }

    pub fn find_queue_families(
        instance: &ash::Instance,
        surface: &surface::Surface,
        physical_device: &vk::PhysicalDevice,
    ) -> Result<query_family::QueryFamilyIndices> {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(*physical_device) };
        let mut indices = query_family::QueryFamilyIndices::none();

        for (idx, queue_family) in queue_families.iter().enumerate() {
            let present_support = unsafe {
                surface.get_physical_device_surface_support(physical_device, idx as u32)?
            };

            if queue_family.queue_count > 0 {
                if (queue_family.queue_flags & vk::QueueFlags::GRAPHICS) == vk::QueueFlags::GRAPHICS
                {
                    indices.graphics_family = Some(idx as u32);
                }
                if present_support {
                    indices.present_family = Some(idx as u32);
                }
            }

            if indices.is_complete() {
                break;
            }
        }

        Ok(indices)
    }

    fn check_device_extension_support(
        instance: &ash::Instance,
        physical_device: &vk::PhysicalDevice,
    ) -> Result<bool> {
        let required_extensions = Self::DEVICE_EXTENSIONS
            .iter()
            .map(|extension| unsafe { CStr::from_ptr(*extension) })
            .collect::<Vec<_>>();
        let required_extensions_available =
            unsafe { instance.enumerate_device_extension_properties(*physical_device) }?
                .iter()
                .filter(|extension| {
                    let extension_name =
                        unsafe { CStr::from_ptr(extension.extension_name.as_ptr()) };

                    required_extensions.contains(&extension_name)
                })
                .count();

        Ok(required_extensions_available == required_extensions.len())
    }
}
