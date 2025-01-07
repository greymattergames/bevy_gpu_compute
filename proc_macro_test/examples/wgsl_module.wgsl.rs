//todo remove this
#![feature(f16)]

// provided wgsl type helpers
trait WgslScalar {}
impl WgslScalar for bool {}
impl WgslScalar for u32 {}
impl WgslScalar for i32 {}
impl WgslScalar for f32 {}
impl WgslScalar for f16 {}

type Vec2<T: WgslScalar> = [T; 2];
type Vec3<T: WgslScalar> = [T; 3];
type Vec4<T: WgslScalar> = [T; 4];

type Mat2x2<T: WgslScalar> = [Vec2<T>; 2];
type Mat3x3<T: WgslScalar> = [Vec3<T>; 3];
type Mat4x4<T: WgslScalar> = [Vec4<T>; 4];

//* user Shader-Module-Constant
const example_module_const: u32 = 42;
//* library generated per-pipeline constants, will be inserted below

//*  user generated buffer types
// only one group of uniforms because this library is designed for simple compute shaders
struct Uniforms {
    time: f32,
    resolution: Vec2<f32>,
}
//* user input vectors
//todo this changes to alias
type Position = [f32; 2]; // todo, this changes to array<f32,
type Radius = f32;
//* user output vectors
struct CollisionResult {
    entity1: u32,
    entity2: u32,
}

//*  library generated buffer types, will go below

//* bindings, all handled by the library, will go below

//* USER GENERATED HELPER FUNCTIONS
// Optimized distance calculation
// todo the array defs convert
fn calculate_distance_squared(p1: [f32; 2], p2: [f32; 2]) -> f32 {
    let dx = p1[0] - p2[0];
    let dy = p1[1] - p2[1];
    return dx * dx + dy * dy;
}
//* Library generated helper functions, will go below

//todo this is automatically made available
struct GlobalId {
    x: u32,
    y: u32,
    z: u32,
}

//todo, this gets removed and all calls to its methods get reformatted
struct WgslInput {}

impl WgslInput {
    fn vec_len<T>() -> u32 {
        unimplemented!()
    }
    fn vec_val<T>(index: u32) -> T {
        unimplemented!()
    }
}
struct WgslOutput {}
impl WgslOutput {
    fn vec_push<T>(val: T) {
        unimplemented!()
    }
    fn vec_set<T>(index: u32, val: T) {
        unimplemented!()
    }
}

//* ENTRY POINT FUNCTION
// @compute @workgroup_size(WORKGROUP_SIZE_X, WORKGROUP_SIZE_Y, WORKGROUP_SIZE_Z)
fn wgsl_main(
    //todo change name to main
    // @builtin(global_invocation_id) global_id: vec3<u32>,
    // todo this changes to the above
    global_id: GlobalId,
) {
    //* USER GENERATED LOGIC
    let current_entity = global_id.x;
    let other_entity = global_id.y;
    // Early exit if invalid entity or zero radius
    if current_entity >= WgslInput::vec_len::<Position>()
        || other_entity >= WgslInput::vec_len::<Position>()
        || current_entity == other_entity
        || current_entity >= other_entity
    {
        return;
    }

    let current_radius = WgslInput::vec_val::<Radius>(current_entity);
    let other_radius = WgslInput::vec_val::<Radius>(other_entity);
    if current_radius <= 0.0 || other_radius <= 0.0 {
        return;
    }
    let current_pos = WgslInput::vec_val::<Position>(current_entity);
    let other_pos = WgslInput::vec_val::<Position>(other_entity);

    let dist_squared = calculate_distance_squared(current_pos, other_pos);
    let radius_sum = current_radius + other_radius;

    // Compare squared distances to avoid sqrt
    if dist_squared < radius_sum * radius_sum {
        WgslOutput::vec_push::<CollisionResult>(CollisionResult {
            entity1: current_entity,
            entity2: other_entity,
        });
    }
}
