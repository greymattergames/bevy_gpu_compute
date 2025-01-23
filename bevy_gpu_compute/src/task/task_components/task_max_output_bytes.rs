use bevy_gpu_compute_core::MaxOutputLengths;

use crate::task::outputs::definitions::output_vector_metadata_spec::OutputVectorsMetadataSpec;

#[derive(Debug)]
pub struct TaskMaxOutputBytes(usize);

impl Default for TaskMaxOutputBytes {
    fn default() -> Self {
        TaskMaxOutputBytes(0)
    }
}
impl TaskMaxOutputBytes {
    pub fn new(max_output_bytes: usize) -> Self {
        TaskMaxOutputBytes(max_output_bytes)
    }
    pub fn from_max_lengths_and_spec(
        max_output_vector_lengths: &MaxOutputLengths,
        output_vector_metadata_spec: &OutputVectorsMetadataSpec,
    ) -> Self {
        let max_output_bytes = output_vector_metadata_spec.get_all_metadata().iter().fold(
            0,
            |acc, output_metadata| {
                if let Some(m) = output_metadata {
                    acc + max_output_vector_lengths.get_by_name(m.name()) * m.get_bytes()
                } else {
                    acc
                }
            },
        );
        TaskMaxOutputBytes(max_output_bytes)
    }
    pub fn get(&self) -> usize {
        self.0
    }
}
