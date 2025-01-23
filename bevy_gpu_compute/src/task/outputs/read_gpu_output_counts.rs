use std::sync::{Arc, Mutex};

use bevy::{
    log,
    render::renderer::{RenderDevice, RenderQueue},
};
use wgpu::Buffer;

use crate::task::task_components::task::BevyGpuComputeTask;

use super::{
    definitions::wgsl_counter::WgslCounter,
    helpers::get_gpu_output_counter_value::get_gpu_output_counter_value,
};

pub fn read_gpu_output_counts(
    task: &mut BevyGpuComputeTask,
    render_device: &RenderDevice,
    render_queue: &RenderQueue,
) -> Vec<Option<usize>> {
    let local_res_counts: Arc<Mutex<Vec<Option<usize>>>> = Arc::new(Mutex::new(Vec::new()));
    task.spec
        .output_vectors_metadata_spec()
        .get_all_metadata()
        .iter()
        .enumerate()
        .for_each(|(i, spec)| {
            if let Some(s) = spec {
                if s.get_include_count() {
                    log::info!("Reading count for output {}", i);
                    let count = read_gpu_output_counts_single_output_type(
                        render_device,
                        render_queue,
                        &task.buffers.output_count[i],
                        &task.buffers.output_count_staging[i],
                    );
                    local_res_counts.lock().unwrap().push(Some(count as usize));
                } else {
                    local_res_counts.lock().unwrap().push(None);
                }
            } else {
                local_res_counts.lock().unwrap().push(None);
            }
        });
    local_res_counts.lock().unwrap().to_vec()
}

fn read_gpu_output_counts_single_output_type(
    render_device: &RenderDevice,
    render_queue: &RenderQueue,
    count_buffer: &Buffer,
    count_staging_buffer: &Buffer,
) -> u32 {
    let count = get_gpu_output_counter_value(
        &render_device,
        &render_queue,
        &count_buffer,
        &count_staging_buffer,
        std::mem::size_of::<WgslCounter>() as u64,
    );
    let r = count.unwrap().count;
    log::info!("Read count: {}", r);
    r
}
