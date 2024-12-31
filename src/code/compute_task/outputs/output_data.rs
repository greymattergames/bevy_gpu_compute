use bevy::prelude::Component;
use bytemuck::Pod;

use super::output_spec::{BlankOutputVectorTypesSpec, OutputVectorMetadata, OutputVectorTypesSpec};

#[derive(Component)]
pub struct OutputData<T: OutputVectorTypesSpec> {
    output1: Option<Vec<T::Output1>>,
    output2: Option<Vec<T::Output2>>,
    output3: Option<Vec<T::Output3>>,
    output4: Option<Vec<T::Output4>>,
    output5: Option<Vec<T::Output5>>,
    output6: Option<Vec<T::Output6>>,

    _phantom: std::marker::PhantomData<T>,
}

impl Default for OutputData<BlankOutputVectorTypesSpec> {
    fn default() -> Self {
        OutputData {
            output1: None,
            output2: None,
            output3: None,
            output4: None,
            output5: None,
            output6: None,

            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: OutputVectorTypesSpec> OutputData<T> {
    pub fn new() -> Self {
        OutputData {
            output1: None,
            output2: None,
            output3: None,
            output4: None,
            output5: None,
            output6: None,

            _phantom: std::marker::PhantomData,
        }
    }
    pub fn get_output1_metadata() -> Option<&'static OutputVectorMetadata> {
        T::OUTPUT1_METADATA.as_ref()
    }
    pub fn get_output2_metadata() -> Option<&'static OutputVectorMetadata> {
        T::OUTPUT2_METADATA.as_ref()
    }
    pub fn get_output3_metadata() -> Option<&'static OutputVectorMetadata> {
        T::OUTPUT3_METADATA.as_ref()
    }
    pub fn get_output4_metadata() -> Option<&'static OutputVectorMetadata> {
        T::OUTPUT4_METADATA.as_ref()
    }
    pub fn get_output5_metadata() -> Option<&'static OutputVectorMetadata> {
        T::OUTPUT5_METADATA.as_ref()
    }
    pub fn get_output6_metadata() -> Option<&'static OutputVectorMetadata> {
        T::OUTPUT6_METADATA.as_ref()
    }
    pub fn get_all_metadata() -> [Option<&'static OutputVectorMetadata>; 6] {
        [
            T::OUTPUT1_METADATA.as_ref(),
            T::OUTPUT2_METADATA.as_ref(),
            T::OUTPUT3_METADATA.as_ref(),
            T::OUTPUT4_METADATA.as_ref(),
            T::OUTPUT5_METADATA.as_ref(),
            T::OUTPUT6_METADATA.as_ref(),
        ]
    }

    // Set outputs from raw bytes
    pub fn set_output1_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        if bytes.len() % std::mem::size_of::<T::Output1>() != 0 {
            return Err("Byte length not aligned with output type size".to_string());
        }

        self.output1 = Some(bytemuck::cast_slice(bytes).to_vec());
        Ok(())
    }

    pub fn set_output2_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        if bytes.len() % std::mem::size_of::<T::Output2>() != 0 {
            return Err("Byte length not aligned with output type size".to_string());
        }

        self.output2 = Some(bytemuck::cast_slice(bytes).to_vec());
        Ok(())
    }
    pub fn set_output3_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        if bytes.len() % std::mem::size_of::<T::Output3>() != 0 {
            return Err("Byte length not aligned with output type size".to_string());
        }

        self.output3 = Some(bytemuck::cast_slice(bytes).to_vec());
        Ok(())
    }
    pub fn set_output4_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        if bytes.len() % std::mem::size_of::<T::Output4>() != 0 {
            return Err("Byte length not aligned with output type size".to_string());
        }

        self.output4 = Some(bytemuck::cast_slice(bytes).to_vec());
        Ok(())
    }
    pub fn set_output5_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        if bytes.len() % std::mem::size_of::<T::Output5>() != 0 {
            return Err("Byte length not aligned with output type size".to_string());
        }

        self.output5 = Some(bytemuck::cast_slice(bytes).to_vec());
        Ok(())
    }
    pub fn set_output6_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        if bytes.len() % std::mem::size_of::<T::Output6>() != 0 {
            return Err("Byte length not aligned with output type size".to_string());
        }

        self.output6 = Some(bytemuck::cast_slice(bytes).to_vec());
        Ok(())
    }

    // Type-safe getters for processed results
    pub fn get_output1(&self) -> Option<&[T::Output1]> {
        self.output1.as_deref()
    }

    pub fn get_output2(&self) -> Option<&[T::Output2]> {
        self.output2.as_deref()
    }
    pub fn get_output3(&self) -> Option<&[T::Output3]> {
        self.output3.as_deref()
    }
    pub fn get_output4(&self) -> Option<&[T::Output4]> {
        self.output4.as_deref()
    }
    pub fn get_output5(&self) -> Option<&[T::Output5]> {
        self.output5.as_deref()
    }
    pub fn get_output6(&self) -> Option<&[T::Output6]> {
        self.output6.as_deref()
    }
}
