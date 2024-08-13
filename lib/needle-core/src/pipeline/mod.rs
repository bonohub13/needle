use crate::{device::Device, utils::is_debug_build};
use anyhow::{Context, Result};
use ash::{util::read_spv, vk};
use std::{any::type_name, ffi::CStr, fs::File};

pub mod config;
pub mod vertex;

pub struct Pipeline {
    graphics_pipeline: vk::Pipeline,
    vert_shader_module: vk::ShaderModule,
    frag_shader_module: vk::ShaderModule,
}

impl Pipeline {
    pub fn new(
        device: &Device,
        vert_shader_path: &str,
        frag_shader_path: &str,
        config_info: &config::PipelineConfigInfo,
    ) -> Result<Self> {
        let frag_shader_module = {
            let mut frag_code = Self::read_file(frag_shader_path)?;

            Self::create_shader_module(device, &mut frag_code)
        }?;
        let vert_shader_module = {
            let mut vert_code = Self::read_file(vert_shader_path)?;

            Self::create_shader_module(device, &mut vert_code)
        }?;
        let graphics_pipeline = Self::create_graphics_pipeline(
            device,
            &vert_shader_module,
            &frag_shader_module,
            config_info,
        )?;

        Ok(Self {
            graphics_pipeline,
            vert_shader_module,
            frag_shader_module,
        })
    }

    pub fn default_pipeline_config_info<'a>() -> config::PipelineConfigInfo<'a> {
        config::PipelineConfigInfo {
            binding_descriptions: vertex::Vertex::binding_descriptions(),
            attribute_descriptions: vertex::Vertex::attribute_descriptions(),
            viewport_info: vk::PipelineViewportStateCreateInfo::default()
                .viewports(&[])
                .scissors(&[]),
            input_assembly_info: vk::PipelineInputAssemblyStateCreateInfo::default()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
                .primitive_restart_enable(false),
            rasterization_info: vk::PipelineRasterizationStateCreateInfo::default()
                .depth_clamp_enable(false)
                .rasterizer_discard_enable(false)
                .polygon_mode(vk::PolygonMode::FILL)
                .line_width(1.0f32)
                .cull_mode(vk::CullModeFlags::NONE)
                .front_face(vk::FrontFace::CLOCKWISE)
                .depth_bias_enable(false)
                .depth_bias_constant_factor(0.0)
                .depth_bias_clamp(0.0)
                .depth_bias_slope_factor(0.0),
            multisample_info: vk::PipelineMultisampleStateCreateInfo::default()
                .sample_shading_enable(false)
                .rasterization_samples(vk::SampleCountFlags::TYPE_1)
                .min_sample_shading(1.0)
                .sample_mask(&[])
                .alpha_to_coverage_enable(false)
                .alpha_to_one_enable(false),
            color_blend_attachment: vk::PipelineColorBlendAttachmentState::default()
                .color_write_mask(vk::ColorComponentFlags::RGBA)
                .blend_enable(false)
                .src_color_blend_factor(vk::BlendFactor::ONE)
                .dst_color_blend_factor(vk::BlendFactor::ZERO)
                .color_blend_op(vk::BlendOp::ADD)
                .src_alpha_blend_factor(vk::BlendFactor::ONE)
                .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
                .alpha_blend_op(vk::BlendOp::ADD),
            depth_stencil_info: vk::PipelineDepthStencilStateCreateInfo::default()
                .depth_test_enable(true)
                .depth_write_enable(true)
                .depth_compare_op(vk::CompareOp::LESS)
                .depth_bounds_test_enable(false)
                .min_depth_bounds(0.0)
                .max_depth_bounds(1.0)
                .stencil_test_enable(false),
            dynamic_state_enables: vec![
                vk::DynamicState::VIEWPORT_WITH_COUNT,
                vk::DynamicState::SCISSOR_WITH_COUNT,
            ],
            pipeline_layout: vk::PipelineLayout::null(),
            render_pass: vk::RenderPass::null(),
            subpass: 0,
        }
    }

    pub fn enable_alpha_blending<'a>() -> config::PipelineConfigInfo<'a> {
        config::PipelineConfigInfo {
            binding_descriptions: vertex::Vertex::binding_descriptions(),
            attribute_descriptions: vertex::Vertex::attribute_descriptions(),
            viewport_info: vk::PipelineViewportStateCreateInfo::default()
                .viewports(&[])
                .scissors(&[]),
            input_assembly_info: vk::PipelineInputAssemblyStateCreateInfo::default()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
                .primitive_restart_enable(false),
            rasterization_info: vk::PipelineRasterizationStateCreateInfo::default()
                .depth_clamp_enable(false)
                .rasterizer_discard_enable(false)
                .polygon_mode(vk::PolygonMode::FILL)
                .line_width(1.0f32)
                .cull_mode(vk::CullModeFlags::NONE)
                .front_face(vk::FrontFace::CLOCKWISE)
                .depth_bias_enable(false)
                .depth_bias_constant_factor(0.0)
                .depth_bias_clamp(0.0)
                .depth_bias_slope_factor(0.0),
            multisample_info: vk::PipelineMultisampleStateCreateInfo::default()
                .sample_shading_enable(false)
                .rasterization_samples(vk::SampleCountFlags::TYPE_1)
                .min_sample_shading(1.0)
                .sample_mask(&[])
                .alpha_to_coverage_enable(false)
                .alpha_to_one_enable(false),
            color_blend_attachment: vk::PipelineColorBlendAttachmentState::default()
                .color_write_mask(vk::ColorComponentFlags::RGBA)
                .blend_enable(false)
                .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
                .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                .color_blend_op(vk::BlendOp::ADD)
                .src_alpha_blend_factor(vk::BlendFactor::ONE)
                .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
                .alpha_blend_op(vk::BlendOp::ADD),
            depth_stencil_info: vk::PipelineDepthStencilStateCreateInfo::default()
                .depth_test_enable(true)
                .depth_write_enable(true)
                .depth_compare_op(vk::CompareOp::LESS)
                .depth_bounds_test_enable(false)
                .min_depth_bounds(0.0)
                .max_depth_bounds(1.0)
                .stencil_test_enable(false),
            dynamic_state_enables: vec![
                vk::DynamicState::VIEWPORT_WITH_COUNT,
                vk::DynamicState::SCISSOR_WITH_COUNT,
            ],
            pipeline_layout: vk::PipelineLayout::null(),
            render_pass: vk::RenderPass::null(),
            subpass: 0,
        }
    }
    pub unsafe fn destroy(&mut self, device: &Device) {
        let device = device.device();

        if is_debug_build() {
            println!("Performing cleanup procedure for {}", type_name::<Self>());
        }

        device.destroy_shader_module(self.frag_shader_module, None);
        device.destroy_shader_module(self.vert_shader_module, None);
        device.destroy_pipeline(self.graphics_pipeline, None);

        if is_debug_build() {
            println!("Completed cleanup procedure for {}", type_name::<Self>());
        }
    }

    /* Private functions */
    fn create_graphics_pipeline(
        device: &Device,
        vert_shader_module: &vk::ShaderModule,
        frag_shader_module: &vk::ShaderModule,
        config_info: &config::PipelineConfigInfo,
    ) -> Result<vk::Pipeline> {
        assert!(
            config_info.pipeline_layout != vk::PipelineLayout::null(),
            "Cannot create graphics pipeline: No pipeline_layout provided in config_info"
        );
        assert!(
            config_info.render_pass != vk::RenderPass::null(),
            "Cannot create graphics pipeline: No render_pass provided in config_info"
        );

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(*vert_shader_module)
                .name(unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") }),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(*frag_shader_module)
                .name(unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") }),
        ];
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_attribute_descriptions(&config_info.attribute_descriptions)
            .vertex_binding_descriptions(&config_info.binding_descriptions);
        let color_blend_info = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(std::slice::from_ref(&config_info.color_blend_attachment))
            .blend_constants([0.0f32, 0.0f32, 0.0f32, 0.0f32]);
        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::default()
            .dynamic_states(&config_info.dynamic_state_enables)
            .flags(vk::PipelineDynamicStateCreateFlags::empty());
        let create_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&config_info.input_assembly_info)
            .viewport_state(&config_info.viewport_info)
            .rasterization_state(&config_info.rasterization_info)
            .multisample_state(&config_info.multisample_info)
            .color_blend_state(&color_blend_info)
            .depth_stencil_state(&config_info.depth_stencil_info)
            .dynamic_state(&dynamic_state_info)
            .layout(config_info.pipeline_layout)
            .render_pass(config_info.render_pass)
            .subpass(config_info.subpass)
            .base_pipeline_index(-1)
            .base_pipeline_handle(vk::Pipeline::null());
        let graphics_pipeline = match unsafe {
            device.device().create_graphics_pipelines(
                vk::PipelineCache::null(),
                std::slice::from_ref(&create_info),
                None,
            )
        } {
            Ok(pipeline) => Ok(pipeline),
            Err((_, err)) => Err(err),
        }?
        .into_iter()
        .next()
        .context("Failed to create graphics pipeline")?;

        Ok(graphics_pipeline)
    }

    fn create_shader_module(device: &Device, shader_code: &mut File) -> Result<vk::ShaderModule> {
        let spv_code = read_spv(shader_code)?;
        let create_info = vk::ShaderModuleCreateInfo::default().code(&spv_code);
        let shader_module = unsafe { device.device().create_shader_module(&create_info, None) }?;

        Ok(shader_module)
    }

    fn read_file(file_path: &str) -> Result<File> {
        Ok(File::open(file_path)?)
    }
}
