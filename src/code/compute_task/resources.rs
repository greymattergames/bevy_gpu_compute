use std::hash::{Hash, Hasher};

use bevy::{
    prelude::{Component, Resource},
    render::render_resource::{BindGroupLayout, Buffer},
};

// Resources to store reusable GPU state

#[derive(Default, Component)]
///Flag component
pub struct GpuAcceleratedBevy {}
