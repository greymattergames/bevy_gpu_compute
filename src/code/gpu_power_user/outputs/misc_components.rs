use std::collections::HashMap;

use bevy::prelude::Component;

#[derive(Component)]
pub struct OutputCountsFromGpu(pub HashMap<String, Option<usize>>);
