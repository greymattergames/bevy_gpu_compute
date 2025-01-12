#[cfg(test)]
mod component_tests {

    use super::*;
    use proc_macro2::TokenStream;
    use quote::{ToTokens, format_ident};
    use rust_to_wgsl::shader_module;
    use shared::wgsl_components::WgslShaderModuleUserPortion;
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
                y: Vec3<f32>,
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
            "struct TStruct { x: f32, y: vec3<f32> }"
        );
    }
    #[test]
    fn test_struct_creation_with_nested_transforms() {
        #[shader_module]
        pub mod test_module {
            use shared::wgsl_in_rust_helpers::*;

            struct TStruct {
                x: f32,
                y: Vec3<f32>,
            }
            fn main(global_id: WgslGlobalId) {
                let obj = TStruct {
                    x: 1.0,
                    y: Vec3 {
                        x: 2.0,
                        y: 3.0,
                        z: 4.0,
                    },
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
        assert!(t2.helper_types.len() == 1);
        assert_eq!(
            t2.main_function.unwrap().code.wgsl_code,
            "fn main(@builtin(global_invocation_id) global_id: vec3<u32>)\n{     let obj = TStruct(1.0, vec3(2.0,3.0,4.0)); return; }"
        );
    }
}
