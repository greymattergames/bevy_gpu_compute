use std::hash::{Hash, Hasher};

use bevy::prelude::Component;

#[derive(Component, Hash)]
/// tuple is organized in the order (x, y, z)
pub struct IterationSpace {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}
impl Default for IterationSpace {
    fn default() -> Self {
        IterationSpace { x: 1, y: 1, z: 1 }
    }
}

impl IterationSpace {
    pub fn num_dimmensions(&self) -> usize {
        if self.x > 1 && self.y > 1 && self.z > 1 {
            3
        } else if self.x > 1 && self.y > 1 && self.z == 1 {
            2
        } else {
            1
        }
    }
    pub fn get_hash(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}
