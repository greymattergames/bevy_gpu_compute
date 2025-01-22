use bevy_gpu_compute_macro::wgsl_shader_module;
#[wgsl_shader_module]
pub mod test_module {
    use bevy_gpu_compute_core::wgsl_helpers::*;
    use bevy_gpu_compute_macro::wgsl_config;
    #[wgsl_config]
    struct MyConfig {
        value: bool,
    }
    fn main(iter_pos: WgslIterationPosition) {}
}
