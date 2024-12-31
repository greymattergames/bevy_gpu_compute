use bevy::{
    diagnostic::SystemInfo,
    ecs::batching::BatchingStrategy,
    log,
    prelude::{Query, Res},
};
use sysinfo::{MemoryRefreshKind, RefreshKind, System};

use super::{
    iteration_space_dependent_resources::max_num_outputs_per_type::MaxNumGpuOutputItemsPerOutputType,
    outputs::output_spec::OutputSpecs,
};

pub fn verify_enough_memory(
    tasks: Query<(&OutputSpecs, &MaxNumGpuOutputItemsPerOutputType)>,
    sys_info: Res<SystemInfo>,
) {
    let total_bytes = tasks
        .iter()
        .fold(0, |sum, (output_specs, max_num_outputs)| {
            sum + get_max_output_bytes_single_task(output_specs, max_num_outputs)
        });
    let sys = System::new_with_specifics(
        RefreshKind::nothing().with_memory(MemoryRefreshKind::nothing().with_ram()),
    );
    assert!(sys.total_memory() > 0);
    let available_memory = sys.total_memory() as f32;
    if total_bytes as f32 > available_memory * 0.9 {
        log::error!(
            "Not enough memory to store all outputs, either reduce the number of entities or allow more potential collision misses by lowering the max_detectable_collisions_scale"
        );
        log::info!(
            "Available memory: {} GB",
            available_memory as f32 / 1024.0 / 1024.0 / 1024.0
        );
        log::info!(
            "Max Output size: {} GB",
            total_bytes as f32 / 1024.0 / 1024.0 / 1024.0
        );
        panic!("Not enough memory to store all outputs");
    }
}

fn get_max_output_bytes_single_task(
    output_specs: &OutputSpecs,
    max_num_outputs: &MaxNumGpuOutputItemsPerOutputType,
) -> usize {
    output_specs.specs.iter().fold(0, |sum, (label, spec)| {
        sum + max_num_outputs.get(label) * spec.item_bytes
    })
}
