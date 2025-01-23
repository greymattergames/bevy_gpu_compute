// Proc macros
pub use bevy_gpu_compute_macro::wgsl_config;
pub use bevy_gpu_compute_macro::wgsl_input_array;
pub use bevy_gpu_compute_macro::wgsl_output_array;
pub use bevy_gpu_compute_macro::wgsl_output_vec;
pub use bevy_gpu_compute_macro::wgsl_shader_module;

//helpers when writing the shader module:
pub use bevy_gpu_compute_core::wgsl_helpers::*;

pub use crate::plugin::BevyGpuComputePlugin;
pub use crate::run_ids::BevyGpuComputeRunIds;

pub use crate::system_params::task_creator::BevyGpuComputeTaskCreator;
pub use crate::system_params::task_deleter::BevyGpuComputeTaskDeleter;
pub use crate::system_params::task_reader::GpuTaskReader;
pub use crate::system_params::task_runner::GpuTaskRunner;
pub use crate::task::task_components::task_run_id::TaskRunId;
pub use crate::task::task_specification::iteration_space::IterationSpace;
pub use crate::task::task_specification::task_specification::ComputeTaskSpecification;
