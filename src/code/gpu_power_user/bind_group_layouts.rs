use bevy::{prelude::Component, render::render_resource::BindGroupLayout};

#[derive(Component)]
pub struct BindGroupLayouts(pub BindGroupLayout);
