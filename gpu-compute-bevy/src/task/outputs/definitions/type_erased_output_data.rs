use bevy::prelude::Component;
use shared::misc_types::OutputVectorTypesSpec;

use super::output_data::OutputData;

#[derive(Clone, Component)]
pub struct TypeErasedOutputData {
    output0: Option<Vec<u8>>,
    output1: Option<Vec<u8>>,
    output2: Option<Vec<u8>>,
    output3: Option<Vec<u8>>,
    output4: Option<Vec<u8>>,
    output5: Option<Vec<u8>>,
}
impl Default for TypeErasedOutputData {
    fn default() -> Self {
        TypeErasedOutputData {
            output0: None,
            output1: None,
            output2: None,
            output3: None,
            output4: None,
            output5: None,
        }
    }
}

impl TypeErasedOutputData {
    pub fn empty() -> Self {
        TypeErasedOutputData {
            output0: None,
            output1: None,
            output2: None,
            output3: None,
            output4: None,
            output5: None,
        }
    }

    pub fn set_output_from_bytes(&mut self, index: usize, bytes: Vec<u8>) {
        match index {
            0 => self.output0 = Some(bytes),
            1 => self.output1 = Some(bytes),
            2 => self.output2 = Some(bytes),
            3 => self.output3 = Some(bytes),
            4 => self.output4 = Some(bytes),
            5 => self.output5 = Some(bytes),
            _ => panic!("Invalid output index"),
        }
    }

    pub fn into_typed<T: OutputVectorTypesSpec>(self) -> Result<OutputData<T>, String> {
        let mut output_data = OutputData::empty();

        if let Some(bytes) = self.output0 {
            output_data.set_output0_from_bytes(&bytes)?;
        }
        if let Some(bytes) = self.output1 {
            output_data.set_output1_from_bytes(&bytes)?;
        }
        if let Some(bytes) = self.output2 {
            output_data.set_output2_from_bytes(&bytes)?;
        }
        if let Some(bytes) = self.output3 {
            output_data.set_output3_from_bytes(&bytes)?;
        }
        if let Some(bytes) = self.output4 {
            output_data.set_output4_from_bytes(&bytes)?;
        }
        if let Some(bytes) = self.output5 {
            output_data.set_output5_from_bytes(&bytes)?;
        }

        Ok(output_data)
    }
}
