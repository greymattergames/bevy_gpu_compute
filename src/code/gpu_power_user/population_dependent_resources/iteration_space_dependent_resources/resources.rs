use std::{alloc::System, time::SystemTime};

use bevy::{prelude::Resource, time::Time, utils::HashMap};

use crate::code::gpu_power_user::population_dependent_resources::resources::IterationSpace;

#[derive(Resource)]
pub struct BatchCollidablePopulation(pub usize);

#[derive(Resource)]
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
        self.map.get(result_type).copied().unwrap_or(0)
    }
}

#[derive(Resource)]
pub struct NumGpuWorkgroupsRequired {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}
