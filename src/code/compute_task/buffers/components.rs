use std::collections::HashMap;

use bevy::{prelude::Component, render::render_resource::Buffer};

#[derive(Default, Component)]
pub struct InputBuffers(pub Vec<Buffer>);
#[derive(Default, Component)]
pub struct OutputBuffers(pub Vec<Buffer>);

#[derive(Default, Component)]
pub struct OutputStagingBuffers(pub Vec<Buffer>);

#[derive(Default, Component)]
pub struct OutputCountBuffers(pub Vec<Buffer>);

#[derive(Default, Component)]
pub struct OutputCountStagingBuffers(pub Vec<Buffer>);

#[derive(Default, Component)]
#[require(
    OutputBuffers,
    OutputCountBuffers,
    OutputStagingBuffers,
    OutputCountStagingBuffers,
    InputBuffers
)]
pub struct GpuAcceleratedBevyBuffers {}
