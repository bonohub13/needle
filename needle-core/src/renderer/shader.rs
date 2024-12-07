use crate::NeedleLabel;
use anyhow::Result;
use std::{fs::OpenOptions, io::Read};
use wgpu::{RenderPipeline, ShaderModule};

pub struct ShaderRenderer {
    vert_shader: ShaderModule,
    frag_shader: ShaderModule,
    vert_shader_buffer: Box<[u8]>,
    frag_shader_buffer: Box<[u8]>,
    pipeline: RenderPipeline,
}

impl ShaderRenderer {
    pub fn new(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
        vert_shader_path: &str,
        frag_shader_path: &str,
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) -> Result<Self> {
        let vert_shader_buffer = Self::read_shader(vert_shader_path)?;
        let frag_shader_buffer = Self::read_shader(frag_shader_path)?;
        let vert_shader = unsafe {
            device.create_shader_module_spirv(&wgpu::ShaderModuleDescriptorSpirV {
                label: Some(NeedleLabel::Shader("Vertex").as_str()),
                source: wgpu::util::make_spirv_raw(&vert_shader_buffer),
            })
        };
        let frag_shader = unsafe {
            device.create_shader_module_spirv(&wgpu::ShaderModuleDescriptorSpirV {
                label: Some(NeedleLabel::Shader("Fragment").as_str()),
                source: wgpu::util::make_spirv_raw(&frag_shader_buffer),
            })
        };
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(NeedleLabel::PipelineLayout("Render").as_str()),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(NeedleLabel::Pipeline("Render").as_str()),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vert_shader,
                entry_point: Some("main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &frag_shader,
                entry_point: Some("main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Ok(Self {
            vert_shader_buffer,
            frag_shader_buffer,
            vert_shader,
            frag_shader,
            pipeline: render_pipeline,
        })
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.draw(0..3, 0..1);
    }

    #[inline]
    pub const fn pipeline(&self) -> &RenderPipeline {
        &self.pipeline
    }

    fn read_shader(path: &str) -> Result<Box<[u8]>> {
        let mut reader = OpenOptions::new().read(true).open(path)?;
        let mut buffer = vec![];

        reader.read_to_end(&mut buffer)?;
        if (buffer.len() & 4) != 0 {
            for _ in 0..(buffer.len() % 4) {
                buffer.push(0);
            }
        }

        let buffer = Box::from_iter(buffer);

        Ok(buffer)
    }
}
