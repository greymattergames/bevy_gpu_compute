use std::cmp::min;

use bevy::{
    log,
    render::renderer::{RenderDevice, RenderQueue},
};
use bevy_gpu_compute_core::TypesSpec;

use crate::task::{task_commands::GpuTaskCommands, task_components::task::BevyGpuComputeTask};

use super::{
    definitions::type_erased_output_data::TypeErasedOutputData,
    helpers::get_gpu_output_as_bytes_vec::get_gpu_output_as_bytes_vec,
};

/**
 * We put this all into a single system because we cannot pass the buffer slice around easily.
 * */
pub fn read_gpu_outputs(
    output_counts: Vec<Option<usize>>,
    task: &mut BevyGpuComputeTask,
    render_device: &RenderDevice,
    render_queue: &RenderQueue,
) {
    let mut type_erased_output = TypeErasedOutputData::empty();

    task.spec
        .output_vectors_metadata_spec()
        .get_all_metadata()
        .iter()
        .enumerate()
        .for_each(|(i, metadata)| {
            if let Some(m) = metadata {
                let out_buffer = task.buffers.output.get(i).unwrap();
                let staging_buffer = task.buffers.output_staging.get(i).unwrap();
                let total_byte_size = min(
                    if let Some(Some(c)) = output_counts.get(i) {
                        let size = c * m.get_bytes();
                        log::info!("using output count to size buffer, size: {}", size);
                        size
                    } else {
                        usize::MAX
                    },
                    task.spec.output_array_lengths().get_by_name(m.name()) * m.get_bytes(),
                );
                log::info!("total_byte_size: {}", total_byte_size);
                if total_byte_size < 1 {
                    type_erased_output.set_output_from_bytes(i, Vec::new());
                } else {
                    let raw_bytes = get_gpu_output_as_bytes_vec(
                        &render_device,
                        &render_queue,
                        &out_buffer,
                        staging_buffer,
                        total_byte_size as u64,
                    );
                    // log::info!("raw_bytes: {:?}", raw_bytes);
                    if let Some(raw_bytes) = raw_bytes {
                        type_erased_output.set_output_from_bytes(i, raw_bytes);
                    } else {
                        panic!("Failed to read output from GPU");
                    }
                }
            }
        });
    task.output_data = Some(type_erased_output)
}
