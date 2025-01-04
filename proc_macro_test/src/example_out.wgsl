//* user Shader-Module-Constant
const example_module_const: u32 = 42u;
//* user Pipeline-Overridable-Constant
override example_override_const: u32 = 42u;
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
//* user input vectors
struct Positions {
    positions: array<array<f32,2>,ARRAY_SIZE>
}
struct Radii {
    radii: array<f32,ARRAY_SIZE>
}
//* user output vectors
struct CollisionResult {
    entity1: u32,
    entity2: u32,
}
struct CollisionResults {
    results: array<CollisionResult, MAX_ARRAY_SIZE>,
}

//*  library generated buffer types
struct Counter {
    count: atomic<u32>,
}

//* bindings, all handled by the library, the user never sees these
@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<storage, read> positions: Positions;
@group(0) @binding(2) var<storage, read> radii: Radii;
@group(0) @binding(3) var<storage, read_write> results: CollisionResults;
@group(0) @binding(4) var<storage, read_write> counter: Counter;

//* USER GENERATED HELPER FUNCTIONS
// Optimized distance calculation
fn calculate_distance_squared(p1: array<f32,2>, p2: array<f32,2>) -> f32 {
    let dx = p1[0] - p2[0];
    let dy = p1[1] - p2[1];
    return dx * dx + dy * dy;
}


//* ENTRY POINT FUNCTION
@compute @workgroup_size(WORKGROUP_SIZE_X, WORKGROUP_SIZE_Y, WORKGROUP_SIZE_Z)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>,
@builtin(workgroup_id) workgroup_id: vec3<u32>
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

    // not part of the actual collision detection, just here as an example, to test funcionality of our macros
    let t = example_const * override example_override_const;

    let dist_squared = calculate_distance_squared(current_pos,other_pos);
    let radius_sum = current_radius + other_radius;
    
    // Compare squared distances to avoid sqrt
    if dist_squared < radius_sum * radius_sum{
        let index = atomicAdd(&counter.count, 1u);
        if index < MAX_ARRAY_SIZE {
            results.results[index].entity1 = current_entity;
            results.results[index].entity2 = other_entity;
        }
    }
}