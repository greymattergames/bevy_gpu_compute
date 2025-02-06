#[cfg(test)]
mod tests {
    use crate::{
        state::ModuleTransformState,
        transformer::{
            custom_types::custom_type::{CustomType, CustomTypeKind},
            transform_wgsl_helper_methods_for_cpu::run::transform_wgsl_helper_methods_for_cpu,
        },
    };

    use proc_macro2::TokenStream;
    use quote::{ToTokens, format_ident};
    use syn::{ItemMod, parse_quote};

    #[test]
    fn test_vec_len() {
        let input: ItemMod = parse_quote! {
            mod test {
                fn example() {
                    let x = WgslVecInput::vec_len::<Position>();
                }
            }
        };
        let expected_output =
            "mod test { fn example () { let x = position_input_array . len () as u32 ; } }";
        let mut state = ModuleTransformState::empty(input, "".to_string());
        let custom_types = vec![CustomType::new(
            &format_ident!("Position"),
            CustomTypeKind::InputArray,
            TokenStream::new(),
        )];
        state.custom_types = Some(custom_types);
        transform_wgsl_helper_methods_for_cpu(&mut state);
        let result = state.rust_module_for_cpu.to_token_stream().to_string();
        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }

    #[test]
    fn test_vec_val() {
        let input: ItemMod = parse_quote! {
            mod test {

                fn example() {
                    let x = WgslVecInput::vec_val::<Radius>(5);
                }
            }
        };
        let expected_output =
            "mod test { fn example () { let x = radius_input_array [5 as usize] ; } }";

        let mut state = ModuleTransformState::empty(input, "".to_string());
        let custom_types = vec![CustomType::new(
            &format_ident!("Radius"),
            CustomTypeKind::InputArray,
            TokenStream::new(),
        )];
        state.custom_types = Some(custom_types);
        transform_wgsl_helper_methods_for_cpu(&mut state);
        let result = state.rust_module_for_cpu.to_token_stream().to_string();
        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }

    #[test]
    fn test_push() {
        let input: ItemMod = parse_quote! {
            mod test {
                fn example() {
                    WgslOutput::push::<CollisionResult>(value);
                }
            }
        };

        let expected_output =
            "mod test { fn example () { collisionresult_output_array . push (value) ; } }";
        let mut state = ModuleTransformState::empty(input, "".to_string());
        let custom_types = vec![CustomType::new(
            &format_ident!("CollisionResult"),
            CustomTypeKind::OutputVec,
            TokenStream::new(),
        )];
        state.custom_types = Some(custom_types);
        transform_wgsl_helper_methods_for_cpu(&mut state);
        let result = state.rust_module_for_cpu.to_token_stream().to_string();

        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }

    #[test]
    fn test_output_max_len() {
        let input: ItemMod = parse_quote! {
            mod test {
                fn example() {
                    let x = WgslOutput::max_len::<CollisionResult>();
                }
            }
        };
        let expected_output =
            "mod test { fn example () { let x = collisionresult_output_array . len () as u32 ; } }";

        let mut state = ModuleTransformState::empty(input, "".to_string());
        let custom_types = vec![CustomType::new(
            &format_ident!("CollisionResult"),
            CustomTypeKind::OutputVec,
            TokenStream::new(),
        )];
        state.custom_types = Some(custom_types);
        transform_wgsl_helper_methods_for_cpu(&mut state);
        let result = state.rust_module_for_cpu.to_token_stream().to_string();

        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }

    #[test]
    fn test_output_len() {
        let input: ItemMod = parse_quote! {
            mod test {
                fn example() {
                    let x = WgslOutput::len::<CollisionResult>();
                }
            }
        };
        let expected_output =
            "mod test { fn example () { let x = collisionresult_output_array . len () as u32 ; } }";

        let mut state = ModuleTransformState::empty(input, "".to_string());
        let custom_types = vec![CustomType::new(
            &format_ident!("CollisionResult"),
            CustomTypeKind::OutputVec,
            TokenStream::new(),
        )];
        state.custom_types = Some(custom_types);
        transform_wgsl_helper_methods_for_cpu(&mut state);
        let result = state.rust_module_for_cpu.to_token_stream().to_string();

        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }

    #[test]
    fn test_output_set() {
        let input: ItemMod = parse_quote! {
            mod test {
                fn example() {
                    WgslOutput::set::<CollisionResult>(idx, val);
                }
            }
        };
        let expected_output =
            "mod test { fn example () { collisionresult_output_array [idx as usize] = val ; } }";

        let mut state = ModuleTransformState::empty(input, "".to_string());
        let custom_types = vec![CustomType::new(
            &format_ident!("CollisionResult"),
            CustomTypeKind::OutputArray,
            TokenStream::new(),
        )];
        state.custom_types = Some(custom_types);
        transform_wgsl_helper_methods_for_cpu(&mut state);
        let result = state.rust_module_for_cpu.to_token_stream().to_string();

        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }
    #[test]
    fn test_config_get() {
        let input: ItemMod = parse_quote! {
            mod test {
                fn example() {
                    let t = WgslConfigInput::get::<Position>();
                }
            }
        };
        let expected_output = "mod test { fn example () { let t = position ; } }";

        let mut state = ModuleTransformState::empty(input, "".to_string());
        let custom_types = vec![CustomType::new(
            &format_ident!("Position"),
            CustomTypeKind::Uniform,
            TokenStream::new(),
        )];
        state.custom_types = Some(custom_types);
        transform_wgsl_helper_methods_for_cpu(&mut state);
        let result = state.rust_module_for_cpu.to_token_stream().to_string();

        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }
}
