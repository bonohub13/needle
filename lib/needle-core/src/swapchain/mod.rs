use crate::device::Device;
use anyhow::{Context, Result};
use ash::{khr, vk};

pub struct Swapchain {
    image_format: vk::Format,
    depth_format: vk::Format,
    extent: vk::Extent2D,
    framebuffers: Vec<vk::Framebuffer>,
    render_pass: vk::RenderPass,
    depth_images: Vec<vk::Image>,
    depth_image_memories: Vec<vk::DeviceMemory>,
    depth_image_views: Vec<vk::ImageView>,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    window_extent: vk::Extent2D,
    device: khr::swapchain::Device,
    swapchain: vk::SwapchainKHR,
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    images_in_flight: Vec<vk::Fence>,
    current_frame: usize,
}

impl Swapchain {
    pub const MAX_FRAMES_IN_FLIGHT: i32 = 2;

    pub fn new(device: &Device, extent: vk::Extent2D) -> Result<Self> {
        Self::create_from_previous_swapchain(device, extent, &vk::SwapchainKHR::null())
    }

    pub fn null(device: &Device) -> Self {
        Self {
            image_format: vk::Format::default(),
            depth_format: vk::Format::default(),
            extent: vk::Extent2D::default(),
            framebuffers: vec![],
            render_pass: vk::RenderPass::null(),
            depth_images: vec![],
            depth_image_memories: vec![],
            depth_image_views: vec![],
            images: vec![],
            image_views: vec![],
            window_extent: vk::Extent2D::default(),
            device: device.swapchain_device(),
            swapchain: vk::SwapchainKHR::null(),
            image_available_semaphores: vec![],
            render_finished_semaphores: vec![],
            in_flight_fences: vec![],
            images_in_flight: vec![],
            current_frame: 0,
        }
    }

    pub fn create_from_previous_swapchain(
        device: &Device,
        window_extent: vk::Extent2D,
        previous_swapchain: &vk::SwapchainKHR,
    ) -> Result<Self> {
        let (swapchain_device, swapchain, images, image_format, extent) =
            Self::create_swapchain(device, &window_extent, previous_swapchain)?;
        let image_views = Self::create_image_views(device, &images, image_format)?;
        let (depth_images, depth_image_memories, depth_image_views, depth_format) =
            Self::create_depth_resources(device, extent, &images)?;
        let render_pass = Self::create_render_pass(device, image_format)?;
        let framebuffers = Self::create_framebuffers(
            device,
            &extent,
            &images,
            &image_views,
            &depth_image_views,
            &render_pass,
        )?;
        let (
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            images_in_flight,
        ) = Self::create_sync_objects(device, &images)?;

        Ok(Self {
            image_format,
            depth_format,
            extent,
            framebuffers,
            render_pass,
            depth_images,
            depth_image_memories,
            depth_image_views,
            images,
            image_views,
            window_extent,
            device: swapchain_device,
            swapchain,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            images_in_flight,
            current_frame: 0,
        })
    }

    pub fn destroy(&mut self, device: &Device) {
        let device = device.device();

        self.image_views
            .iter()
            .for_each(|image_view| unsafe { device.destroy_image_view(*image_view, None) });
        self.image_views.clear();

        if self.swapchain != vk::SwapchainKHR::null() {
            unsafe {
                self.device.destroy_swapchain(self.swapchain, None);
            }
        }

        self.depth_images
            .iter()
            .enumerate()
            .for_each(|(index, depth_image)| unsafe {
                device.destroy_image_view(self.depth_image_views[index], None);
                device.destroy_image(*depth_image, None);
                device.free_memory(self.depth_image_memories[index], None);
            });

        self.framebuffers
            .iter()
            .for_each(|framebuffer| unsafe { device.destroy_framebuffer(*framebuffer, None) });

        unsafe { device.destroy_render_pass(self.render_pass, None) };

        for index in 0..Self::MAX_FRAMES_IN_FLIGHT as usize {
            unsafe {
                device.destroy_fence(self.in_flight_fences[index], None);
                device.destroy_semaphore(self.render_finished_semaphores[index], None);
                device.destroy_semaphore(self.image_available_semaphores[index], None);
            }
        }
    }

    #[inline]
    pub fn swapchain(&self) -> &vk::SwapchainKHR {
        &self.swapchain
    }

    #[inline]
    pub fn find_depth_format(&self, device: &Device) -> Result<vk::Format> {
        Self::find_depth_format_from_device(device)
    }

    pub fn compare_swap_formats(&self, swapchain: &Self) -> bool {
        self.image_format == swapchain.image_format && self.depth_format == swapchain.depth_format
    }

    /* Private functions */

    fn create_swapchain(
        device: &Device,
        window_extent: &vk::Extent2D,
        previous_swapchain: &vk::SwapchainKHR,
    ) -> Result<(
        khr::swapchain::Device,
        vk::SwapchainKHR,
        Vec<vk::Image>,
        vk::Format,
        vk::Extent2D,
    )> {
        let swapchain_support = unsafe { device.swapchain_support() }?;
        let surface_format = Self::choose_swap_surface_format(&swapchain_support.formats)?;
        let surface_present_mode = Self::choose_swap_present_mode(&swapchain_support.present_modes);
        let surface_extent =
            Self::choose_swap_extent(window_extent, &swapchain_support.capabilities);
        let image_count = if swapchain_support.capabilities.max_image_count > 0
            && (swapchain_support.capabilities.min_image_count + 1)
                > swapchain_support.capabilities.max_image_count
        {
            swapchain_support.capabilities.max_image_count
        } else {
            swapchain_support.capabilities.min_image_count + 1
        };
        let indices = device.find_physical_queue_families()?;
        let queue_family_indices = [
            indices
                .graphics_family
                .context("Graphics queue family is not available")?,
            indices
                .present_family
                .context("Present queue family is not available")?,
        ];
        let queue_family_matches = queue_family_indices[0] == queue_family_indices[1];
        let create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(*device.surface().surface())
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(surface_extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(if queue_family_matches {
                vk::SharingMode::EXCLUSIVE
            } else {
                vk::SharingMode::CONCURRENT
            })
            .queue_family_indices(if queue_family_matches {
                &[]
            } else {
                &queue_family_indices
            })
            .pre_transform(swapchain_support.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(surface_present_mode)
            .clipped(true)
            .old_swapchain(if *previous_swapchain != vk::SwapchainKHR::null() {
                *previous_swapchain
            } else {
                vk::SwapchainKHR::null()
            });
        let device = device.swapchain_device();
        let swapchain = unsafe { device.create_swapchain(&create_info, None) }?;
        let images = unsafe { device.get_swapchain_images(swapchain) }?;

        Ok((
            device,
            swapchain,
            images,
            surface_format.format,
            surface_extent,
        ))
    }

    fn create_image_views(
        device: &Device,
        images: &[vk::Image],
        image_format: vk::Format,
    ) -> Result<Vec<vk::ImageView>> {
        let mut image_views = vec![vk::ImageView::null(); images.len()];

        for (index, image) in images.iter().enumerate() {
            let create_info = vk::ImageViewCreateInfo::default()
                .image(*image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(image_format)
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1),
                );

            image_views[index] = unsafe { device.device().create_image_view(&create_info, None) }?;
        }

        Ok(image_views)
    }

    fn create_render_pass(device: &Device, image_format: vk::Format) -> Result<vk::RenderPass> {
        let attachments = [
            vk::AttachmentDescription::default()
                .format(image_format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::PRESENT_SRC_KHR),
            vk::AttachmentDescription::default()
                .format(Self::find_depth_format_from_device(device)?)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
        ];
        let color_attachment = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        let depth_stencil_attachment = vk::AttachmentReference::default()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_attachment))
            .depth_stencil_attachment(&depth_stencil_attachment);
        let dependency = vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .src_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .src_access_mask(vk::AccessFlags::empty())
            .dst_subpass(0)
            .dst_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .dst_access_mask(
                vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            );
        let create_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachments)
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(std::slice::from_ref(&dependency));

        Ok(unsafe { device.device().create_render_pass(&create_info, None) }?)
    }

    fn create_depth_resources(
        device: &Device,
        extent: vk::Extent2D,
        images: &[vk::Image],
    ) -> Result<(
        Vec<vk::Image>,
        Vec<vk::DeviceMemory>,
        Vec<vk::ImageView>,
        vk::Format,
    )> {
        let depth_format = Self::find_depth_format_from_device(device)?;
        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .format(depth_format)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .samples(vk::SampleCountFlags::TYPE_1)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let image_count = images.len();
        let mut depth_images = vec![vk::Image::null(); image_count];
        let mut depth_image_memories = vec![vk::DeviceMemory::null(); image_count];
        let mut depth_image_views = vec![vk::ImageView::null(); image_count];

        for index in 0..image_count {
            (depth_images[index], depth_image_memories[index]) = device
                .create_image_with_info(&image_info, vk::MemoryPropertyFlags::DEVICE_LOCAL)?;
            depth_image_views[index] = {
                let create_info = vk::ImageViewCreateInfo::default()
                    .image(depth_images[index])
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(depth_format)
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::DEPTH,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    });

                unsafe { device.device().create_image_view(&create_info, None) }?
            };
        }

        Ok((
            depth_images,
            depth_image_memories,
            depth_image_views,
            depth_format,
        ))
    }

    fn create_framebuffers(
        device: &Device,
        extent: &vk::Extent2D,
        images: &[vk::Image],
        image_views: &[vk::ImageView],
        depth_image_views: &[vk::ImageView],
        render_pass: &vk::RenderPass,
    ) -> Result<Vec<vk::Framebuffer>> {
        let image_count = images.len();
        let mut framebuffers = vec![vk::Framebuffer::null(); image_count];

        for index in 0..image_count {
            let attachments = [image_views[index], depth_image_views[index]];
            let create_info = vk::FramebufferCreateInfo::default()
                .render_pass(*render_pass)
                .attachments(&attachments)
                .width(extent.width)
                .height(extent.height)
                .layers(1);

            framebuffers[index] =
                unsafe { device.device().create_framebuffer(&create_info, None) }?;
        }

        Ok(framebuffers)
    }

    fn create_sync_objects(
        device: &Device,
        images: &[vk::Image],
    ) -> Result<(
        Vec<vk::Semaphore>,
        Vec<vk::Semaphore>,
        Vec<vk::Fence>,
        Vec<vk::Fence>,
    )> {
        let image_count = images.len();
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
        let mut image_available_semaphores =
            vec![vk::Semaphore::null(); Self::MAX_FRAMES_IN_FLIGHT as usize];
        let mut render_finished_semaphores =
            vec![vk::Semaphore::null(); Self::MAX_FRAMES_IN_FLIGHT as usize];

        let mut in_flight_fences = vec![vk::Fence::null(); Self::MAX_FRAMES_IN_FLIGHT as usize];

        let mut images_in_flight = vec![vk::Fence::null(); image_count];

        for index in 0..Self::MAX_FRAMES_IN_FLIGHT as usize {
            image_available_semaphores[index] =
                unsafe { device.device().create_semaphore(&semaphore_info, None) }?;
            render_finished_semaphores[index] =
                unsafe { device.device().create_semaphore(&semaphore_info, None) }?;
            in_flight_fences[index] = unsafe { device.device().create_fence(&fence_info, None) }?;
        }

        Ok((
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            images_in_flight,
        ))
    }

    fn choose_swap_surface_format(
        available_formats: &[vk::SurfaceFormatKHR],
    ) -> Result<vk::SurfaceFormatKHR> {
        let target = vk::SurfaceFormatKHR {
            format: vk::Format::R8G8B8A8_SRGB,
            color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
        };

        if available_formats.contains(&target) {
            Ok(target)
        } else {
            Ok(*available_formats
                .iter()
                .next()
                .context("No format was availble")?)
        }
    }

    fn choose_swap_present_mode(
        available_present_modes: &[vk::PresentModeKHR],
    ) -> vk::PresentModeKHR {
        if available_present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
            println!("Present mode: Mailbox");

            vk::PresentModeKHR::MAILBOX
        } else {
            println!("Present mode: V-Sync");

            vk::PresentModeKHR::FIFO
        }
    }

    fn choose_swap_extent(
        window_extent: &vk::Extent2D,
        capabilities: &vk::SurfaceCapabilitiesKHR,
    ) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D::default()
                .width(
                    capabilities
                        .min_image_extent
                        .width
                        .max(capabilities.max_image_extent.width.min(window_extent.width)),
                )
                .height(
                    capabilities.min_image_extent.height.max(
                        capabilities
                            .max_image_extent
                            .height
                            .min(window_extent.height),
                    ),
                )
        }
    }

    #[inline]
    fn find_depth_format_from_device(device: &Device) -> Result<vk::Format> {
        device.find_supported_format(
            &[
                vk::Format::D32_SFLOAT,
                vk::Format::D32_SFLOAT_S8_UINT,
                vk::Format::D24_UNORM_S8_UINT,
            ],
            vk::ImageTiling::OPTIMAL,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        )
    }
}
