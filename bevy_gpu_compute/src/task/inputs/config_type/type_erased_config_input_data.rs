use bevy::prelude::Component;
use bevy_gpu_compute_core::TypesSpec;

use super::config_data::{ConfigInputData, ConfigInputDataTrait};

#[derive(Component)]
pub struct TypeErasedConfigInputData {
    inner: Box<dyn ConfigInputDataTrait>,
}
impl TypeErasedConfigInputData {
    pub fn new<T: TypesSpec + 'static + Send + Sync>(input_data: ConfigInputData<T>) -> Self {
        Self {
            inner: Box::new(input_data),
        }
    }
    pub fn input_bytes(&self, index: usize) -> Option<&[u8]> {
        self.inner.input_bytes(index)
    }
}
