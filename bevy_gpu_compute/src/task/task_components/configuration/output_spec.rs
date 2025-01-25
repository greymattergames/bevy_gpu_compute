use bevy_gpu_compute_core::MaxOutputLengths;

use crate::task::outputs::definitions::output_vector_metadata_spec::OutputVectorsMetadataSpec;

#[derive(Clone, Default)]

pub struct OutputSpec {
    arrays: OutputVectorsMetadataSpec,
    max_lengths: MaxOutputLengths,
}
impl OutputSpec {
    pub fn new(arrays: OutputVectorsMetadataSpec, max_lengths: MaxOutputLengths) -> Self {
        OutputSpec {
            arrays,
            max_lengths,
        }
    }
    pub fn arrays(&self) -> &OutputVectorsMetadataSpec {
        &self.arrays
    }
    pub fn max_lengths(&self) -> &MaxOutputLengths {
        &self.max_lengths
    }
    /// ensure that runtime state was properly updated whenever you update max lengths
    pub fn _internal_set_max_lengths(&mut self, new_max_lengths: MaxOutputLengths) {
        self.max_lengths = new_max_lengths;
    }
}
