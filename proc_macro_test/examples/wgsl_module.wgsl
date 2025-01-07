//* user Shader-Module-Constant
const example_module_const: u32 = 42;
//* library generated per-pipeline constants, will be inserted below
override POSITION_INPUT_ARRAY_LENGTH u32;
override RAIDUS_INPUT_ARRAY_LENGTH u32;
override COLLISIONRESULT_OUTPUT_ARRAY_LENGTH u32;

//*  user generated buffer types
// only one group of uniforms because this library is designed for simple compute shaders
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
}
//* user input vectors
alias Position = array<f32, 2>;
alias Radius = f32;
//* user output vectors
struct CollisionResult {
    entity1: u32,
    entity2: u32,
}

//*  library generated buffer types, will go below

//* bindings, all handled by the library, will go below

//* USER GENERATED HELPER FUNCTIONS
// Optimized distance calculation
fn calculate_distance_squared(p1: array<f32, 2>, p2: array<f32, 2>) -> f32 {
    let dx = p1[0] - p2[0];
    let dy = p1[1] - p2[1];
    return dx * dx + dy * dy;
}
//* Library generated helper functions, will go below

//* ENTRY POINT FUNCTION
@compute @workgroup_size(WORKGROUP_SIZE_X, WORKGROUP_SIZE_Y, WORKGROUP_SIZE_Z)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    //* USER GENERATED LOGIC
    let current_entity = global_id.x;
    let other_entity = global_id.y;
    // Early exit if invalid entity or zero radius
    if current_entity >= POSITION_INPUT_ARRAY_LENGTH
        || other_entity >= POSITION_INPUT_ARRAY_LENGTH
        || current_entity == other_entity
        || current_entity >= other_entity
    {
        return;
    }

    let current_radius = radius_input_array[current_entity];
    let other_radius = radius_input_array[other_entity];
    if current_radius <= 0.0 || other_radius <= 0.0 {
        return;
    }
    let current_pos = position_input_array[current_entity];
    let other_pos = position_input_array[other_entity];

    let dist_squared = calculate_distance_squared(current_pos, other_pos);
    let radius_sum = current_radius + other_radius;

    // Compare squared distances to avoid sqrt
    if dist_squared < radius_sum * radius_sum {
        let index = atomicAdd(&collisionresult_counter, 1u);
        if index < COLLISIONRESULT_OUTPUT_ARRAY_LENGTH{
            collisionresult_output_array[index] = CollisionResult(current_entity,other_entity);
        }   
    }
}
