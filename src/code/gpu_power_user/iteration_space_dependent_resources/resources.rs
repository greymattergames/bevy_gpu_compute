use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    time::SystemTime,
};

use bevy::prelude::{Component, Resource};

#[derive(Resource, Hash)]
/// tuple is organized in the order (x, y, z)
pub struct IterationSpace {
    pub x: usize,
    pub y: usize,
    pub z: usize,
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

#[derive(Resource)]
/// Defaults should generally not be altered. Based on this resource: https://developer.arm.com/documentation/101897/0303/Compute-shading/Workgroup-sizes
pub struct WorkgroupSizes {
    x: usize,
    y: usize,
    z: usize,
    num_dimmensions: usize,
}
impl WorkgroupSizes {
    pub fn num_dimmensions(&self) -> usize {
        self.num_dimmensions
    }
    pub fn from_iter_space(iter_space: IterationSpace) -> Self {
        let num_dimmensions = iter_space.num_dimmensions();
        if num_dimmensions == 3 {
            Self {
                x: 4,
                y: 4,
                z: 4,
                num_dimmensions: 3,
            }
        } else if num_dimmensions == 2 {
            Self {
                x: 8,
                y: 8,
                z: 1,
                num_dimmensions: 2,
            }
        } else {
            Self {
                x: 64,
                y: 1,
                z: 1,
                num_dimmensions: 1,
            }
        }
    }
    pub fn three_d() -> Self {
        Self {
            x: 4,
            y: 4,
            z: 4,
            num_dimmensions: 3,
        }
    }
    pub fn two_d() -> Self {
        Self {
            x: 8,
            y: 8,
            z: 1,
            num_dimmensions: 2,
        }
    }
    pub fn one_d() -> Self {
        Self {
            x: 64,
            y: 1,
            z: 1,
            num_dimmensions: 1,
        }
    }
    pub fn custom_use_at_own_risk(x: usize, y: usize, z: usize, num_dimmensions: usize) -> Self {
        Self {
            x,
            y,
            z,
            num_dimmensions,
        }
    }
    pub fn x(&self) -> usize {
        self.x
    }
    pub fn y(&self) -> usize {
        self.y
    }
    pub fn z(&self) -> usize {
        self.z
    }
}

#[derive(Resource)]
pub struct BatchCollidablePopulation(pub usize);

#[derive(Component)]
/// Will always use the static map if it is present, otherwise it will use the callback
pub struct MaxNumGpuOutputItemsPerOutputType {
    pub unique_id: usize,
    map: HashMap<String, usize>,
    callback: Option<fn(iter_space: &IterationSpace, output_variable_name: String) -> usize>,
    pub uses_callback: bool,
}
impl MaxNumGpuOutputItemsPerOutputType {
    pub fn new(map: HashMap<String, usize>) -> Self {
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
        initial_output_variable_names: Vec<String>,
        callback: fn(iter_space: &IterationSpace, output_variable_name: String) -> usize,
    ) -> Self {
        let mut map = HashMap::default();
        for output_variable_name in initial_output_variable_names {
            map.insert(
                output_variable_name.clone(),
                callback(initial_iteration_space, output_variable_name),
            );
        }
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
    pub fn update_with_callback(
        &mut self,
        iter_space: &IterationSpace,
        output_variable_names: Vec<String>,
    ) {
        if let Some(callback) = self.callback {
            for output_variable_name in output_variable_names {
                self.map.insert(
                    output_variable_name.clone(),
                    callback(iter_space, output_variable_name),
                );
            }
            self.unique_id += 1;
        }
    }
    pub fn get(&self, result_type: &str) -> usize {
        if let Some(r) = self.map.get(result_type).copied() {
            r
        } else {
            panic!("No max output value found for result type: {}", result_type);
        }
    }
}

#[derive(Resource)]
pub struct NumGpuWorkgroupsRequired {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}
