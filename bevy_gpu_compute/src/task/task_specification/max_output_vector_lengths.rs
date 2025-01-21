use std::collections::HashMap;

use bevy_gpu_compute_core::custom_type_name::ShaderCustomTypeName;

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

    pub fn get_by_name(&self, output_item_name: &ShaderCustomTypeName) -> usize {
        assert!(
            self.length_per_wgsl_output_type_name
                .contains_key(output_item_name.name()),
            " could not find {} in {:?} for max output lengths",
            output_item_name.name(),
            self.length_per_wgsl_output_type_name
        );
        return self.length_per_wgsl_output_type_name[output_item_name.name()];
    }
    pub fn set(&mut self, output_type_name: &str, length: usize) {
        self.length_per_wgsl_output_type_name
            .insert(output_type_name.to_string(), length);
    }
    pub fn get_map(&self) -> &HashMap<String, usize> {
        &self.length_per_wgsl_output_type_name
    }
}
