mod app;
mod error;
mod renderer;
mod time;

pub use app::*;
pub use error::*;
pub use renderer::*;
pub use time::*;
pub use wgpu::{include_spirv_raw, include_wgsl};

use std::fmt::{Display, Formatter, Result};

#[allow(dead_code)]
#[derive(Debug)]
enum NeedleLabel<'a> {
    Device(&'a str),
    PipelineLayout(&'a str),
    RenderPipeline(&'a str),
    CommandEncoder(&'a str),
    RenderPass(&'a str),
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
            Self::RenderPipeline(label) => {
                if label.len() == 0 {
                    "Render Pipeline"
                } else {
                    stringify!("{} Render Pipeline", label)
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
        };

        write!(f, "{}", label)
    }
}
