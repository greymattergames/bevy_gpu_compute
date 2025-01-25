use bevy_gpu_compute_core::{
    TypeErasedArrayInputData, TypeErasedArrayOutputData, TypeErasedConfigInputData,
};

use super::input_lengths::InputArrayDataLengths;

pub struct TaskData {
    config_input: Option<TypeErasedConfigInputData>,
    input: Option<TypeErasedArrayInputData>,
    output: Option<TypeErasedArrayOutputData>,
    input_lengths: Option<InputArrayDataLengths>,
}

impl Default for TaskData {
    fn default() -> Self {
        TaskData {
            config_input: None,
            input: None,
            output: None,
            input_lengths: None,
        }
    }
}
impl TaskData {
    pub fn new() -> Self {
        TaskData::default()
    }
    pub fn config_input(&self) -> &Option<TypeErasedConfigInputData> {
        &self.config_input
    }
    pub fn input(&self) -> &Option<TypeErasedArrayInputData> {
        &self.input
    }
    pub fn output(&self) -> &Option<TypeErasedArrayOutputData> {
        &self.output
    }
    pub fn output_mut(&mut self) -> &mut Option<TypeErasedArrayOutputData> {
        &mut self.output
    }
    pub fn input_lengths(&self) -> &Option<InputArrayDataLengths> {
        &self.input_lengths
    }
    pub fn set_config_input(&mut self, config_input: TypeErasedConfigInputData) {
        self.config_input = Some(config_input);
    }
    //todo inputs and lengths must be linked
    // pub fn set_input(&mut self, input: TypeErasedArrayInputData) {
    // self.input = Some(input);
    // }
    pub fn set_output(&mut self, output: TypeErasedArrayOutputData) {
        self.output = Some(output);
    }
    pub fn clear_output(&mut self) {
        self.output = None;
    }
    /// returns `true` if lengths have changed, `false` otherwise
    pub fn set_input_and_check_lengths_changed(&mut self, input: TypeErasedArrayInputData) -> bool {
        let mut lengths_changed = false;
        let lengths = InputArrayDataLengths::new(input.get_lengths().clone());
        if self.input_lengths.is_none() {
            self.input_lengths = Some(lengths);
            lengths_changed = true;
        } else {
            let new_hash = self
                .input_lengths
                .as_mut()
                .unwrap()
                .update_and_return_new_hash_if_changed(lengths);
            if new_hash.is_some() {
                lengths_changed = true;
            }
        }
        self.input = Some(input);
        lengths_changed
    }
    // pub fn set_input_lengths(&mut self, input_lengths: InputArrayDataLengths) {
    // self.input_lengths = Some(input_lengths);
    // }
}
