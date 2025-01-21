use bevy::prelude::Component;

#[derive(Default, Component)]
pub struct GpuOutputCounts(pub Vec<Option<usize>>);
