use bevy::prelude::Component;

use crate::task::outputs::definitions::{
    max_output_vector_lengths::MaxOutputVectorLengths,
    output_vector_metadata_spec::OutputVectorMetadataSpec,
};

#[derive(Component)]
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
        max_output_vector_lengths: &MaxOutputVectorLengths,
        output_vector_metadata_spec: &OutputVectorMetadataSpec,
    ) -> Self {
        let max_output_bytes = output_vector_metadata_spec
            .get_all_metadata()
            .iter()
            .enumerate()
            .fold(0, |acc, (i, output_metadata)| {
                if let Some(m) = output_metadata {
                    acc + max_output_vector_lengths.get(i) * m.get_bytes()
                } else {
                    acc
                }
            });
        TaskMaxOutputBytes(max_output_bytes)
    }
    pub fn get(&self) -> usize {
        self.0
    }
}
