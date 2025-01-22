use bevy_gpu_compute_macro::wgsl_shader_module;

#[wgsl_shader_module]
pub mod something {

    /** This is doc comment */
    /// Doc comment\n
    fn main() {
        // This is a line comment
    }
}
