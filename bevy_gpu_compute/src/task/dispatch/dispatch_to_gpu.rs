use bevy::render::renderer::{RenderDevice, RenderQueue};

use crate::task::{compute_pipeline::pipeline_cache::PipelineKey, task::BevyGpuComputeTask};
pub fn dispatch_to_gpu(
    task: &mut BevyGpuComputeTask,
    render_device: &RenderDevice,
    render_queue: &RenderQueue,
) {
    let mut encoder = render_device.create_command_encoder(&Default::default());
    {
        let mut compute_pass = encoder.begin_compute_pass(&Default::default());
        let key = PipelineKey {
            pipeline_consts_version: task.configuration().version(),
        };
        compute_pass.set_pipeline(
            task.runtime_state_mut()
                .pipeline_cache_mut()
                .cache
                .get(&key)
                .unwrap(),
        );
        compute_pass.set_bind_group(0, task.runtime_state().bind_group().as_ref().unwrap(), &[]);
        compute_pass.dispatch_workgroups(
            task.runtime_state().workgroup_space().x(),
            task.runtime_state().workgroup_space().y(),
            task.runtime_state().workgroup_space().z(),
        );
    }
    render_queue.submit(std::iter::once(encoder.finish()));
}
