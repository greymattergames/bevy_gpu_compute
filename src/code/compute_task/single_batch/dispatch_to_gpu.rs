use bevy::{
    ecs::batching::BatchingStrategy,
    prelude::{Query, Res, ResMut},
    render::renderer::{RenderDevice, RenderQueue},
};

use crate::code::compute_task::{
    iteration_space_dependent_resources::{
        pipeline::cache::{PipelineCache, PipelineKey},
        workgroup_sizes::NumGpuWorkgroupsRequired,
    },
    wgsl_code::WgslCode,
};

use super::create_bind_group::BindGroupComponent;

pub fn dispatch_to_gpu(
    mut tasks: Query<(
        &BindGroupComponent,
        &NumGpuWorkgroupsRequired,
        &WgslCode,
        &mut PipelineCache,
    )>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    tasks
        .par_iter_mut()
        .batching_strategy(BatchingStrategy::default())
        .for_each(
            |(bind_group, num_gpu_workgroups_required, wgsl_code, mut pipeline_cache)| {
                dispatch_to_gpu_single_task(
                    &render_device,
                    &render_queue,
                    bind_group,
                    num_gpu_workgroups_required,
                    wgsl_code,
                    &mut pipeline_cache,
                );
            },
        );
}

fn dispatch_to_gpu_single_task(
    render_device: &Res<RenderDevice>,
    render_queue: &Res<RenderQueue>,
    bind_group: &BindGroupComponent,
    num_gpu_workgroups_required: &NumGpuWorkgroupsRequired,
    wgsl_code: &WgslCode,
    compute_pipeline_cache: &mut PipelineCache,
) {
    let mut encoder = render_device.create_command_encoder(&Default::default());
    {
        let mut compute_pass = encoder.begin_compute_pass(&Default::default());
        let key = PipelineKey {
            wgsl_hash: wgsl_code.code_hash as u64,
        };
        compute_pass.set_pipeline(&compute_pipeline_cache.cache.get(&key).unwrap());
        compute_pass.set_bind_group(0, bind_group.0.as_ref().unwrap(), &[]);
        compute_pass.dispatch_workgroups(
            num_gpu_workgroups_required.x,
            num_gpu_workgroups_required.y,
            num_gpu_workgroups_required.z,
        );
    }
    render_queue.submit(std::iter::once(encoder.finish()));
}
