use bevy::{
    ecs::system::SystemParam,
    log,
    prelude::{Entity, Query, Res},
    render::renderer::{RenderDevice, RenderQueue},
};

use crate::{
    ram_limit::RamLimit,
    task::{
        buffers::{
            create_config_input_buffers::update_config_input_buffers,
            create_input_buffers::update_input_buffers,
            create_output_buffers::update_output_buffers,
        },
        commands::{GpuTaskCommand, GpuTaskCommands},
        compute_pipeline::update_on_pipeline_const_change::update_compute_pipeline,
        dispatch::{create_bind_group::create_bind_group, dispatch_to_gpu::dispatch_to_gpu},
        lib::BevyGpuComputeTask,
        outputs::{
            read_gpu_output_counts::read_gpu_output_counts, read_gpu_task_outputs::read_gpu_outputs,
        },
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

impl GpuTaskRunner<'_, '_> {
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
        let mut should_recompute_memory = false;
        for cmd in commands.commands {
            log::info!("Running command: {}", cmd);
            match cmd {
                GpuTaskCommand::SetConfigInputs(inputs) => {
                    task.current_data_mut().set_config_input(*inputs);
                    update_config_input_buffers(&mut task, &self.render_device);
                }
                GpuTaskCommand::SetInputs(data) => {
                    let lengths_changed = task
                        .current_data_mut()
                        .set_input_and_check_lengths_changed(*data);
                    if lengths_changed {
                        update_compute_pipeline(&mut task, &self.render_device);
                    }
                    update_input_buffers(&mut task, &self.render_device);
                    create_bind_group(&mut task, &self.render_device);
                }
                GpuTaskCommand::Mutate {
                    iteration_space,
                    max_output_lengths,
                } => {
                    task.mutate(iteration_space, max_output_lengths);
                    update_compute_pipeline(&mut task, &self.render_device);
                    update_output_buffers(&mut task, &self.render_device);
                    should_recompute_memory = true;
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
        if should_recompute_memory {
            let all_tasks: Vec<_> = self.tasks.iter().map(|(_, t)| t).collect();
            verify_have_enough_memory(&all_tasks, &self.ram_limit);
        }
    }
}
