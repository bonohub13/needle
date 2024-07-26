use ash::vk;

pub struct PipelineConfigInfo<'a> {
    pub binding_descriptions: Vec<vk::VertexInputBindingDescription>,
    pub attribute_descriptions: Vec<vk::VertexInputAttributeDescription>,
    pub viewport_info: vk::PipelineViewportStateCreateInfo<'a>,
    pub input_assembly_info: vk::PipelineInputAssemblyStateCreateInfo<'a>,
    pub rasterization_info: vk::PipelineRasterizationStateCreateInfo<'a>,
    pub multisample_info: vk::PipelineMultisampleStateCreateInfo<'a>,
    pub color_blend_attachment: vk::PipelineColorBlendAttachmentState,
    pub depth_stencil_info: vk::PipelineDepthStencilStateCreateInfo<'a>,
    pub dynamic_state_enables: Vec<vk::DynamicState>,
    pub pipeline_layout: vk::PipelineLayout,
    pub render_pass: vk::RenderPass,
    pub subpass: u32,
}
