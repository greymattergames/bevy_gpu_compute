#[cfg(test)]
mod component_tests {

    use super::*;
    use proc_macro2::TokenStream;
    use quote::{ToTokens, format_ident};
    use rust_to_wgsl::shader_module;
    use shared::wgsl_in_rust_helpers::Vec3Bool;
    use syn::{ItemMod, parse_quote};

    #[test]
    fn test_simple_struct() {
        #[shader_module]
        pub mod test_module {
            use shared::wgsl_in_rust_helpers::WgslGlobalId;

            struct TStruct {
                x: f32,
                y: f32,
            }
            fn main(global_id: WgslGlobalId) {
                return;
            }
        }

        let t2 = test_module::parsed();
        assert!(t2.output_arrays.len() == 0);
        assert!(t2.input_arrays.len() == 0);
        assert!(t2.uniforms.len() == 0);
        assert!(t2.helper_functions.len() == 0);
        assert!(t2.main_function.is_some());
        assert!(t2.helper_types.len() == 1);
        assert_eq!(
            t2.main_function.unwrap().code.wgsl_code,
            "fn main(@builtin(global_invocation_id) global_id: vec3<u32>) { return; }"
        );
    }

    #[test]
    fn test_struct_creation() {
        #[shader_module]
        pub mod test_module {
            use shared::wgsl_in_rust_helpers::WgslGlobalId;

            struct TStruct {
                x: f32,
                y: f32,
            }
            fn main(global_id: WgslGlobalId) {
                let obj = TStruct { x: 1.0, y: 2.0 };
                return;
            }
        }

        let t2 = test_module::parsed();
        assert!(t2.output_arrays.len() == 0);
        assert!(t2.input_arrays.len() == 0);
        assert!(t2.uniforms.len() == 0);
        assert!(t2.helper_functions.len() == 0);
        assert!(t2.main_function.is_some());
        assert!(t2.helper_types.len() == 1);
        assert_eq!(
            t2.main_function.unwrap().code.wgsl_code,
            "fn main(@builtin(global_invocation_id) global_id: vec3<u32>)\n{ let obj = TStruct(1.0, 2.0); return; }"
        );
    }
    #[test]
    fn test_simple_type_transforms() {
        #[shader_module]
        pub mod test_module {
            use shared::wgsl_in_rust_helpers::{WgslGlobalId, *};
            struct TStruct {
                x: f32,
                y: Vec3F32,
            }
            fn main(global_id: WgslGlobalId) {}
        }

        let t2 = test_module::parsed();
        assert!(t2.output_arrays.len() == 0);
        assert!(t2.input_arrays.len() == 0);
        assert!(t2.uniforms.len() == 0);
        assert!(t2.helper_functions.len() == 0);
        assert!(t2.main_function.is_some());
        assert!(t2.helper_types.len() == 1);
        assert_eq!(
            t2.helper_types.first().unwrap().code.wgsl_code,
            "struct TStruct { x : f32, y : vec3 < f32 > , }"
        );
    }
    #[test]
    fn test_struct_creation_with_nested_transforms() {
        #[shader_module]
        pub mod test_module {
            use shared::wgsl_in_rust_helpers::*;

            struct TStruct {
                x: f32,
                y: Vec3F32,
            }
            fn main(global_id: WgslGlobalId) {
                let obj = TStruct {
                    x: 1.0,
                    y: Vec3F32::new(2.0, 3.0, 4.0),
                };
                return;
            }
        }

        let t2 = test_module::parsed();
        assert!(t2.output_arrays.len() == 0);
        assert!(t2.input_arrays.len() == 0);
        assert!(t2.uniforms.len() == 0);
        assert!(t2.helper_functions.len() == 0);
        assert!(t2.main_function.is_some());
        assert!(t2.static_consts.len() == 0);

        assert!(t2.helper_types.len() == 1);
        assert_eq!(
            t2.main_function.unwrap().code.wgsl_code,
            "fn main(@builtin(global_invocation_id) global_id: vec3<u32>)\n{ let obj = TStruct(1.0,vec3<f32>(2.0, 3.0, 4.0)); return; }"
        );
    }
    #[test]
    fn test_type_alias() {
        #[shader_module]
        pub mod test_module {
            use shared::wgsl_in_rust_helpers::*;
            type MyType = i32;
            fn main(global_id: WgslGlobalId) {}
        }

        let t2 = test_module::parsed();
        assert!(t2.output_arrays.len() == 0);
        assert!(t2.input_arrays.len() == 0);
        assert!(t2.uniforms.len() == 0);
        assert!(t2.helper_functions.len() == 0);
        assert!(t2.main_function.is_some());
        assert!(t2.helper_types.len() == 1);
        assert_eq!(
            t2.helper_types.first().unwrap().code.wgsl_code,
            "alias MyType  = i32;"
        );
    }
    #[test]
    fn test_consts() {
        let t = Vec3Bool::new(true, false, true);
        #[shader_module]
        pub mod test_module {
            use shared::wgsl_in_rust_helpers::{WgslGlobalId, *};
            const MY_CONST: i32 = 3;
            fn main(global_id: WgslGlobalId) {}
        }

        let t2 = test_module::parsed();
        assert!(t2.output_arrays.len() == 0);
        assert!(t2.input_arrays.len() == 0);
        assert!(t2.uniforms.len() == 0);
        assert!(t2.helper_functions.len() == 0);
        assert!(t2.main_function.is_some());
        assert!(t2.static_consts.len() == 1);
        assert!(t2.helper_types.len() == 0);
        assert_eq!(
            t2.static_consts.first().unwrap().code.wgsl_code,
            "const MY_CONST : i32 = 3;"
        );
    }
    #[test]
    fn test_uniforms() {
        #[shader_module]
        pub mod test_module {
            use rust_to_wgsl::wgsl_config;
            use shared::wgsl_in_rust_helpers::*;
            #[wgsl_config]
            struct Uniforms {
                time: f32,
                resolution: Vec2F32,
            }
            fn main(global_id: WgslGlobalId) {}
        }
        let t2 = test_module::parsed();
        assert!(t2.output_arrays.len() == 0);
        assert!(t2.input_arrays.len() == 0);
        assert!(t2.uniforms.len() == 1);
        assert!(t2.helper_functions.len() == 0);
        assert!(t2.main_function.is_some());
        assert!(t2.static_consts.len() == 0);
        assert!(t2.helper_types.len() == 0);
        assert_eq!(
            t2.uniforms.first().unwrap().code.wgsl_code,
            "struct Uniforms { time : f32, resolution : vec2 < f32 > , }"
        );
    }
    #[test]
    fn test_input_arrays() {
        #[shader_module]
        pub mod test_module {
            use rust_to_wgsl::wgsl_input_array;
            use shared::wgsl_in_rust_helpers::*;
            #[wgsl_input_array]
            type Position = [f32; 2];
            fn main(global_id: WgslGlobalId) {}
        }
        let t2 = test_module::parsed();
        assert!(t2.output_arrays.len() == 0);
        assert!(t2.input_arrays.len() == 1);
        assert!(t2.uniforms.len() == 0);
        // type Position = array<f32, 2>;
        assert!(t2.helper_functions.len() == 0);
        assert!(t2.main_function.is_some());
        assert!(t2.static_consts.len() == 0);
        assert!(t2.helper_types.len() == 0);
        assert_eq!(
            t2.input_arrays.first().unwrap().array_type.code.wgsl_code,
            "alias position_input_array  = array < Position, POSITION_INPUT_ARRAY_LENGTH > ;"
        );
        assert_eq!(
            t2.input_arrays.first().unwrap().item_type.code.wgsl_code,
            "alias Position  = array < f32, 2 > ;"
        )
    }

    #[test]
    fn test_output_arrays() {
        #[shader_module]
        pub mod test_module {
            use rust_to_wgsl::wgsl_output_array;
            use shared::wgsl_in_rust_helpers::*;
            #[wgsl_output_array]
            struct CollisionResult {
                entity1: u32,
                entity2: u32,
            }
            fn main(global_id: WgslGlobalId) {}
        }
        let t2 = test_module::parsed();
        assert!(t2.output_arrays.len() == 1);
        assert!(t2.input_arrays.len() == 0);
        assert!(t2.uniforms.len() == 0);
        assert!(t2.helper_functions.len() == 0);
        assert!(t2.main_function.is_some());
        assert!(t2.static_consts.len() == 0);
        assert!(t2.helper_types.len() == 0);
        assert_eq!(
            t2.output_arrays.first().unwrap().item_type.code.wgsl_code,
            "struct CollisionResult { entity1 : u32, entity2 : u32, }"
        );
        assert_eq!(
            t2.output_arrays.first().unwrap().array_type.code.wgsl_code,
            "alias collisionresult_output_array  = array < CollisionResult,
COLLISIONRESULT_OUTPUT_ARRAY_LENGTH > ;"
        );
        assert!(
            t2.output_arrays
                .first()
                .unwrap()
                .atomic_counter_type
                .is_none()
        );
    }
    #[test]
    fn test_output_vec() {
        #[shader_module]
        pub mod test_module {
            use rust_to_wgsl::wgsl_output_vec;
            use shared::wgsl_in_rust_helpers::*;
            #[wgsl_output_vec]
            struct CollisionResult {
                entity1: u32,
                entity2: u32,
            }
            fn main(global_id: WgslGlobalId) {}
        }
        let t2 = test_module::parsed();
        assert!(t2.output_arrays.len() == 1);
        assert!(t2.input_arrays.len() == 0);
        assert!(t2.uniforms.len() == 0);
        assert!(t2.helper_functions.len() == 0);
        assert!(t2.main_function.is_some());
        assert!(t2.static_consts.len() == 0);
        assert!(t2.helper_types.len() == 0);
        assert_eq!(
            t2.output_arrays.first().unwrap().item_type.code.wgsl_code,
            "struct CollisionResult { entity1 : u32, entity2 : u32, }"
        );
        assert_eq!(
            t2.output_arrays.first().unwrap().array_type.code.wgsl_code,
            "alias collisionresult_output_array  = array < CollisionResult,
COLLISIONRESULT_OUTPUT_ARRAY_LENGTH > ;"
        );
        assert!(
            t2.output_arrays
                .first()
                .unwrap()
                .atomic_counter_type
                .is_some()
        );
        assert_eq!(
            t2.output_arrays
                .first()
                .unwrap()
                .atomic_counter_type
                .as_ref()
                .unwrap()
                .code
                .wgsl_code,
            "alias collisionresult_counter  = atomic < u32 > ;"
        )
    }
    #[test]
    fn test_helper_functions() {
        #[shader_module]
        pub mod test_module {
            use shared::wgsl_in_rust_helpers::*;
            fn calculate_distance_squared(p1: [f32; 2], p2: [f32; 2]) -> f32 {
                let dx = p1[0] - p2[0];
                let dy = p1[1] - p2[1];
                return dx * dx + dy * dy;
            }
            fn main(global_id: WgslGlobalId) {}
        }
        let t2 = test_module::parsed();
        assert!(t2.output_arrays.len() == 0);
        assert!(t2.input_arrays.len() == 0);
        assert!(t2.uniforms.len() == 0);
        assert!(t2.helper_functions.len() == 1);
        assert!(t2.main_function.is_some());
        assert!(t2.static_consts.len() == 0);
        assert!(t2.helper_types.len() == 0);
        assert_eq!(
            t2.helper_functions.first().unwrap().code.wgsl_code,
            "fn calculate_distance_squared(p1 : array < f32, 2 > , p2 : array < f32, 2 >)\n-> f32\n{\n    let dx = p1 [0] - p2 [0]; let dy = p1 [1] - p2 [1]; return dx * dx + dy *\n    dy;\n}"
        );
    }
}
