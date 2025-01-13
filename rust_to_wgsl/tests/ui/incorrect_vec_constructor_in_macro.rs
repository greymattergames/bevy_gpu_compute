use rust_to_wgsl::shader_module;

#[shader_module]
mod my_mod {
    use shared::wgsl_in_rust_helpers::*;
    const MY_CONST: Vec3Bool = Vec3Bool {
        x: true,
        y: false,
        z: true,
    };
    fn main(global_id: WgslGlobalId) {}
}

fn main() {}
