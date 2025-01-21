use bevy_gpu_compute_macro::wgsl_shader_module;

#[wgsl_shader_module]
mod my_mod {
    use bevy_gpu_compute_core::wgsl_helpers::*;
    const MY_CONST: Vec3Bool = Vec3Bool {
        x: true,
        y: false,
        z: true,
    };
    fn main(iter_pos: WgslIterationPosition) {}
}

fn main() {}
