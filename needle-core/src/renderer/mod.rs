mod shader;
mod text;

pub use shader::*;
pub use text::*;

use crate::error::NeedleErr;
use wgpu::{Device, Queue, RenderPass, SurfaceConfiguration};
use winit::dpi::PhysicalSize;

pub trait Renderer {
    fn resize(&mut self, size: &PhysicalSize<u32>);
    fn update(&mut self, queue: &Queue, config: &SurfaceConfiguration);
    fn prepare(&mut self, margin: f32, device: &Device, queue: &Queue) -> NeedleErr<()>;
    fn render(&mut self, render_pass: &mut RenderPass) -> NeedleErr<()>;
}
