#[cfg(test)]
mod tests {
    use crate::{
        state::ModuleTransformState,
        transformer::{
            allowed_types::AllowedRustTypes,
            custom_types::custom_type::{CustomType, CustomTypeKind},
            transform_wgsl_helper_methods::run::transform_wgsl_helper_methods,
        },
    };

    use super::*;
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
            "mod test { fn example () { let x = POSITION_INPUT_ARRAY_LENGTH ; } }";
        let mut state = ModuleTransformState::empty(input);
        let custom_types = vec![CustomType::new(
            &format_ident!("Position"),
            CustomTypeKind::InputArray,
            TokenStream::new(),
        )];
        state.allowed_types = Some(AllowedRustTypes::new(custom_types));
        transform_wgsl_helper_methods(&mut state);
        let result = state.rust_module.to_token_stream().to_string();
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
        let expected_output = "mod test { fn example () { let x = radius_input_array [5] ; } }";

        let mut state = ModuleTransformState::empty(input);
        let custom_types = vec![CustomType::new(
            &format_ident!("Radius"),
            CustomTypeKind::InputArray,
            TokenStream::new(),
        )];
        state.allowed_types = Some(AllowedRustTypes::new(custom_types));
        transform_wgsl_helper_methods(&mut state);
        let result = state.rust_module.to_token_stream().to_string();
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

        let expected_output = "mod test { fn example () { { let collisionresult_output_array_index = atomicAdd (& collisionresult_counter , 1u) ; if collisionresult_output_array_index < COLLISIONRESULT_OUTPUT_ARRAY_LENGTH { collisionresult_output_array [collisionresult_output_array_index] = value ; } } ; } }";
        let mut state = ModuleTransformState::empty(input);
        let custom_types = vec![CustomType::new(
            &format_ident!("CollisionResult"),
            CustomTypeKind::OutputVec,
            TokenStream::new(),
        )];
        state.allowed_types = Some(AllowedRustTypes::new(custom_types));
        transform_wgsl_helper_methods(&mut state);
        let result = state.rust_module.to_token_stream().to_string();

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
            "mod test { fn example () { let x = COLLISIONRESULT_OUTPUT_ARRAY_LENGTH ; } }";

        let mut state = ModuleTransformState::empty(input);
        let custom_types = vec![CustomType::new(
            &format_ident!("CollisionResult"),
            CustomTypeKind::OutputVec,
            TokenStream::new(),
        )];
        state.allowed_types = Some(AllowedRustTypes::new(custom_types));
        transform_wgsl_helper_methods(&mut state);
        let result = state.rust_module.to_token_stream().to_string();

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
            "mod test { fn example () { collisionresult_output_array [idx] = val ; } }";

        let mut state = ModuleTransformState::empty(input);
        let custom_types = vec![CustomType::new(
            &format_ident!("CollisionResult"),
            CustomTypeKind::OutputArray,
            TokenStream::new(),
        )];
        state.allowed_types = Some(AllowedRustTypes::new(custom_types));
        transform_wgsl_helper_methods(&mut state);
        let result = state.rust_module.to_token_stream().to_string();

        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }
}
