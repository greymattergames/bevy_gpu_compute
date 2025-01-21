// Most commonly needed types/traits that users will want
pub use crate::GpuAcceleratedBevy;
pub use crate::GpuAcceleratedBevyPlugin;
pub use crate::GpuAcceleratedBevyState;

// Task-related types users commonly need
pub use crate::task::events::GpuComputeTaskSuccessEvent;
pub use crate::task::task_specification::iteration_space::IterationSpace;
pub use crate::task::task_specification::max_output_vector_lengths::MaxOutputLengths;

// Proc macros
pub use bevy_gpu_compute_macro::wgsl_config;
pub use bevy_gpu_compute_macro::wgsl_input_array;
pub use bevy_gpu_compute_macro::wgsl_output_array;
pub use bevy_gpu_compute_macro::wgsl_output_vec;
pub use bevy_gpu_compute_macro::wgsl_shader_module;

//
pub use bevy_gpu_compute_core::wgsl_in_rust_helpers::*;

use plugin::*;
