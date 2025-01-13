use rust_to_wgsl::shader_module;
use shared::wgsl_components::*;
// This module will be transformed
// the user would not normally input the comments, those are just there for the developer, temporary
#[shader_module]
pub mod collision_shader {

    use rust_to_wgsl::*;
    use shared::wgsl_in_rust_helpers::*;
    //* no other use or import statements allowed, since they break wgsl
    //* user Shader-Module-Constant
    const example_module_const: u32 = 42;
    //* library generated per -pipeline constants, will be inserted below
    //*  user generated buffer types
    // only one group of uniforms because this library is designed for simple compute shaders
    #[wgsl_config]
    struct Uniforms {
        time: f32,
        resolution: Vec2F32,
    }
    //* user input vectors
    //todo this changes to alias
    #[wgsl_input_array]
    type Position = [f32; 2];
    // todo, this changes to array<f32, 2>
    #[wgsl_input_array]
    type Radius = f32;
    //* user output vectors
    #[wgsl_output_vec]
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
    //* ENTRY POINT FUNCTION
    // @compute @workgroup_size(WORKGROUP_SIZE_X, WORKGROUP_SIZE_Y, WORKGROUP_SIZE_Z)
    fn main(
        //todo change name to main
        // @builtin(global_invocation_id) global_id: vec3<u32>,
        // todo this changes to the above
        global_id: WgslGlobalId,
    ) {
        //* USER GENERATED LOGIC
        let current_entity = global_id.x;
        let other_entity = global_id.y;
        // Early exit if invalid entity or zero radius
        if current_entity >= WgslVecInput::vec_len::<Position>()
            || other_entity >= WgslVecInput::vec_len::<Position>()
            || current_entity == other_entity
            || current_entity >= other_entity
        {
            return;
        }
        let current_radius = WgslVecInput::vec_val::<Radius>(current_entity);
        let other_radius = WgslVecInput::vec_val::<Radius>(other_entity);
        if current_radius <= 0.0 || other_radius <= 0.0 {
            return;
        }
        let current_pos = WgslVecInput::vec_val::<Position>(current_entity);
        let other_pos = WgslVecInput::vec_val::<Position>(other_entity);
        let dist_squared = calculate_distance_squared(current_pos, other_pos);
        let radius_sum = current_radius + other_radius;
        // Compare squared distances to avoid sqrt
        if dist_squared < radius_sum * radius_sum {
            WgslOutput::push::<CollisionResult>(CollisionResult {
                entity1: current_entity,
                entity2: other_entity,
            });
        }
    }
}

// Each input/output needs a binding number, as well as appropriate handling for pipeline consts
// will need to be able to change the library-generated pipeline const values (array lengths, etc.) in the code
//automatically determine which outputs need atomic counters based on if a push statement is used for them

fn main() {
    // User can test the Rust version directly
    let shader = collision_shader::parsed();
    // let shader = collision_shader::create_pipeline();
    // Can also get WGSL version
    // let wgsl = collision_shader::as_wgsl();
    let p = collision_shader::parsed();

    // println!("{:?} string", p.helper_types);
}
