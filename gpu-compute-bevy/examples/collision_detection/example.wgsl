const MY_CONST : bool = true;
override _LIB_WORKGROUP_SIZE_X: u32;
override _LIB_WORKGROUP_SIZE_Y: u32;
override _LIB_WORKGROUP_SIZE_Z: u32;
override POSITION_INPUT_ARRAY_LENGTH: u32;
override RADIUS_INPUT_ARRAY_LENGTH: u32;
override COLLISIONRESULT_OUTPUT_ARRAY_LENGTH: u32;
struct Config { time : f32, resolution : vec2 < f32 > , }
struct Position { v : vec2 < f32 > , }
alias PositionInputArray  = array < Position, POSITION_INPUT_ARRAY_LENGTH > ;
alias Radius  = f32;
alias RadiusInputArray  = array < Radius, RADIUS_INPUT_ARRAY_LENGTH > ;
struct CollisionResult { entity1 : u32, entity2 : u32, }
alias CollisionResultOutputArray  = array < CollisionResult,
COLLISIONRESULT_OUTPUT_ARRAY_LENGTH > ;
@group(0) @binding(0) var<uniform> config: Config;

@group(0) @binding(1) var<storage, read> position_input_array: PositionInputArray;

@group(0) @binding(2) var<storage, read> radius_input_array: RadiusInputArray;

@group(0) @binding(3) var<storage, read_write> collisionresult_output_array: CollisionResultOutputArray;

@group(0) @binding(4) var<storage, read_write> collisionresult_counter: atomic<u32>;

fn calculate_distance_squared(p1 : vec2 < f32 > , p2 : vec2 < f32 >) -> f32
{ let dx = p1.x - p2 [0]; let dy = p1.y - p2 [1]; return dx * dx + dy * dy; }

#ifdef ONE_DIMMENSIONAL
   @compute @workgroup_size(64, 1, 1)
#elseif TWO_DIMMENSIONAL
   @compute @workgroup_size(8,8,1)
#else 
   @compute @workgroup_size(4,4,4)
#endif

fn main(@builtin(global_invocation_id) iter_pos: vec3<u32>)
{
    let current_entity = iter_pos.x; let other_entity = iter_pos.y; let
    out_of_bounds = current_entity >= POSITION_INPUT_ARRAY_LENGTH ||
    other_entity >= POSITION_INPUT_ARRAY_LENGTH; if out_of_bounds ||
    current_entity == other_entity || current_entity >= other_entity
    { return; } let current_radius = radius_input_array [current_entity]; let
    other_radius = radius_input_array [other_entity]; if current_radius <= 0.0
    || other_radius <= 0.0 { return; } let current_pos = position_input_array
    [current_entity]; let other_pos = position_input_array [other_entity]; let
    dist_squared = calculate_distance_squared(current_pos.v, other_pos.v); let
    radius_sum = current_radius + other_radius; if dist_squared < radius_sum *
    radius_sum
    {
        {
            let collisionresult_output_array_index =
            atomicAdd(& collisionresult_counter, 1u); if
            collisionresult_output_array_index <
            COLLISIONRESULT_OUTPUT_ARRAY_LENGTH
            {
                collisionresult_output_array
                [collisionresult_output_array_index] =
                CollisionResult(current_entity, other_entity);
            }
        };
    }
}