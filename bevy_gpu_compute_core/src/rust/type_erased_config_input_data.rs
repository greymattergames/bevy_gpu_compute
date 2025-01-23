use bytemuck::Pod;
use std::collections::HashMap;
pub struct TypeErasedConfigInputData {
    bytes_per_wgsl_config_type_name: HashMap<String, Vec<u8>>,
}
impl TypeErasedConfigInputData {
    pub fn new(bytes_per_wgsl_config_type_name: HashMap<String, Vec<u8>>) -> Self {
        Self {
            bytes_per_wgsl_config_type_name,
        }
    }
    pub fn set<T: Pod + Send + Sync + std::fmt::Debug>(&mut self, config_name: &str, data: T) {
        self.bytes_per_wgsl_config_type_name
            .insert(config_name.to_string(), bytemuck::bytes_of(&data).to_vec());
    }
    pub fn get_bytes(&self, config_name: &str) -> Option<&[u8]> {
        self.bytes_per_wgsl_config_type_name
            .get(config_name)
            .map(|v| v.as_slice())
    }
    pub fn get_map(&self) -> &HashMap<String, Vec<u8>> {
        &self.bytes_per_wgsl_config_type_name
    }
}
