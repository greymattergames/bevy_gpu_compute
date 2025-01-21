use rust_to_wgsl::shader_module;

#[shader_module]
mod my_mod {
    use shared::wgsl_in_rust_helpers::*;
    type MyAlias = Vec3F32;
    fn main(global_id: WgslGlobalId) {}
}

fn main() {}
