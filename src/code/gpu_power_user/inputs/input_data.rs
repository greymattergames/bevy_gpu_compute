use std::any::Any;

use bevy::{prelude::Component, utils::HashMap};
use bytemuck::Pod;

#[derive(Component)]

pub struct InputData {
    pub data: HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl InputData {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn insert<T: Pod + Send + Sync + 'static>(&mut self, label: &str, data: Vec<T>) {
        self.data.insert(label.to_string(), Box::new(data));
    }
}
