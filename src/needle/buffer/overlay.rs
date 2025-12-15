// Copyright 2025 Kensuke Saito
// SPDX-License-Identifier: MIT

use glm::Vec4;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Overlay {
    position: Vec4,
    size: Vec4,
    color: Vec4,
}

impl Overlay {
    pub const fn new(position: &[f32; 2], size: &[f32; 2], color: &[f32; 4]) -> Self {
        Self {
            position: glm::vec4(position[0], position[1], 0.0, 0.0),
            size: glm::vec4(size[0], size[1], 0.0, 0.0),
            color: glm::vec4(color[0], color[1], color[2], color[3]),
        }
    }
}
