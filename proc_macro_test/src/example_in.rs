pub struct WgslGlobalId {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

pub fn user_code() {
    const module_const: u32 = 0; //value DOES matter, as it is never overwritten
    #[repr(C)]
    #[derive(Debug, Clone, Copy, Zeroable, Pod, WgslPipelineConst)]
    struct PipelineConst(pub u32);
    #[repr(C)]
    #[derive(Debug, Clone, Copy, Zeroable, Pod, WgslUniform)]
    struct Uniforms {
        time: f32,
        resolution: vec2<f32>,
    }
    #[repr(C)]
    #[derive(Debug, Clone, Copy, WgslInputVector)]
    struct Position(pub [f32; 2]);
    #[repr(C)]
    #[derive(Debug, Clone, Copy, Zeroable, Pod, WgslInputVector)]
    struct Radius(pub f32);

    #[repr(C)]
    #[derive(Debug, Clone, Copy, Zeroable, Pod, WgslOutputVector)]
    struct CollisionResult {
        entity1: u32,
        entity2: u32,
    }
    #[derive(WgslHelperFunction)]
    // Optimized distance calculation
    fn calculate_distance_squared(p1: [f32; 2], p2: [f32; 2]) -> f32 {
        let dx = p1[0] - p2[0];
        let dy = p1[1] - p2[1];
        return dx * dx + dy * dy;
    }
    #[derive(WgslMainFunction)]
    fn main(global_id: WgslGlobalId) {
        let current_entity = global_id.x;
        let other_entity = global_id.y;
        // Early exit if invalid entity or zero radius
        if current_entity >= wgsl_input_vec_len_const!(Position) //slightly more performant for user to refer to the const directly like this rather than putting it in a variable
            || other_entity >= wgsl_input_vec_len_const!(Position)
            || current_entity == other_entity
            || current_entity >= other_entity
        {
            return;
        }

        let current_radius: Radius = wgsl_input_vec_val_at_index!(current_entity);
        let other_radius: Radius = wgsl_input_vec_val_at_index!(Radius, other_entity);
        if current_radius <= 0.0 || other_radius <= 0.0 {
            return;
        }
        let current_pos: Position = wgsl_input_vec_val_at_index!(current_entity);
        let other_pos: Position = wgsl_input_vec_val_at_index!(other_entity);

        let dist_squared = calculate_distance_squared(current_pos, other_pos);
        let radius_sum = current_radius + other_radius;

        // Compare squared distances to avoid sqrt
        if dist_squared < radius_sum * radius_sum {
            let index = wgsl_output_vec_counter_next!(CollisionResult);
            if index < wgsl_output_vec_len_const!(CollisionResult) {
                let result = CollisionResult {
                    entity1: current_entity,
                    entity2: other_entity,
                };
                wgsl_output_vec_write_to_index!(result, index);
            }
        }
    }
}

pub fn library_code() {
    // do not worry about HOW we get these values, just worry about converting them into correct WGSL code
    create_binding()
}
