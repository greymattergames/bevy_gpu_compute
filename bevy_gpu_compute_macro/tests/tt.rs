// pub fn parsed() -> WgslShaderModuleUserPortion {
//     WgslShaderModuleUserPortion {
//         static_consts: [
//             WgslConstAssignment {
//                 code: WgslShaderModuleSectionCode {
//                     rust_code: ("const example_module_const : u32 = 42;")
//                         .to_string(),
//                     wgsl_code: ("const example_module_const : u32 = 42;")
//                         .to_string(),
//                 },
//             },
//         ]
//             .into(),
//         helper_types: [].into(),
//         uniforms: Vec::from([
//             WgslType {
//                 name: ShaderCustomTypeName::new("Config"),
//                 code: WgslShaderModuleSectionCode {
//                     rust_code: ("#[wgsl_config] struct Config { pub example_value : f32, }")
//                         .to_string(),
//                     wgsl_code: ("struct Config { example_value : f32, }")
//                         .to_string(),
//                 },
//             },
//         ]),
//         input_arrays: [
//             WgslInputArray {
//                 item_type: WgslType {
//                     name: ShaderCustomTypeName::new("Position"),
//                     code: WgslShaderModuleSectionCode {
//                         rust_code: ("#[wgsl_input_array] type Position = [f32; 2];")
//                             .to_string(),
//                         wgsl_code: ("alias Position  = array < f32, 2 > ;")
//                             .to_string(),
//                     },
//                 },
//             },
//             WgslInputArray {
//                 item_type: WgslType {
//                     name: ShaderCustomTypeName::new("Radius"),
//                     code: WgslShaderModuleSectionCode {
//                         rust_code: ("#[wgsl_input_array] type Radius = f32;")
//                             .to_string(),
//                         wgsl_code: ("alias Radius  = f32;").to_string(),
//                     },
//                 },
//             },
//         ]
//             .into(),
//         output_arrays: [
//             WgslOutputArray {
//                 item_type: WgslType {
//                     name: ShaderCustomTypeName::new("CollisionResult"),
//                     code: WgslShaderModuleSectionCode {
//                         rust_code: ("#[wgsl_output_vec] struct CollisionResult { entity1 : u32, entity2 : u32, }")
//                             .to_string(),
//                         wgsl_code: ("struct CollisionResult { entity1 : u32, entity2 : u32, }")
//                             .to_string(),
//                     },
//                 },
//                 atomic_counter_name: Some("collisionresult_counter".to_string()),
//             },
//         ]
//             .into(),
//         helper_functions: [
//             WgslFunction {
//                 name: ("calculate_distance_squared").to_string(),
//                 code: WgslShaderModuleSectionCode {
//                     rust_code: ("pub fn calculate_distance_squared(p1 : [f32; 2], p2 : [f32; 2]) -> f32\n{\n    let dx = p1 [0] - p2 [0]; let dy = p1 [1] - p2 [1]; return dx * dx + dy *\n    dy;\n}")
//                         .to_string(),
//                     wgsl_code: ("pub fn\ncalculate_distance_squared(p1 : array < f32, 2 > , p2 : array < f32, 2 >) ->\nf32\n{\n    let dx = p1 [0] - p2 [0]; let dy = p1 [1] - p2 [1]; return dx * dx + dy *\n    dy;\n}")
//                         .to_string(),
//                 },
//             },
//         ]
//             .into(),
//         main_function: Some(WgslFunction {
//             name: ("main").to_string(),
//             code: WgslShaderModuleSectionCode {
//                 rust_code: ("pub fn main(iter_pos : WgslIterationPosition)\n{\n    let current_entity = iter_pos.x; let other_entity = iter_pos.y; if\n    current_entity >= POSITION_INPUT_ARRAY_LENGTH || other_entity >=\n    POSITION_INPUT_ARRAY_LENGTH || current_entity == other_entity ||\n    current_entity >= other_entity { return; } let current_radius =\n    radius_input_array [current_entity] + example_module_const as f32; let\n    other_radius = radius_input_array [other_entity] + config.example_value as\n    f32; if current_radius <= 0.0 || other_radius <= 0.0 { return; } let\n    current_pos = position_input_array [current_entity]; let other_pos =\n    position_input_array [other_entity]; let dist_squared =\n    calculate_distance_squared(current_pos, other_pos); let radius_sum =\n    current_radius + other_radius; if dist_squared < radius_sum * radius_sum\n    {\n        {\n            let collisionresult_output_array_index =\n            atomicAdd(& collisionresult_counter, 1u); if\n            collisionresult_output_array_index <\n            COLLISIONRESULT_OUTPUT_ARRAY_LENGTH\n            {\n                collisionresult_output_array\n                [collisionresult_output_array_index] = CollisionResult\n                { entity1 : current_entity, entity2 : other_entity, };\n            }\n        };\n    }\n}")
//                     .to_string(),
//                 wgsl_code: ("pub fn main(@builtin(global_invocation_id) iter_pos: vec3<u32>)\n{\n    let current_entity = iter_pos.x; let other_entity = iter_pos.y; if\n    current_entity >= POSITION_INPUT_ARRAY_LENGTH || other_entity >=\n    POSITION_INPUT_ARRAY_LENGTH || current_entity == other_entity ||\n    current_entity >= other_entity { return; } let current_radius =\n    radius_input_array [current_entity] + f32(example_module_const); let\n    other_radius = radius_input_array [other_entity] +\n    f32(config.example_value); if current_radius <= 0.0 || other_radius <= 0.0\n    { return; } let current_pos = position_input_array [current_entity]; let\n    other_pos = position_input_array [other_entity]; let dist_squared =\n    calculate_distance_squared(current_pos, other_pos); let radius_sum =\n    current_radius + other_radius; if dist_squared < radius_sum * radius_sum\n    {\n        {\n            let collisionresult_output_array_index =\n            atomicAdd(& collisionresult_counter, 1u); if\n            collisionresult_output_array_index <\n            COLLISIONRESULT_OUTPUT_ARRAY_LENGTH\n            {\n                collisionresult_output_array\n                [collisionresult_output_array_index] =\n                CollisionResult(current_entity, other_entity);\n            }\n        };\n    }\n}")
//                     .to_string(),
//             },
//         }),
//         binding_numbers_by_variable_name: Some(
//             HashMap::from([
//                 ("radius_input_array".to_string(), 3u32),
//                 ("collisionresult_counter".to_string(), 5u32),
//                 ("config".to_string(), 1u32),
//                 ("position_input_array".to_string(), 2u32),
//                 ("collisionresult_output_array".to_string(), 4u32),
//             ]),
//         ),
//     }
// }
// pub mod on_cpu {
//     use super::*;
//     use bevy_gpu_compute_core::wgsl_helpers::*;
//     const example_module_const: u32 = 42;
//     pub fn calculate_distance_squared(p1: [f32; 2], p2: [f32; 2]) -> f32 {
//         let dx = p1[0] - p2[0];
//         let dy = p1[1] - p2[1];
//         return dx * dx + dy * dy;
//     }
//     pub fn main(
//         iter_pos: WgslIterationPosition,
//         config: Config,
//         position_input_array: Vec<Position>,
//         radius_input_array: Vec<Radius>,
//         mut collisionresult_output_array: &mut Vec<CollisionResult>,
//     ) {
//         let COLLISIONRESULT_OUTPUT_ARRAY_LENGTH = collisionresult_output_array.len();
//         let RADIUS_INPUT_ARRAY_LENGTH = radius_input_array.len();
//         let POSITION_INPUT_ARRAY_LENGTH = position_input_array.len();
//         let current_entity = iter_pos.x;
//         let other_entity = iter_pos.y;
//         if current_entity >= position_input_array.len()
//             || other_entity >= position_input_array.len()
//             || current_entity == other_entity
//             || current_entity >= other_entity
//         {
//             return;
//         }
//         let current_radius = radius_input_array[current_entity] + example_module_const as f32;
//         let other_radius = radius_input_array[other_entity] + config.example_value as f32;
//         if current_radius <= 0.0 || other_radius <= 0.0 {
//             return;
//         }
//         let current_pos = position_input_array[current_entity];
//         let other_pos = position_input_array[other_entity];
//         let dist_squared = calculate_distance_squared(current_pos, other_pos);
//         let radius_sum = current_radius + other_radius;
//         if dist_squared < radius_sum * radius_sum {
//             collisionresult_output_array.push(CollisionResult {
//                 entity1: current_entity,
//                 entity2: other_entity,
//             });
//         }
//     }
// }
