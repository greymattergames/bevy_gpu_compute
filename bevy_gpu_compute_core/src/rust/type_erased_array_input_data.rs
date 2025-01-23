use bytemuck::Pod;
use std::collections::HashMap;
pub struct TypeErasedArrayInputData {
    bytes_per_wgsl_input_type_name: HashMap<String, Vec<u8>>,
    lengths_per_wgsl_input_type_name: HashMap<String, usize>,
}
impl TypeErasedArrayInputData {
    pub fn new(
        bytes_per_wgsl_input_type_name: HashMap<String, Vec<u8>>,
        lengths_per_wgsl_input_type_name: HashMap<String, usize>,
    ) -> Self {
        Self {
            bytes_per_wgsl_input_type_name,
            lengths_per_wgsl_input_type_name,
        }
    }
    pub fn set<T: Pod + Send + Sync + std::fmt::Debug>(&mut self, input_name: &str, data: Vec<T>) {
        let length = data.len();
        self.bytes_per_wgsl_input_type_name
            .insert(input_name.to_string(), bytemuck::cast_slice(&data).to_vec());
        self.lengths_per_wgsl_input_type_name
            .insert(input_name.to_string(), length);
    }
    pub fn get_bytes(&self, input_name: &str) -> Option<&[u8]> {
        self.bytes_per_wgsl_input_type_name
            .get(input_name)
            .map(|v| v.as_slice())
    }
    pub fn get_length(&self, input_name: &str) -> Option<usize> {
        self.lengths_per_wgsl_input_type_name
            .get(input_name)
            .copied()
    }
    pub fn get_lengths(&self) -> &HashMap<String, usize> {
        &self.lengths_per_wgsl_input_type_name
    }
    pub fn get_map(&self) -> &HashMap<String, Vec<u8>> {
        &self.bytes_per_wgsl_input_type_name
    }
}
