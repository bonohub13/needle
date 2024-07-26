use anyhow::{Context, Result};
use ash::vk;
use std::collections::HashSet;

pub struct QueryFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

impl QueryFamilyIndices {
    pub fn none() -> Self {
        Self {
            graphics_family: None,
            present_family: None,
        }
    }

    pub fn unique_queue_families(&self) -> Result<HashSet<u32>> {
        Ok(HashSet::from([
            self.graphics_family
                .context("Graphics queue family missing")?,
            self.present_family
                .context("Present queue family missing")?,
        ]))
    }

    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}
