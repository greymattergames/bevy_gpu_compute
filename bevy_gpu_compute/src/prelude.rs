// Proc macros
pub use bevy_gpu_compute_macro::wgsl_config;
pub use bevy_gpu_compute_macro::wgsl_input_array;
pub use bevy_gpu_compute_macro::wgsl_output_array;
pub use bevy_gpu_compute_macro::wgsl_output_vec;
pub use bevy_gpu_compute_macro::wgsl_shader_module;

//helpers when writing the shader module:
pub use bevy_gpu_compute_core::wgsl_helpers::*;

pub use crate::plugin::BevyGpuComputePlugin;
pub use crate::plugin::finished_gpu_tasks;
pub use crate::plugin::starting_gpu_tasks;
pub use crate::resource::BevyGpuCompute;
pub use crate::run_ids::BevyGpuComputeRunIds;
pub use crate::task::events::GpuComputeTaskSuccessEvent;
pub use crate::task::inputs::array_type::input_data::InputData;
pub use crate::task::inputs::config_type::config_data::ConfigInputData;

pub use crate::task::outputs::definitions::type_erased_output_data::TypeErasedOutputData;
pub use crate::task::task_components::task_run_id::TaskRunId;
pub use crate::task::task_specification::iteration_space::IterationSpace;
pub use crate::task::task_specification::max_output_vector_lengths::MaxOutputLengths;
pub use crate::task::task_specification::task_specification::ComputeTaskSpecification;
