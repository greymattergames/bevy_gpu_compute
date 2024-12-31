use std::any::Any;

use bevy::{prelude::Component, utils::HashMap};
use bytemuck::Pod;

use super::input_spec::{BlankInputVectorTypesSpec, InputVectorMetadata, InputVectorTypesSpec};

#[derive(Component)]
pub struct InputData<T: InputVectorTypesSpec> {
    input1: Option<Vec<T::Input1>>,
    input2: Option<Vec<T::Input2>>,
    input3: Option<Vec<T::Input3>>,
    input4: Option<Vec<T::Input4>>,
    input5: Option<Vec<T::Input5>>,
    input6: Option<Vec<T::Input6>>,
    _phantom: std::marker::PhantomData<T>,
}
impl Default for InputData<BlankInputVectorTypesSpec> {
    fn default() -> Self {
        InputData {
            input1: None,
            input2: None,
            input3: None,
            input4: None,
            input5: None,
            input6: None,

            _phantom: std::marker::PhantomData,
        }
    }
}

// Implementation for the task container
impl<T: InputVectorTypesSpec> InputData<T> {
    pub fn new() -> Self {
        InputData {
            input1: None,
            input2: None,
            input3: None,
            input4: None,
            input5: None,
            input6: None,

            _phantom: std::marker::PhantomData,
        }
    }

    // Type-safe setters that take vectors of Pod types
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
    pub fn set_input6(&mut self, input: Vec<T::Input6>) {
        self.input6 = Some(input);
    }
    // Add methods to access metadata
    // Add methods to access metadata
    pub fn get_input1_metadata() -> Option<&'static InputVectorMetadata> {
        T::INPUT1_METADATA.as_ref()
    }

    pub fn get_input2_metadata() -> Option<&'static InputVectorMetadata> {
        T::INPUT2_METADATA.as_ref()
    }

    pub fn get_input3_metadata() -> Option<&'static InputVectorMetadata> {
        T::INPUT3_METADATA.as_ref()
    }

    pub fn get_input4_metadata() -> Option<&'static InputVectorMetadata> {
        T::INPUT4_METADATA.as_ref()
    }

    pub fn get_input5_metadata() -> Option<&'static InputVectorMetadata> {
        T::INPUT5_METADATA.as_ref()
    }

    pub fn get_input6_metadata() -> Option<&'static InputVectorMetadata> {
        T::INPUT6_METADATA.as_ref()
    }

    // Optional: Helper method to get all metadata
    pub fn get_all_metadata() -> [Option<&'static InputVectorMetadata>; 6] {
        [
            T::INPUT1_METADATA.as_ref(),
            T::INPUT2_METADATA.as_ref(),
            T::INPUT3_METADATA.as_ref(),
            T::INPUT4_METADATA.as_ref(),
            T::INPUT5_METADATA.as_ref(),
            T::INPUT6_METADATA.as_ref(),
        ]
    }

    // Get raw bytes from inputs
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
        // self.input2.as_ref().map(|data| bytemuck::cast_slice(data))
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
    pub fn input6_bytes(&self) -> Option<&[u8]> {
        if let Some(data) = &self.input6 {
            Some(bytemuck::cast_slice(data))
        } else {
            None
        }
    }
    pub fn input_bytes(&self, index: usize) -> Option<&[u8]> {
        match index {
            0 => self.input1_bytes(),
            1 => self.input2_bytes(),
            2 => self.input3_bytes(),
            3 => self.input4_bytes(),
            4 => self.input5_bytes(),
            5 => self.input6_bytes(),
            _ => None,
        }
    }
}
