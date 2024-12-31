use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;

use bevy::prelude::Component;
use bytemuck::Pod;

use super::input_data::InputData;

// This holds type information for a specific Input
pub struct InputSpec {
    pub item_bytes: usize,
    pub type_id: TypeId,
    pub binding_number: u32,
    /// If true may slightly improve performance
    pub skip_validation: bool,
}

// Modified to hold type information
#[derive(Component)]
pub struct InputSpecs {
    /// Maps a label to a tuple of (item_bytes, type_id, binding_number)
    pub specs: HashMap<String, InputSpec>,
}
impl Default for InputSpecs {
    fn default() -> Self {
        Self::new()
    }
}

// Builder-style API for users to register their types
impl InputSpecs {
    pub fn new() -> Self {
        Self {
            specs: HashMap::new(),
        }
    }

    pub fn register<T: Pod + 'static>(&mut self, label: &str, binding_number: u32) {
        let item_bytes = std::mem::size_of::<T>();
        self.specs.insert(label.to_string(), InputSpec {
            item_bytes,
            type_id: TypeId::of::<Vec<T>>(),
            binding_number,
            skip_validation: false,
        });
    }
    pub fn skip_validation<T: Pod + 'static>(&mut self, label: &str) {
        if let Some(spec) = self.specs.get_mut(label) {
            spec.skip_validation = true;
        }
    }
    pub fn validate_data(&self, input_data: &InputData) -> Result<(), String> {
        for (label, spec) in self.specs.iter() {
            let data = input_data
                .data
                .get(label)
                .ok_or_else(|| format!("Missing input data for label: {}", label))?;

            if data.type_id() != spec.type_id {
                return Err(format!(
                    "Type mismatch for {}: expected {:?}, got {:?}",
                    label,
                    spec.type_id,
                    data.type_id()
                ));
            }
        }
        Ok(())
    }
}
