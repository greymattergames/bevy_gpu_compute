fn main() {
    // Watch shader source files for changes
    println!("cargo:rerun-if-changed=examples/collision_shader.rs");

    // Optional: Pre-generate WGSL at build time
    // generate_shader_modules().unwrap();
}
