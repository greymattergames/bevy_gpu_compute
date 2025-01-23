use std::collections::HashMap;

pub struct TypeErasedArrayOutputData {
    bytes_per_wgsl_output_type_name: HashMap<String, Vec<u8>>,
}
impl TypeErasedArrayOutputData {
    pub fn new(bytes_per_wgsl_output_type_name: HashMap<String, Vec<u8>>) -> Self {
        Self {
            bytes_per_wgsl_output_type_name,
        }
    }
    pub fn set(&mut self, input_name: &str, bytes: &[u8]) {
        self.bytes_per_wgsl_output_type_name
            .insert(input_name.to_string(), bytes.to_vec());
    }
    pub fn get_bytes(&self, input_name: &str) -> Option<&[u8]> {
        self.bytes_per_wgsl_output_type_name
            .get(input_name)
            .map(|v| v.as_slice())
    }
}

pub trait OutputDataBuilderTrait {
    fn from(out_data: &TypeErasedArrayOutputData) -> Self;
}
