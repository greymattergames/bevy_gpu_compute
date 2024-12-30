use std::collections::HashMap;

use bevy::{prelude::Component, render::render_resource::Buffer};

#[derive(Component)]
pub struct InputBuffers(pub HashMap<String, Buffer>);
#[derive(Component)]
pub struct OutputBuffers(pub HashMap<String, Buffer>);

#[derive(Component)]
pub struct OutputStagingBuffers(pub HashMap<String, Buffer>);

#[derive(Component)]
pub struct OutputCountBuffers(pub HashMap<String, Buffer>);

#[derive(Component)]
pub struct OutputCountStagingBuffers(pub HashMap<String, Buffer>);
