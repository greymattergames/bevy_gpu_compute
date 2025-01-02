use std::hash::{Hash, Hasher};

use bevy::prelude::Component;

#[derive(Clone)]
pub struct WgslCode {
    code: String,
    entry_point_function_name: String,
    pub code_hash: u64,
}
impl Default for WgslCode {
    fn default() -> Self {
        Self {
            code: "".to_string(),
            entry_point_function_name: "".to_string(),
            code_hash: 0,
        }
    }
}

impl WgslCode {
    pub fn new(wgsl_code: String, entry_point_function_name: String) -> Self {
        Self {
            code: wgsl_code.clone(),
            entry_point_function_name,
            code_hash: {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                wgsl_code.hash(&mut hasher);
                hasher.finish()
            },
        }
    }
    pub fn from_file(file_path: &str, entry_point_function_name: String) -> Self {
        let code = std::fs::read_to_string(file_path).unwrap();
        Self::new(code, entry_point_function_name)
    }
    pub fn code(&self) -> &str {
        &self.code
    }
    pub fn entry_point_function_name(&self) -> &str {
        &self.entry_point_function_name
    }
}
