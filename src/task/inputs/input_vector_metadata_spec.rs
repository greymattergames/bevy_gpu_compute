use bevy::prelude::Component;

use super::input_vector_types_spec::InputVectorTypesSpec;

pub struct InputVectorMetadataDefinition {
    pub binding_number: u32,
}
#[derive(Clone, Copy)]
pub struct InputVectorMetadata {
    bytes: usize,
    binding_number: u32,
}

impl InputVectorMetadata {
    pub fn new(bytes: usize, binding_number: u32) -> Self {
        InputVectorMetadata {
            bytes,
            binding_number,
        }
    }
    pub fn get_bytes(&self) -> usize {
        self.bytes
    }
    pub fn get_binding_number(&self) -> u32 {
        self.binding_number
    }
}

#[derive(Clone, Copy)]
pub struct InputVectorsMetadataSpec {
    input0: Option<InputVectorMetadata>,
    input1: Option<InputVectorMetadata>,
    input2: Option<InputVectorMetadata>,
    input3: Option<InputVectorMetadata>,
    input4: Option<InputVectorMetadata>,
    input5: Option<InputVectorMetadata>,
}

impl Default for InputVectorsMetadataSpec {
    fn default() -> Self {
        Self::empty()
    }
}

impl InputVectorsMetadataSpec {
    pub fn empty() -> Self {
        InputVectorsMetadataSpec {
            input0: None,
            input1: None,
            input2: None,
            input3: None,
            input4: None,
            input5: None,
        }
    }
    fn get_input<ST>(
        i: usize,
        definitions: [Option<&InputVectorMetadataDefinition>; 6],
    ) -> Option<InputVectorMetadata> {
        if let Some(def) = definitions[i] {
            Some(InputVectorMetadata::new(
                std::mem::size_of::<ST>(),
                def.binding_number,
            ))
        } else {
            None
        }
    }
    pub fn from_input_vector_types_spec<T: InputVectorTypesSpec>(
        definitions: [Option<&InputVectorMetadataDefinition>; 6],
    ) -> Self {
        Self {
            input0: Self::get_input::<T::Input0>(0, definitions),
            input1: Self::get_input::<T::Input1>(1, definitions),
            input2: Self::get_input::<T::Input2>(2, definitions),
            input3: Self::get_input::<T::Input3>(3, definitions),
            input4: Self::get_input::<T::Input4>(4, definitions),
            input5: Self::get_input::<T::Input5>(5, definitions),
        }
    }
    pub fn get_all_metadata(&self) -> [Option<&InputVectorMetadata>; 6] {
        [
            self.input0.as_ref(),
            self.input1.as_ref(),
            self.input2.as_ref(),
            self.input3.as_ref(),
            self.input4.as_ref(),
            self.input5.as_ref(),
        ]
    }
    pub fn get_input0_metadata(&self) -> Option<&InputVectorMetadata> {
        self.input0.as_ref()
    }
    pub fn get_input1_metadata(&self) -> Option<&InputVectorMetadata> {
        self.input1.as_ref()
    }
    pub fn get_input2_metadata(&self) -> Option<&InputVectorMetadata> {
        self.input2.as_ref()
    }
    pub fn get_input3_metadata(&self) -> Option<&InputVectorMetadata> {
        self.input3.as_ref()
    }
    pub fn get_input4_metadata(&self) -> Option<&InputVectorMetadata> {
        self.input4.as_ref()
    }
    pub fn get_input5_metadata(&self) -> Option<&InputVectorMetadata> {
        self.input5.as_ref()
    }
    pub fn set_input0_metadata(&mut self, metadata: InputVectorMetadata) {
        self.input0 = Some(metadata);
    }
    pub fn set_input1_metadata(&mut self, metadata: InputVectorMetadata) {
        self.input1 = Some(metadata);
    }
    pub fn set_input2_metadata(&mut self, metadata: InputVectorMetadata) {
        self.input2 = Some(metadata);
    }
    pub fn set_input3_metadata(&mut self, metadata: InputVectorMetadata) {
        self.input3 = Some(metadata);
    }
    pub fn set_input4_metadata(&mut self, metadata: InputVectorMetadata) {
        self.input4 = Some(metadata);
    }
    pub fn set_input5_metadata(&mut self, metadata: InputVectorMetadata) {
        self.input5 = Some(metadata);
    }
}
