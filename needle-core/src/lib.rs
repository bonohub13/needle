mod app;
mod config;
mod error;
mod renderer;
mod texture;
mod time;

pub use app::*;
pub use config::*;
pub use error::*;
pub use renderer::*;
pub use texture::*;
pub use time::*;
pub use wgpu::{include_spirv_raw, include_wgsl};

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
    Shader(&'a str),
    Texture(&'a str),
}

impl NeedleLabel<'_> {
    pub fn as_str(&self) -> &str {
        stringify!("{}", self)
    }
}

impl<'a> Display for NeedleLabel<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let label = match self {
            Self::Device(label) => {
                if label.len() == 0 {
                    "Device"
                } else {
                    stringify!("{} Device", label)
                }
            }
            Self::PipelineLayout(label) => {
                if label.len() == 0 {
                    "Pipeline Layout"
                } else {
                    stringify!("{} Pipeline Layout", label)
                }
            }
            Self::Pipeline(label) => {
                if label.len() == 0 {
                    "Render Pipeline"
                } else {
                    stringify!("{} Pipeline", label)
                }
            }
            Self::CommandEncoder(label) => {
                if label.len() == 0 {
                    "Command Encoder"
                } else {
                    stringify!("{} Command Encoder", label)
                }
            }
            Self::RenderPass(label) => {
                if label.len() == 0 {
                    "Render Pass"
                } else {
                    stringify!("{} Render Pass", label)
                }
            }
            Self::Shader(label) => {
                if label.len() == 0 {
                    "Shader"
                } else {
                    stringify!("{} Shader", label)
                }
            }
            Self::Texture(label) => {
                if label.len() == 0 {
                    "Texture"
                } else {
                    stringify!("{} Texture", label)
                }
            }
        };

        write!(f, "{}", label)
    }
}
