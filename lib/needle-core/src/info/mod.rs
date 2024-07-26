use crate::{ENGINE_NAME, ENGINE_VERSION};
use ash::vk;
use std::ffi::CStr;

#[derive(Debug, Clone)]
pub struct AppInfo {
    name: Box<CStr>,
    version: u32,
    engine_name: Box<CStr>,
    engine_version: u32,
    api_version: u32,
}

impl AppInfo {
    pub fn new(name: &str, version: u32) -> Self {
        Self::new_version_1_3(name, version)
    }

    pub fn new_with_api_version(name: &str, version: u32, api_version: u32) -> Self {
        let name = unsafe { CStr::from_ptr(name.as_ptr() as *const i8) };

        Self {
            name: name.into(),
            version,
            engine_name: ENGINE_NAME.into(),
            engine_version: ENGINE_VERSION,
            api_version: match api_version {
                vk::API_VERSION_1_0
                | vk::API_VERSION_1_1
                | vk::API_VERSION_1_2
                | vk::API_VERSION_1_3 => api_version,
                _ => {
                    println!(
                        "Specified Vulkan API version ({:?}) was invalid.",
                        api_version
                    );
                    println!("Default to API_VERSION_1_0 ({})", vk::API_VERSION_1_0);

                    vk::API_VERSION_1_0
                }
            },
        }
    }

    pub fn new_version_1_0(name: &str, version: u32) -> Self {
        Self::new_with_api_version(name, version, vk::API_VERSION_1_0)
    }

    pub fn new_version_1_1(name: &str, version: u32) -> Self {
        Self::new_with_api_version(name, version, vk::API_VERSION_1_1)
    }

    pub fn new_version_1_2(name: &str, version: u32) -> Self {
        Self::new_with_api_version(name, version, vk::API_VERSION_1_2)
    }

    pub fn new_version_1_3(name: &str, version: u32) -> Self {
        Self::new_with_api_version(name, version, vk::API_VERSION_1_3)
    }

    pub const fn name(&self) -> &CStr {
        &self.name
    }

    pub const fn version(&self) -> u32 {
        self.version
    }

    pub const fn engine_name(&self) -> &CStr {
        &self.engine_name
    }

    pub const fn engine_version(&self) -> u32 {
        self.engine_version
    }

    pub const fn api_version(&self) -> u32 {
        self.api_version
    }
}
