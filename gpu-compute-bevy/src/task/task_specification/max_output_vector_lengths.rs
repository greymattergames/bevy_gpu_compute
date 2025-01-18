use std::{collections::HashMap, time::SystemTime};

use bevy::prelude::Component;
use shared::custom_type_name::CustomTypeName;

#[derive(Debug, Clone, PartialEq)]
/**
### These vectors lengths are very important for overall performance, the lower the better
#### But if they are too low they will cut off valid output data

*/
pub struct MaxOutputLengths {
    length_per_wgsl_output_type_name: HashMap<String, usize>,
}
impl Default for MaxOutputLengths {
    fn default() -> Self {
        Self {
            length_per_wgsl_output_type_name: HashMap::default(),
        }
    }
}

impl MaxOutputLengths {
    pub fn new(length_per_wgsl_output_type_name: HashMap<String, usize>) -> Self {
        Self {
            length_per_wgsl_output_type_name: length_per_wgsl_output_type_name,
        }
    }
    pub fn empty() -> Self {
        Self {
            length_per_wgsl_output_type_name: HashMap::default(),
        }
    }

    pub fn get_by_name(&self, output_item_name: &CustomTypeName) -> usize {
        return self.length_per_wgsl_output_type_name[&output_item_name.output_array_length()];
    }
    pub fn set(&mut self, output_type_name: &str, length: usize) {
        self.length_per_wgsl_output_type_name
            .insert(output_type_name.to_string(), length);
    }
    pub fn get_map(&self) -> &HashMap<String, usize> {
        &self.length_per_wgsl_output_type_name
    }
}
