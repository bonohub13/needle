mod base;
mod buffers;
mod config;
mod error;
mod renderer;
mod texture;
mod time;
mod utils;

pub use base::*;
pub use buffers::*;
pub use config::*;
pub use error::*;
pub use renderer::*;
pub use texture::*;
pub use time::*;

use std::fmt::{Display, Formatter, Result};

pub fn version_info() -> String {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");

    format!("{} {}", name, version)
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum NeedleLabel<'a> {
    Device(&'a str),
    PipelineLayout(&'a str),
    Pipeline(&'a str),
    CommandEncoder(&'a str),
    RenderPass(&'a str),
    Renderer(&'a str),
    Shader(&'a str),
    Texture(&'a str),
    VertexBuffer(&'a str),
    IndexBuffer(&'a str),
    UniformBuffer(&'a str),
    BindGroupLayout(&'a str),
    BindGroup(&'a str),
}

impl NeedleLabel<'_> {
    pub fn to_string(&self) -> String {
        format!("{}", self)
    }
}

impl<'a> Display for NeedleLabel<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let label = match self {
            Self::Device(label) => {
                if label.len() == 0 {
                    "Device".to_string()
                } else {
                    format!("{} Device", label)
                }
            }
            Self::PipelineLayout(label) => {
                if label.len() == 0 {
                    "Pipeline Layout".to_string()
                } else {
                    format!("{} Pipeline Layout", label)
                }
            }
            Self::Pipeline(label) => {
                if label.len() == 0 {
                    "Render Pipeline".to_string()
                } else {
                    format!("{} Pipeline", label)
                }
            }
            Self::CommandEncoder(label) => {
                if label.len() == 0 {
                    "Command Encoder".to_string()
                } else {
                    format!("{} Command Encoder", label)
                }
            }
            Self::RenderPass(label) => {
                if label.len() == 0 {
                    "Render Pass".to_string()
                } else {
                    format!("{} Render Pass", label)
                }
            }
            Self::Renderer(label) => {
                if label.len() == 0 {
                    "Renderer".to_string()
                } else {
                    format!("{} Renderer", label)
                }
            }
            Self::Shader(label) => {
                if label.len() == 0 {
                    "Shader".to_string()
                } else {
                    format!("{} Shader", label)
                }
            }
            Self::Texture(label) => {
                if label.len() == 0 {
                    "Texture".to_string()
                } else {
                    format!("{} Texture", label)
                }
            }
            Self::VertexBuffer(label) => {
                if label.len() == 0 {
                    "Vertex Buffer".to_string()
                } else {
                    format!("{} Vertex Buffer", label)
                }
            }
            Self::IndexBuffer(label) => {
                if label.len() == 0 {
                    "Index Buffer".to_string()
                } else {
                    format!("{} Index Buffer", label)
                }
            }
            Self::UniformBuffer(label) => {
                if label.len() == 0 {
                    "Uniform Buffer".to_string()
                } else {
                    format!("{} Uniform Buffer", label)
                }
            }
            Self::BindGroupLayout(label) => {
                if label.len() == 0 {
                    "Bind Group Layout".to_string()
                } else {
                    format!("{} Bind Group Layout", label)
                }
            }
            Self::BindGroup(label) => {
                if label.len() == 0 {
                    "Bind Group".to_string()
                } else {
                    format!("{} Bind Group", label)
                }
            }
        };

        write!(f, "{}", label)
    }
}
