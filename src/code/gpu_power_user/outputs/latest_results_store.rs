use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use bevy::prelude::Component;
use bytemuck::Pod;

use super::output_spec::OutputSpecs;

#[derive(Component)]
pub struct LatestResultsStore {
    pub results: HashMap<String, Box<dyn Any + Send + Sync>>,
    type_registry: HashMap<String, TypeId>,
}

impl LatestResultsStore {
    pub fn new(specs: &OutputSpecs) -> Self {
        let type_registry = specs
            .specs
            .iter()
            .map(|(k, spec)| (k.clone(), spec.type_id))
            .collect();

        Self {
            results: HashMap::new(),
            type_registry,
        }
    }

    // Type-safe getter for users
    pub fn get<T: Pod + 'static>(&self, label: &str) -> Option<&Vec<T>> {
        let expected_type = TypeId::of::<Vec<T>>();
        if self.type_registry.get(label) != Some(&expected_type) {
            return None;
        }
        self.results.get(label)?.downcast_ref()
    }
}
