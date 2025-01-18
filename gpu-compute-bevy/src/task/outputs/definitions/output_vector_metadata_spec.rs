use bevy::prelude::Component;
use shared::{
    custom_type_name::{self, CustomTypeName},
    misc_types::OutputVectorTypesSpec,
};

pub struct OutputVectorMetadataDefinition<'a> {
    pub binding_number: u32,
    pub include_count: bool,
    pub count_binding_number: Option<u32>,
    pub name: &'a CustomTypeName,
}
#[derive(Clone, Debug)]
pub struct OutputVectorMetadata {
    bytes: usize,
    binding_number: u32,
    include_count: bool,
    count_binding_number: Option<u32>,
    name: CustomTypeName,
}

impl OutputVectorMetadata {
    pub fn new(
        bytes: usize,
        binding_number: u32,
        include_count: bool,
        count_binding_number: Option<u32>,
        name: CustomTypeName,
    ) -> Self {
        OutputVectorMetadata {
            bytes,
            binding_number,
            include_count,
            count_binding_number,
            name,
        }
    }
    pub fn get_bytes(&self) -> usize {
        self.bytes
    }
    pub fn get_binding_number(&self) -> u32 {
        self.binding_number
    }
    pub fn get_include_count(&self) -> bool {
        self.include_count
    }
    pub fn get_count_binding_number(&self) -> Option<u32> {
        self.count_binding_number
    }
    pub fn name(&self) -> &CustomTypeName {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct OutputVectorsMetadataSpec {
    output0: Option<OutputVectorMetadata>,
    output1: Option<OutputVectorMetadata>,
    output2: Option<OutputVectorMetadata>,
    output3: Option<OutputVectorMetadata>,
    output4: Option<OutputVectorMetadata>,
    output5: Option<OutputVectorMetadata>,
}
impl Default for OutputVectorsMetadataSpec {
    fn default() -> Self {
        Self::empty()
    }
}

impl OutputVectorsMetadataSpec {
    pub fn empty() -> Self {
        OutputVectorsMetadataSpec {
            output0: None,
            output1: None,
            output2: None,
            output3: None,
            output4: None,
            output5: None,
        }
    }
    fn get_output<ST>(
        i: usize,
        definitions: &[Option<OutputVectorMetadataDefinition>; 6],
    ) -> Option<OutputVectorMetadata> {
        if let Some(def) = &definitions[i] {
            Some(OutputVectorMetadata::new(
                std::mem::size_of::<ST>(),
                def.binding_number,
                def.include_count,
                if def.include_count {
                    Some(def.count_binding_number.unwrap())
                } else {
                    None
                },
                def.name.clone(),
            ))
        } else {
            None
        }
    }
    pub fn from_output_vector_types_spec<T: OutputVectorTypesSpec>(
        definitions: [Option<OutputVectorMetadataDefinition>; 6],
    ) -> Self {
        Self {
            output0: Self::get_output::<T::Output0>(0, &definitions),
            output1: Self::get_output::<T::Output1>(1, &definitions),
            output2: Self::get_output::<T::Output2>(2, &definitions),
            output3: Self::get_output::<T::Output3>(3, &definitions),
            output4: Self::get_output::<T::Output4>(4, &definitions),
            output5: Self::get_output::<T::Output5>(5, &definitions),
        }
    }
    pub fn get_all_metadata(&self) -> [Option<&OutputVectorMetadata>; 6] {
        [
            self.output0.as_ref(),
            self.output1.as_ref(),
            self.output2.as_ref(),
            self.output3.as_ref(),
            self.output4.as_ref(),
            self.output5.as_ref(),
        ]
    }
    pub fn get_output0_metadata(&self) -> Option<&OutputVectorMetadata> {
        self.output0.as_ref()
    }
    pub fn get_output1_metadata(&self) -> Option<&OutputVectorMetadata> {
        self.output1.as_ref()
    }
    pub fn get_output2_metadata(&self) -> Option<&OutputVectorMetadata> {
        self.output2.as_ref()
    }
    pub fn get_output3_metadata(&self) -> Option<&OutputVectorMetadata> {
        self.output3.as_ref()
    }
    pub fn get_output4_metadata(&self) -> Option<&OutputVectorMetadata> {
        self.output4.as_ref()
    }
    pub fn get_output5_metadata(&self) -> Option<&OutputVectorMetadata> {
        self.output5.as_ref()
    }
    pub fn set_output0_metadata(&mut self, metadata: OutputVectorMetadata) {
        self.output0 = Some(metadata);
    }
    pub fn set_output1_metadata(&mut self, metadata: OutputVectorMetadata) {
        self.output1 = Some(metadata);
    }
    pub fn set_output2_metadata(&mut self, metadata: OutputVectorMetadata) {
        self.output2 = Some(metadata);
    }
    pub fn set_output3_metadata(&mut self, metadata: OutputVectorMetadata) {
        self.output3 = Some(metadata);
    }
    pub fn set_output4_metadata(&mut self, metadata: OutputVectorMetadata) {
        self.output4 = Some(metadata);
    }
    pub fn set_output5_metadata(&mut self, metadata: OutputVectorMetadata) {
        self.output5 = Some(metadata);
    }
}
