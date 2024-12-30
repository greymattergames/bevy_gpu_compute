use std::hash::{Hash, Hasher};

use bevy::{
    prelude::{Component, Resource},
    render::render_resource::{BindGroupLayout, Buffer},
};

// Resources to store reusable GPU state

#[derive(Component)]
pub struct PipelineLayout(pub wgpu::PipelineLayout);

#[derive(Component)]
///Flag component
pub struct GpuAcceleratedBevy {}

#[derive(Component)]
pub struct TaskLabel(pub String);
