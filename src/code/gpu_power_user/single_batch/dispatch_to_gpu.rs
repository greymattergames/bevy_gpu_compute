use bevy::{
    prelude::{Res, ResMut},
    render::renderer::{RenderDevice, RenderQueue},
};

use crate::code::gpu_power_user::{
    iteration_space_dependent_resources::{
        pipeline::cache::{PipelineCache, PipelineKey},
        resources::{IterationSpace, MaxNumGpuOutputItemsPerOutputType, NumGpuWorkgroupsRequired},
    },
    resources::WgslCode,
};

use super::resources::BindGroup;

pub fn dispatch_to_gpu(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut compute_pipeline_cache: ResMut<PipelineCache>,
    bind_group: Res<BindGroup>,
    num_gpu_workgroups_required: Res<NumGpuWorkgroupsRequired>,
    wgsl_code: Res<WgslCode>,
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
