use bevy::prelude::Component;
use shared::misc_types::InputVectorTypesSpec;
use std::any::Any;

use super::input_data::{InputData, InputDataTrait};

#[derive(Component)]
pub struct TypeErasedInputData {
    inner: Box<dyn InputDataTrait>,
}
impl TypeErasedInputData {
    pub fn new<T: InputVectorTypesSpec + 'static + Send + Sync>(input_data: InputData<T>) -> Self {
        Self {
            inner: Box::new(input_data),
        }
    }
    pub fn input_bytes(&self, index: usize) -> Option<&[u8]> {
        self.inner.input_bytes(index)
    }
}
