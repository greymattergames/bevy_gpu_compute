use minimal_proc_macro::fn_macro_ast_viz_debug;
#[test]
fn test_wgsl_ln() {
    use wgsl_ln::wgsl;

    // pub static MANHATTAN_DISTANCE: &str = wgsl!(
    //     fn manhattan_distance(a: vec2<f32>, b: vec2<f32>) -> f32 {
    //         return abs(a.x - b.x) + abs(a.y - b.y);
    //     }
    // );
    // pub static TEST_2: &str = wgsl!(
    //     @group(0) @binding(0) var<storage, read> positions: Positions;
    // );
}

// #[test]
// fn test_proc_macro() {
// println!("test_proc_macro");
// fn_macro_ast_viz_debug!();
// assert_eq!(foo(), 42);
// }
