use std::{collections::HashMap, time::SystemTime};

use bevy::prelude::Component;

use super::iteration_space::IterationSpace;

#[derive(Component)]
/// Will always use the static map if it is present, otherwise it will use the callback
pub struct MaxNumGpuOutputItemsPerOutputType {
    pub unique_id: usize,
    map: HashMap<String, usize>,
    callback: Option<fn(iter_space: &IterationSpace, output_variable_name: &String) -> usize>,
    pub uses_callback: bool,
}
impl Default for MaxNumGpuOutputItemsPerOutputType {
    fn default() -> Self {
        Self {
            unique_id: 0,
            map: HashMap::default(),
            callback: None,
            uses_callback: false,
        }
    }
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
        initial_output_variable_names: Vec<&String>,
        callback: fn(iter_space: &IterationSpace, output_variable_name: &String) -> usize,
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
        output_variable_names: Vec<&String>,
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
