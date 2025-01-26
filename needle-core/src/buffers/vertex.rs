use crate::utils::crop;
use std::mem::size_of;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 4],
}

impl Vertex {
    const VERTEX_ATTR: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4];
    #[inline]
    pub fn new(position: [f32; 3], color: [f32; 4]) -> Self {
        Self { position, color }
    }

    pub fn rectangle(size: [f32; 2], offset: [f32; 2], depth: f32, color: &[f32; 4]) -> Vec<Self> {
        let (min_x, min_y) = (crop(offset[0], 1.0) - 1.0, crop(offset[1], 1.0) - 1.0);
        let vertices = vec![
            Vertex::new([min_x, min_y, depth], *color),     // Top left
            Vertex::new([size[0], min_y, depth], *color),   // Top Right
            Vertex::new([min_x, size[1], depth], *color),   // Bottom left
            Vertex::new([size[0], min_y, depth], *color),   // Top Right
            Vertex::new([size[0], size[1], depth], *color), // Bottom right
            Vertex::new([min_x, size[1], depth], *color),   // Bottom left
        ];

        vertices
    }

    pub fn indexed_rectangle(
        size: [f32; 2],
        offset: [f32; 2],
        depth: f32,
        color: &[f32; 4],
    ) -> (Vec<Self>, Box<[u16]>) {
        let (min_x, min_y) = (crop(offset[0], 2.0) - 1.0, crop(offset[1], 2.0) - 1.0);
        let vertices = vec![
            Vertex::new([min_x, min_y, depth], *color),     // Top left
            Vertex::new([size[0], min_y, depth], *color),   // Top Right
            Vertex::new([size[0], size[1], depth], *color), // Bottom right
            Vertex::new([min_x, size[1], depth], *color),   // Bottom left
        ];
        /* Order to draw
         * Top left
         * Top right
         * Bottom left
         * ---
         * Top right
         * Bottom right
         * Bottom left
         */
        let indices = [
            0, 1, 3, // Upper left triangle
            1, 2, 3, // Lower bottom triangle
        ];

        (vertices, indices.into())
    }

    pub const fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::VERTEX_ATTR,
        }
    }
}
