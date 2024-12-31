use std::{collections::HashMap, time::SystemTime};

use bevy::prelude::Component;

use super::iteration_space::IterationSpace;

#[derive(Component)]
/// Will always use the static map if it is present, otherwise it will use the callback
pub struct MaxOutputVectorLengths {
    pub unique_id: usize,
    map: Vec<usize>,
    callback: Option<fn(iter_space: &IterationSpace) -> Vec<usize>>,
    pub uses_callback: bool,
}
impl Default for MaxOutputVectorLengths {
    fn default() -> Self {
        Self {
            unique_id: 0,
            map: Vec::default(),
            callback: None,
            uses_callback: false,
        }
    }
}

impl MaxOutputVectorLengths {
    pub fn new(map: Vec<usize>) -> Self {
        Self {
            // from timestamp
            unique_id: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as usize,
            map: map,
            callback: None,
            uses_callback: false,
        }
    }
    pub fn from_callback(
        initial_iteration_space: &IterationSpace,
        callback: fn(iter_space: &IterationSpace) -> Vec<usize>,
    ) -> Self {
        let mut map = callback(initial_iteration_space);
        Self {
            unique_id: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as usize,
            map: map,
            callback: Some(callback),
            uses_callback: true,
        }
    }
    pub fn update_with_callback(&mut self, iter_space: &IterationSpace) {
        if let Some(callback) = self.callback {
            self.map = callback(iter_space);
            self.unique_id += 1;
        }
    }
    pub fn get(&self, output_index: usize) -> usize {
        return self.map[output_index];
    }
}
