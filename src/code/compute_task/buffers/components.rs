use std::collections::HashMap;

use bevy::{prelude::Component, render::render_resource::Buffer};

#[derive(Default, Component)]
pub struct InputBuffers(pub HashMap<String, Buffer>);
#[derive(Default, Component)]
pub struct OutputBuffers(pub HashMap<String, Buffer>);

#[derive(Default, Component)]
pub struct OutputStagingBuffers(pub HashMap<String, Buffer>);

#[derive(Default, Component)]
pub struct OutputCountBuffers(pub HashMap<String, Buffer>);

#[derive(Default, Component)]
pub struct OutputCountStagingBuffers(pub HashMap<String, Buffer>);

#[derive(Default, Component)]
#[require(
    OutputBuffers,
    OutputCountBuffers,
    OutputStagingBuffers,
    OutputCountStagingBuffers,
    InputBuffers
)]
pub struct GpuAcceleratedBevyBuffers {}
