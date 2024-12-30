use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;

use bytemuck::Pod;

// This holds type information for a specific output
pub struct TypedOutputSpec<T: Pod> {
    pub label: String,
    pub item_bytes: usize,
    _phantom: PhantomData<T>,
}

// Modified to hold type information
pub struct GpuAccBevyComputeTaskOutputSpecs {
    pub specs: HashMap<String, (usize, TypeId)>,
}

// Builder-style API for users to register their types
impl GpuAccBevyComputeTaskOutputSpecs {
    pub fn new() -> Self {
        Self {
            specs: HashMap::new(),
        }
    }

    pub fn register<T: Pod + 'static>(&mut self, label: &str, item_bytes: usize) {
        self.specs
            .insert(label.to_string(), (item_bytes, TypeId::of::<Vec<T>>()));
    }
}
