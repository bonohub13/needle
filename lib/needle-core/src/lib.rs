use ash::vk;
use std::ffi::CStr;

pub mod app_base;
pub mod device;
pub mod info;
pub mod pipeline;
pub mod renderer;
pub mod swapchain;
pub mod window;

pub use crate::{app_base::AppBase, info::AppInfo};

pub mod utils {
    #[inline]
    pub fn is_debug_build() -> bool {
        cfg!(debug_assertions)
    }
}

const ENGINE_NAME: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"needle-core\0") };
const ENGINE_VERSION: u32 = vk::make_api_version(0, 0, 1, 0);
