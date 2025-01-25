use crate::task::inputs::{
    array_type::input_vector_metadata_spec::InputVectorsMetadataSpec,
    config_type::config_input_metadata_spec::ConfigInputsMetadataSpec,
};

#[derive(Clone, Default)]

pub struct InputSpec {
    arrays: InputVectorsMetadataSpec,
    configs: ConfigInputsMetadataSpec,
}

impl InputSpec {
    pub fn new(arrays: InputVectorsMetadataSpec, configs: ConfigInputsMetadataSpec) -> Self {
        InputSpec { arrays, configs }
    }
    pub fn arrays(&self) -> &InputVectorsMetadataSpec {
        &self.arrays
    }
    pub fn configs(&self) -> &ConfigInputsMetadataSpec {
        &self.configs
    }
}
