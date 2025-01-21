use bevy::{
    ecs::batching::BatchingStrategy,
    prelude::{Query, Res},
    render::renderer::{RenderDevice, RenderQueue},
};

use crate::task::{
    compute_pipeline::cache::{PipelineKey, PipelineLruCache},
    task_specification::{
        gpu_workgroup_space::GpuWorkgroupSpace, task_specification::ComputeTaskSpecification,
    },
};

use super::create_bind_group::BindGroupComponent;

pub fn dispatch_to_gpu(
    mut tasks: Query<(
        &ComputeTaskSpecification,
        &BindGroupComponent,
        &mut PipelineLruCache,
    )>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    tasks
        .par_iter_mut()
        .batching_strategy(BatchingStrategy::default())
        .for_each(|(task_spec, bind_group, mut pipeline_cache)| {
            dispatch_to_gpu_single_task(
                &render_device,
                &render_queue,
                bind_group,
                task_spec.iter_space_and_out_lengths_version(),
                task_spec.gpu_workgroup_space(),
                &mut pipeline_cache,
            );
        });
}

fn dispatch_to_gpu_single_task(
    render_device: &RenderDevice,
    render_queue: &RenderQueue,
    bind_group: &BindGroupComponent,
    pipeline_consts_version: u64,
    num_gpu_workgroups_required: &GpuWorkgroupSpace,
    compute_pipeline_cache: &mut PipelineLruCache,
) {
    let mut encoder = render_device.create_command_encoder(&Default::default());
    {
        let mut compute_pass = encoder.begin_compute_pass(&Default::default());
        let key = PipelineKey {
            pipeline_consts_version: pipeline_consts_version,
        };
        compute_pass.set_pipeline(&compute_pipeline_cache.cache.get(&key).unwrap());
        compute_pass.set_bind_group(0, bind_group.0.as_ref().unwrap(), &[]);
        compute_pass.dispatch_workgroups(
            num_gpu_workgroups_required.x(),
            num_gpu_workgroups_required.y(),
            num_gpu_workgroups_required.z(),
        );
    }
    render_queue.submit(std::iter::once(encoder.finish()));
}
