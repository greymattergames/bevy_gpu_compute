use std::collections::HashMap;

use bevy::{
    ecs::system::SystemParam,
    prelude::{Commands, Entity, Query, Res, Resource},
    render::renderer::{RenderDevice, RenderQueue},
};
use bevy_gpu_compute_core::{
    TypesSpec, wgsl::shader_module::user_defined_portion::WgslShaderModuleUserPortion,
};

use crate::{
    prelude::ComputeTaskSpecification,
    ram_limit::RamLimit,
    task::{
        buffers::{
            create_config_input_buffers::update_config_input_buffers,
            create_input_buffers::update_input_buffers,
            create_output_buffers::update_output_buffers,
        },
        compute_pipeline::update_on_pipeline_const_change::update_compute_pipeline,
        dispatch::{create_bind_group::create_bind_group, dispatch_to_gpu::dispatch_to_gpu},
        outputs::{
            read_gpu_output_counts::read_gpu_output_counts, read_gpu_task_outputs::read_gpu_outputs,
        },
        task_commands::{GpuTaskCommand, GpuTaskCommands},
        task_components::task::BevyGpuComputeTask,
        verify_enough_memory::verify_have_enough_memory,
    },
};

/// The decision to require the user to call this instead of running the commands directly on reciept was made to preserve the API flow of `GpuTaskRunner.task("my_task_name").some_command()`, while working around limitations with passing references to ECS components and resources (lifetime issues).
#[derive(SystemParam)]
pub struct GpuTaskRunner<'w, 's> {
    tasks: Query<'w, 's, (Entity, &'static mut BevyGpuComputeTask)>,
    render_device: Res<'w, RenderDevice>,
    render_queue: Res<'w, RenderQueue>,
    ram_limit: Res<'w, RamLimit>,
}

impl<'w, 's> GpuTaskRunner<'w, 's> {
    /// get a GpuTaskCommands object, which is actually a queue of commands to be run.
    /// #### You MUST call `run_commands` on the returned object to actually run the commands.
    pub fn task(&mut self, name: &str) -> GpuTaskCommands {
        let (entity, _) = self
            .tasks
            .iter_mut()
            .find(|(_, task)| task.name() == name)
            .expect("Task not found");

        GpuTaskCommands::new(entity)
    }

    /// Runs all previously queued commands for the task
    pub fn run_commands(&mut self, commands: GpuTaskCommands) {
        let (_, mut task) = self
            .tasks
            .get_mut(commands.entity())
            .expect("Task entity not found");

        for cmd in commands.commands {
            match cmd {
                GpuTaskCommand::SetConfigInputs(inputs) => {
                    task.config_input_data = Some(*inputs);
                    update_config_input_buffers(&mut task, &self.render_device);
                }
                GpuTaskCommand::SetInputs(data) => {
                    task.input_data = Some(*data);
                    update_input_buffers(&mut task, &self.render_device);
                    create_bind_group(&mut task, &self.render_device);
                }
                GpuTaskCommand::Mutate {
                    iteration_space,
                    max_output_lengths,
                } => {
                    task.spec.mutate(iteration_space, max_output_lengths);
                    update_compute_pipeline(&mut task, &self.render_device);
                    update_output_buffers(&mut task, &self.render_device);
                }
                GpuTaskCommand::Run => {
                    dispatch_to_gpu(&mut task, &self.render_device, &self.render_queue);
                    let output_counts =
                        read_gpu_output_counts(&mut task, &self.render_device, &self.render_queue);
                    read_gpu_outputs(
                        output_counts,
                        &mut task,
                        &self.render_device,
                        &self.render_queue,
                    );
                }
            }
        }
    }
}
