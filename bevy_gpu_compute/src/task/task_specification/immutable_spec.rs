use crate::task::{
    inputs::{
        array_type::input_vector_metadata_spec::InputVectorsMetadataSpec,
        config_type::config_input_metadata_spec::ConfigInputsMetadataSpec,
    },
    outputs::definitions::output_vector_metadata_spec::OutputVectorsMetadataSpec,
    wgsl_code::WgslCode,
};

pub struct ComputeTaskImmutableSpec {
    output_vectors_metadata_spec: OutputVectorsMetadataSpec,
    input_vectors_metadata_spec: InputVectorsMetadataSpec,
    config_input_metadata_spec: ConfigInputsMetadataSpec,
    wgsl_code: WgslCode,
}

impl Default for ComputeTaskImmutableSpec {
    fn default() -> Self {
        ComputeTaskImmutableSpec {
            output_vectors_metadata_spec: OutputVectorsMetadataSpec::default(),
            input_vectors_metadata_spec: InputVectorsMetadataSpec::default(),
            config_input_metadata_spec: ConfigInputsMetadataSpec::default(),
            wgsl_code: WgslCode::default(),
        }
    }
}

impl ComputeTaskImmutableSpec {
    pub fn new(
        output_vectors_metadata_spec: OutputVectorsMetadataSpec,
        input_vectors_metadata_spec: InputVectorsMetadataSpec,
        config_input_metadata_spec: ConfigInputsMetadataSpec,
        wgsl_code: WgslCode,
    ) -> Self {
        ComputeTaskImmutableSpec {
            output_vectors_metadata_spec,
            input_vectors_metadata_spec,
            config_input_metadata_spec,
            wgsl_code,
        }
    }
    pub fn output_vectors_metadata_spec(&self) -> &OutputVectorsMetadataSpec {
        &self.output_vectors_metadata_spec
    }
    pub fn input_vectors_metadata_spec(&self) -> &InputVectorsMetadataSpec {
        &self.input_vectors_metadata_spec
    }
    pub fn config_input_metadata_spec(&self) -> &ConfigInputsMetadataSpec {
        &self.config_input_metadata_spec
    }
    pub fn wgsl_code(&self) -> &WgslCode {
        &self.wgsl_code
    }
}
