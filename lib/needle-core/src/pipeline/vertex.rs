use ash::vk;
use glm::{Vec2, Vec3};
use offset::offset_of;
use ordered_float::OrderedFloat;
use std::{
    hash::{Hash, Hasher},
    mem::size_of,
};

#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: Vec3,
    pub color: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

impl Vertex {
    pub fn new(position: &[f32; 3], color: &[f32; 3]) -> Self {
        Self {
            position: *Vec3::from_array(position),
            color: *Vec3::from_array(color),
            normal: *Vec3::from_array(&[0.0, 0.0, 0.0]),
            uv: *Vec2::from_array(&[0.0, 0.0]),
        }
    }

    pub fn binding_descriptions() -> Vec<vk::VertexInputBindingDescription> {
        vec![vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)]
    }

    pub fn attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription::default()
                .location(0)
                .binding(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Vertex::position).into()),
            vk::VertexInputAttributeDescription::default()
                .location(1)
                .binding(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Vertex::color).into()),
            vk::VertexInputAttributeDescription::default()
                .location(2)
                .binding(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Vertex::normal).into()),
            vk::VertexInputAttributeDescription::default()
                .location(3)
                .binding(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(Vertex::uv).into()),
        ]
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 0.0),
            normal: Vec3::new(0.0, 0.0, 0.0),
            color: Vec3::new(1.0, 1.0, 1.0),
            uv: Vec2::new(0.0, 0.0),
        }
    }
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
            && self.color == other.color
            && self.normal == other.normal
            && self.uv == other.uv
    }
}

impl Eq for Vertex {}

impl Hash for Vertex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.position
            .as_array()
            .iter()
            .for_each(|pos| OrderedFloat(*pos).hash(state));
        self.color
            .as_array()
            .iter()
            .for_each(|rgb| OrderedFloat(*rgb).hash(state));
        self.normal
            .as_array()
            .iter()
            .for_each(|normal| OrderedFloat(*normal).hash(state));
        self.uv
            .as_array()
            .iter()
            .for_each(|uv| OrderedFloat(*uv).hash(state));
    }
}
