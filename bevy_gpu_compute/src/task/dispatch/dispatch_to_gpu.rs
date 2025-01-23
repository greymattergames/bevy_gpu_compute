use bevy::{
    ecs::batching::BatchingStrategy,
    prelude::{Query, Res},
    render::renderer::{RenderDevice, RenderQueue},
};
use bevy_gpu_compute_core::TypesSpec;

use crate::task::{
    compute_pipeline::cache::{PipelineKey, PipelineLruCache},
    task_commands::GpuTaskCommands,
    task_components::task::BevyGpuComputeTask,
    task_specification::{
        gpu_workgroup_space::GpuWorkgroupSpace, task_specification::ComputeTaskSpecification,
    },
};
pub fn dispatch_to_gpu(
    task: &mut BevyGpuComputeTask,
    render_device: &RenderDevice,
    render_queue: &RenderQueue,
) {
    let mut encoder = render_device.create_command_encoder(&Default::default());
    {
        let mut compute_pass = encoder.begin_compute_pass(&Default::default());
        let key = PipelineKey {
            pipeline_consts_version: task.spec.iter_space_and_out_lengths_version(),
        };
        compute_pass.set_pipeline(&task.pipeline_cache.cache.get(&key).unwrap());
        compute_pass.set_bind_group(0, task.bind_group.as_ref().unwrap(), &[]);
        compute_pass.dispatch_workgroups(
            task.num_gpu_workgroups_required.x(),
            task.num_gpu_workgroups_required.y(),
            task.num_gpu_workgroups_required.z(),
        );
    }
    render_queue.submit(std::iter::once(encoder.finish()));
}
