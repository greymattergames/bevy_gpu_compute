// use minimal_proc_macro::wgsl;
#[test]
fn test_wgsl_mine() {
    use wgsl_ln::wgsl;

    //* user input vectors
    struct Position {
        pub x: [f32; 2],
    };
    struct Radius(pub f32);
    // struct CollisionResult {
    // entity1: u32,
    // entity2: u32,
    // }
    // struct Uniforms {
    //     time: f32,
    //     resolution: vec2<f32>,
    // }

    pub static MANHATTAN_DISTANCE: &str = wgsl!(
        struct Position {
            x: (f32, f32),
        }
    );
    // pub static un iforms: &str = wgsl!(Uniforms);
    // pub static TEST_2: &str = wgsl!(
    // @group(0) @binding(0) var<storage, read> positions: Positions;
    // );
}

// #[test]
// fn test_proc_macro() {
// println!("test_proc_macro");
// fn_macro_ast_viz_debug!();
// assert_eq!(foo(), 42);
// }
