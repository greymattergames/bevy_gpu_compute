pub struct WgslGlobalId {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

pub fn user_code_example_defining_wgsl_module() -> String {
    //todo, rust structs in this form: struct Example (f32), are not valid in wgsl, they need to be type aliases instead, example: type Example = f32; This needs to be communicated to the user
    //* BECAUSE TYPES HAVE NOT BEEN CALCULATED WHEN THE MACRO RUNS, SO STRUCTS HAVE TO BE DEFINED WITHIN THE MACRO */
    // #[derive(WgslInputArray)]

    // needs to convert itself
    trait WgslInputArray {
        fn to_wgsl_type_defs() -> String;
        fn to_wgsl_initialization() -> String;
        pub fn new() -> Self;
    }
    #[derive(WgslInputArray)]
    struct Position([f32; 2]);

    struct WgslType {
        name: String,
        lower_name: String,
        upper_name: String,
    }
    impl WgslType {
        fn new(name: &str) -> Self {
            let lower_name = name.to_lowercase();
            let upper_name = name.to_uppercase();
            Self {
                name: name.to_string(),
                lower_name,
                upper_name,
            }
        }
    }

    fn get_type_name<T>() -> WgslType {
        let name = type_name::<T>();
        let clean_name = name.split("::").last().unwrap_or(name);
        return WgslType::new(clean_name);
    }

    fn output_vec_push(t: WgslType, wgsl_string: T) -> String {
        let str = format!(
            "let index = atomicAdd(&{name}_counter, 1u);
            if index < {name_caps}_ARRAY_SIZE{
                {name}_array[index] = {value};
            }
        }",
            name = t.lower_name,
            name_caps = t.upper_name,
            value = wgsl_string
        );
        return str;
    }

    type Radius = f32;

    // define_input_array_types([
    //     Position: {x: f32, y: f32},
    //     Radius: f32,
    //     TestType: {v: u32, r: Radius}, // need to be able to reference custom types
    //     ]);
    //     define_output_vector_types([
    //         ExampleOut: { r: u32}
    //     ])
    //     define_output_array_types([
    //     CollisionResult: { entity1: u32, entity2: u32, test_field: Radius},//need to be able to reference them accross macros too
    //     ])

    //     define_helper_functions([
    //         (fn test_func(v: TestType, p1: Position, p2: Position)-> Position {
    //             return v + p1 + p2;
    //         }),
    //         (fn test_func2(r: Radius)-> u32 {
    //             let r2 = r * test_func(TestType{ v: 3, r: 3.3}, Position {x: 1., y: 1.}, Position {x: 2., y:2.});
    //             return u32(r2)
    //         })
    //     ]);

    //     define_main_function(fn main(global_id: GlobalId)){
    //         let x = global_id.x;
    //         let y = global_id.y;
    //         let position_max = input_vec_len!(Position); -> POSITION_ARRAY_LENGTH;
    //         let position = input_vec_val!(Radius, y); -> radius_array[y]
    //         output_vec_push!(ExampleOut,ExampleOut { r: 3}); ->
    //         // index = counter ...etc.
    //         // if index < EXAMPLEOUT_ARRAY_LENGTH
    //         // exampleout_array[index] = ExampleOut(3);
    //         output_vec_set!(CollisionResult, 0,x);
    //         // collisionresult_array[0] = x;
    //     }
    // #[allow dead_code]
    #[allow(dead_code)]
    let example_module_const: u32 = 0; //value DOES matter, as it is never overwritten
    #[repr(C)]
    #[derive(Debug, Clone, Copy, Zeroable, Pod, WgslUniform)]
    // only one group of uniforms because this library is designed for simple compute shaders
    struct Uniforms {
        time: f32,
        resolution: vec2<f32>,
    }
    type Radius = f32;

    #[repr(C)]
    #[derive(Debug, Clone, Copy, Zeroable, Pod)]
    struct CollisionResult {
        entity1: u32,
        entity2: u32,
    }
    let consts_wgsl: &str = wgsl_consts!([example_module_const]);
    let in_vector_types_wgsl: &str = wgsl_in_vector_types!([Position, Radius]);
    let out_vector_types_wgsl: &str = wgsl_out_vector_types!([CollisionResult]);

    // Optimized distance calculation
    fn calculate_distance_squared(p1: Position, p2: Position) -> f32 {
        let dx = p1[0] - p2[0];
        let dy = p1[1] - p2[1];
        return dx * dx + dy * dy;
    }
    let helpers_wgsl: &str = wgsl_helpers!([calculate_distance_squared]);
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

        let current_radius = wgsl_input_vec_val_at_index!(Radius, current_entity);
        let other_radius = wgsl_input_vec_val_at_index!(Radius, other_entity);
        if current_radius <= 0.0 || other_radius <= 0.0 {
            return;
        }
        let current_pos = wgsl_input_vec_val_at_index!(Position, current_entity);
        let other_pos = wgsl_input_vec_val_at_index!(Position, other_entity);

        let dist_squared = calculate_distance_squared(current_pos, other_pos);
        let radius_sum = current_radius + other_radius;
        let t = WgslType::new("CollisionResult");
        // Compare squared distances to avoid sqrt
        if dist_squared < radius_sum * radius_sum {
            let defined_outside = CollisionResult {
                entity1: current_entity as u32,
                entity2: other_entity as u32,
            };
            output_vec_push(t, rust_to_wgsl! {CollisionResult {
                entity1: current_entity as u32,
                entity2: other_entity as u32,
            }});
            output_vec_push(t, rust_to_wgsl! {defined_outside});
        }
    }
    let main_wgsl: &str = wgsl_main!(main);
    let wgsl_code = format!(
        "{}\n{}\n{}\n{}\n{}",
        consts_wgsl, in_vector_types_wgsl, out_vector_types_wgsl, helpers_wgsl, main_wgsl
    );
    wgsl_code
}

// Example usage
fn main() {
    // Using literal values
    let v1 = Vec2::new(1.0, 2.0);
    println!("{}", v1.to_wgsl_init()); // Output: vec2(1, 2)

    // Using named variables
    let x = WGSLVar::with_name(3.0, "position_x");
    let y = WGSLVar::with_name(4.0, "height");

    let z = WGSLVar::with_name(x + y, "width");
    let v2 = Vec2::new(x, y);
    println!("{}", v2.to_wgsl_init()); // Output: vec2(position_x, height)
}
