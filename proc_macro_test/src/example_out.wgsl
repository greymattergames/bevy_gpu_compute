//!Final fully formed wgsl
//* user Shader-Module-Constant
const example_module_const: u32 = 42u;
//* library generated per-pipeline constants
override ARRAY_SIZE: u32 = 5;
override MAX_ARRAY_SIZE: u32 = 5;
override WORKGROUP_SIZE_X: u32 = 64;
override WORKGROUP_SIZE_Y: u32 = 1;
override WORKGROUP_SIZE_Z: u32 = 1;


//*  user generated buffer types
// only one group of uniforms because this library is designed for simple compute shaders
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
}
alias Position = array<f32,2>;
//* user input vectors
alias PositionArray array<Position,POSITION_ARRAY_SIZE>;
alias Radius = f32;
alias RadiusArray =array<Radius,RADIUS_ARRAY_SIZE>;
//* user output vectors
struct CollisionResult {
    entity1: u32,
    entity2: u32,
}
alias CollisionResultArray = array<CollisionResult, COLLISION_RESULT_ARRAY_SIZE>;

//*  library generated buffer types
alias CollisionResultCounter = atomic<u32>;

//* bindings, all handled by the library, the user never sees these
@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<storage, read> positions: Positions;
@group(0) @binding(2) var<storage, read> radii: Radii;
@group(0) @binding(3) var<storage, read_write> collision_result_array: CollisionResultArray;
@group(0) @binding(4) var<storage, read_write> collision_result_counter: CollisionResultCounter;

//* USER GENERATED HELPER FUNCTIONS
// Optimized distance calculation
fn calculate_distance_squared(p1: array<f32,2>, p2: array<f32,2>) -> f32 {
    let dx = p1[0] - p2[0];
    let dy = p1[1] - p2[1];
    return dx * dx + dy * dy;
}

//* Library generated helper functions
fn collision_result_push_atomic(
    val:CollisionResult,
    arr: array<CollisionResult, COLLISION_RESULT_ARRAY_SIZE>,
) {
    let index = atomicAdd(&collision_result_counter, 1u);
    if index < COLLISION_RESULT_ARRAY_SIZE{
        arr[index] = val;
    }
}

//* ENTRY POINT FUNCTION
@compute @workgroup_size(WORKGROUP_SIZE_X, WORKGROUP_SIZE_Y, WORKGROUP_SIZE_Z)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>,
) {
    //* USER GENERATED LOGIC
    let current_entity = global_id.x;
    let other_entity = global_id.y;
    // Early exit if invalid entity or zero radius
    if current_entity >= ARRAY_SIZE || other_entity >= ARRAY_SIZE || current_entity == other_entity 
    || current_entity >= other_entity {
        return;
    }

    let current_radius = radii.radii[current_entity];
    let other_radius = radii.radii[other_entity];
    if current_radius <= 0.0 || other_radius <= 0.0 {
        return;
    }
    let current_pos = positions.positions[current_entity];
    let other_pos = positions.positions[other_entity];

    let dist_squared = calculate_distance_squared(current_pos,other_pos);
    let radius_sum = current_radius + other_radius;
    
    // Compare squared distances to avoid sqrt
    if dist_squared < radius_sum * radius_sum{
        collisionresult_push_atomic(CollisionResult(current_entity,other_entity),collision_result_array);
    }
}



//! Intermediate stage, before library insertions
//* user Shader-Module-Constant
const example_module_const: u32 = 42u;
//* library generated per-pipeline constants, will be inserted below


//*  user generated buffer types
// only one group of uniforms because this library is designed for simple compute shaders
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
}
alias Position = array<f32,2>;
//* user input vectors
alias PositionArray array<Position,POSITION_ARRAY_SIZE>;
alias Radius = f32;
alias RadiusArray =array<Radius,RADIUS_ARRAY_SIZE>;
//* user output vectors
struct CollisionResult {
    entity1: u32,
    entity2: u32,
}
alias CollisionResultArray = array<CollisionResult, COLLISION_RESULT_ARRAY_SIZE>;

//*  library generated buffer types, will go below

//* bindings, all handled by the library, will go below

//* USER GENERATED HELPER FUNCTIONS
// Optimized distance calculation
fn calculate_distance_squared(p1: array<f32,2>, p2: array<f32,2>) -> f32 {
    let dx = p1[0] - p2[0];
    let dy = p1[1] - p2[1];
    return dx * dx + dy * dy;
}
//* Library generated helper functions, will go below

//* ENTRY POINT FUNCTION
@compute @workgroup_size(WORKGROUP_SIZE_X, WORKGROUP_SIZE_Y, WORKGROUP_SIZE_Z)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>,
) {
    //* USER GENERATED LOGIC
    let current_entity = global_id.x;
    let other_entity = global_id.y;
    // Early exit if invalid entity or zero radius
    if current_entity >= POSITION_ARRAY_SIZE || other_entity >= POSITION_ARRAY_SIZE || current_entity == other_entity 
    || current_entity >= other_entity {
        return;
    }

    let current_radius = radius_array[current_entity];
    let other_radius = radius_array[other_entity];
    if current_radius <= 0.0 || other_radius <= 0.0 {
        return;
    }
    let current_pos = position_array[current_entity];
    let other_pos = position_array[other_entity];

    let dist_squared = calculate_distance_squared(current_pos,other_pos);
    let radius_sum = current_radius + other_radius;
    
    // Compare squared distances to avoid sqrt
    if dist_squared < radius_sum * radius_sum{
        collision_result_push_atomic(CollisionResult(current_entity,other_entity),collision_result_array);
    }
}