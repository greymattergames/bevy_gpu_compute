use bevy::prelude::Component;

#[derive(Component)]
pub struct PipelineLayoutComponent(pub wgpu::PipelineLayout);
