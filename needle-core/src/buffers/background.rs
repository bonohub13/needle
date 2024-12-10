use crate::NeedleLabel;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BackgroundUniform {
    size: [f32; 2],
    color: [f32; 4],
}

impl BackgroundUniform {
    pub fn new(size: [f32; 2], color: [f32; 4]) -> Self {
        Self { size, color }
    }

    pub fn resize(&mut self, size: [f32; 2]) {
        self.size = size;
    }

    pub fn set_color(&mut self, color: [f32; 4]) {
        self.color = color;
    }

    pub fn create_uniform_buffer(
        &self,
        device: &wgpu::Device,
        label: Option<NeedleLabel>,
    ) -> wgpu::Buffer {
        let label = match label {
            Some(label) => label.to_string(),
            None => NeedleLabel::UniformBuffer("Background").to_string(),
        };

        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&label),
            contents: bytemuck::cast_slice(&[*self]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }
}
