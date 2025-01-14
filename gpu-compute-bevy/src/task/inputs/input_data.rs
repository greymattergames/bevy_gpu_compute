use bevy::{log, prelude::Component};
use shared::misc_types::InputVectorTypesSpec;

use super::{
    input_vector_types_spec::BlankInputVectorTypesSpec, type_erased_input_data::TypeErasedInputData,
};

pub trait InputDataTrait: Send + Sync {
    fn input_bytes(&self, index: usize) -> Option<&[u8]>;
}

#[derive(Component, Debug)]
pub struct InputData<T: InputVectorTypesSpec> {
    input0: Option<Vec<T::Input0>>,
    input1: Option<Vec<T::Input1>>,
    input2: Option<Vec<T::Input2>>,
    input3: Option<Vec<T::Input3>>,
    input4: Option<Vec<T::Input4>>,
    input5: Option<Vec<T::Input5>>,
    _phantom: std::marker::PhantomData<T>,
}
impl Default for InputData<BlankInputVectorTypesSpec> {
    fn default() -> Self {
        InputData {
            input0: None,
            input1: None,
            input2: None,
            input3: None,
            input4: None,
            input5: None,

            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: InputVectorTypesSpec> InputData<T> {
    pub fn empty() -> Self {
        InputData {
            input0: None,
            input1: None,
            input2: None,
            input3: None,
            input4: None,
            input5: None,

            _phantom: std::marker::PhantomData,
        }
    }

    // Type-safe setters that take vectors of Pod types
    pub fn set_input0(&mut self, input: Vec<T::Input0>) {
        self.input0 = Some(input);
    }

    pub fn set_input1(&mut self, input: Vec<T::Input1>) {
        self.input1 = Some(input);
    }
    pub fn set_input2(&mut self, input: Vec<T::Input2>) {
        self.input2 = Some(input);
    }
    pub fn set_input3(&mut self, input: Vec<T::Input3>) {
        self.input3 = Some(input);
    }
    pub fn set_input4(&mut self, input: Vec<T::Input4>) {
        self.input4 = Some(input);
    }
    pub fn set_input5(&mut self, input: Vec<T::Input5>) {
        self.input5 = Some(input);
    }

    pub fn input0_bytes(&self) -> Option<&[u8]> {
        if let Some(data) = &self.input0 {
            Some(bytemuck::cast_slice(data))
        } else {
            None
        }
    }

    pub fn input1_bytes(&self) -> Option<&[u8]> {
        if let Some(data) = &self.input1 {
            Some(bytemuck::cast_slice(data))
        } else {
            None
        }
    }
    pub fn input2_bytes(&self) -> Option<&[u8]> {
        if let Some(data) = &self.input2 {
            Some(bytemuck::cast_slice(data))
        } else {
            None
        }
    }
    pub fn input3_bytes(&self) -> Option<&[u8]> {
        if let Some(data) = &self.input3 {
            Some(bytemuck::cast_slice(data))
        } else {
            None
        }
    }
    pub fn input4_bytes(&self) -> Option<&[u8]> {
        if let Some(data) = &self.input4 {
            Some(bytemuck::cast_slice(data))
        } else {
            None
        }
    }
    pub fn input5_bytes(&self) -> Option<&[u8]> {
        if let Some(data) = &self.input5 {
            Some(bytemuck::cast_slice(data))
        } else {
            None
        }
    }
}

impl<T: InputVectorTypesSpec + Send + Sync> InputDataTrait for InputData<T> {
    fn input_bytes(&self, index: usize) -> Option<&[u8]> {
        log::info!("input_bytes index: {}", index);
        match index {
            0 => self.input0_bytes(),
            1 => self.input1_bytes(),
            2 => self.input2_bytes(),
            3 => self.input3_bytes(),
            4 => self.input4_bytes(),
            5 => self.input5_bytes(),
            _ => None,
        }
    }
}
