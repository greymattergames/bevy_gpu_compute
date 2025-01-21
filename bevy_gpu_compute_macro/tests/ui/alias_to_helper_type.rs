use bevy_gpu_compute_macro::wgsl_shader_module;

#[wgsl_shader_module]
mod my_mod {
    use bevy_gpu_compute_core::wgsl_helpers::*;
    type MyAlias = Vec3F32;
    fn main(iter_pos: WgslIterationPosition) {}
}

fn main() {}
