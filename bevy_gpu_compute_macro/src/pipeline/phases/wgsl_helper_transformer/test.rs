#[cfg(test)]
mod tests {
    use crate::pipeline::phases::{
        custom_type_collector::custom_type::{CustomType, CustomTypeKind},
        wgsl_helper_transformer::run::transform_wgsl_helper_methods,
    };

    use proc_macro2::TokenStream;
    use quote::{ToTokens, format_ident};
    use syn::{ItemMod, parse_quote};

    #[test]
    fn test_vec_len() {
        let mut input: ItemMod = parse_quote! {
            mod test {
                fn main() {
                    let x = WgslVecInput::vec_len::<Position>();
                }
            }
        };
        let expected_output = "mod test { fn main () { let x = POSITION_INPUT_ARRAY_LENGTH ; } }";

        let custom_types = vec![CustomType::new(
            &format_ident!("Position"),
            CustomTypeKind::InputArray,
            TokenStream::new(),
        )];

        transform_wgsl_helper_methods(&custom_types, &mut input, false);
        let result = input.to_token_stream().to_string();
        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }

    #[test]
    #[should_panic(
        expected = "WGSL helpers that read from inputs or write to outputs (`bevy_gpu_compute_core::wgsl_helpers`) can only be used inside the main function. It is technically possible to pass in entire input arrays, configs, or output arrays to helper functions, but considering the performance implications, it is not recommended. Instead interact with your inputs and outputs in the main function and pass in only the necessary data to the helper functions."
    )]
    fn test_vec_val_only_in_main() {
        let mut input: ItemMod = parse_quote! {
            mod test {

                fn example() {
                    let x = WgslVecInput::vec_val::<Radius>(5);
                }
            }
        };

        let custom_types = vec![CustomType::new(
            &format_ident!("Radius"),
            CustomTypeKind::InputArray,
            TokenStream::new(),
        )];

        transform_wgsl_helper_methods(&custom_types, &mut input, false);
    }

    #[test]
    fn test_vec_val() {
        let mut input: ItemMod = parse_quote! {
            mod test {

                fn main() {
                    let x = WgslVecInput::vec_val::<Radius>(5);
                }
            }
        };
        let expected_output = "mod test { fn main () { let x = radius_input_array [5] ; } }";

        let custom_types = vec![CustomType::new(
            &format_ident!("Radius"),
            CustomTypeKind::InputArray,
            TokenStream::new(),
        )];

        transform_wgsl_helper_methods(&custom_types, &mut input, false);
        let result = input.to_token_stream().to_string();
        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }

    #[test]
    #[should_panic(
        expected = "WGSL helpers that read from inputs or write to outputs (`bevy_gpu_compute_core::wgsl_helpers`) can only be used inside the main function. It is technically possible to pass in entire input arrays, configs, or output arrays to helper functions, but considering the performance implications, it is not recommended. Instead interact with your inputs and outputs in the main function and pass in only the necessary data to the helper functions."
    )]
    fn test_push_only_in_main() {
        let mut input: ItemMod = parse_quote! {
            mod test {
                fn example() {
                    WgslOutput::push::<CollisionResult>(value);
                }
            }
        };

        let custom_types = vec![CustomType::new(
            &format_ident!("CollisionResult"),
            CustomTypeKind::OutputVec,
            TokenStream::new(),
        )];

        transform_wgsl_helper_methods(&custom_types, &mut input, false);
    }
    #[test]
    fn test_push() {
        let mut input: ItemMod = parse_quote! {
            mod test {
                fn main() {
                    WgslOutput::push::<CollisionResult>(value);
                }
            }
        };

        let expected_output = "mod test { fn main () { { let collisionresult_output_array_index = atomicAdd (& collisionresult_counter , 1u) ; if collisionresult_output_array_index < COLLISIONRESULT_OUTPUT_ARRAY_LENGTH { collisionresult_output_array [collisionresult_output_array_index] = value ; } } ; } }";

        let custom_types = vec![CustomType::new(
            &format_ident!("CollisionResult"),
            CustomTypeKind::OutputVec,
            TokenStream::new(),
        )];

        transform_wgsl_helper_methods(&custom_types, &mut input, false);
        let result = input.to_token_stream().to_string();

        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }

    #[test]
    fn test_output_max_len() {
        let mut input: ItemMod = parse_quote! {
            mod test {
                fn example() {
                    let x = WgslOutput::max_len::<CollisionResult>();
                }
            }
        };
        let expected_output =
            "mod test { fn example () { let x = COLLISIONRESULT_OUTPUT_ARRAY_LENGTH ; } }";

        let custom_types = vec![CustomType::new(
            &format_ident!("CollisionResult"),
            CustomTypeKind::OutputVec,
            TokenStream::new(),
        )];

        transform_wgsl_helper_methods(&custom_types, &mut input, false);
        let result = input.to_token_stream().to_string();

        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }

    #[test]
    fn test_output_len() {
        let mut input: ItemMod = parse_quote! {
            mod test {
                fn example() {
                    let x = WgslOutput::len::<CollisionResult>();
                }
            }
        };
        let expected_output = "mod test { fn example () { let x = collisionresult_counter ; } }";

        let custom_types = vec![CustomType::new(
            &format_ident!("CollisionResult"),
            CustomTypeKind::OutputVec,
            TokenStream::new(),
        )];

        transform_wgsl_helper_methods(&custom_types, &mut input, false);
        let result = input.to_token_stream().to_string();

        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }

    #[test]
    #[should_panic(
        expected = "WGSL helpers that read from inputs or write to outputs (`bevy_gpu_compute_core::wgsl_helpers`) can only be used inside the main function. It is technically possible to pass in entire input arrays, configs, or output arrays to helper functions, but considering the performance implications, it is not recommended. Instead interact with your inputs and outputs in the main function and pass in only the necessary data to the helper functions."
    )]
    fn test_output_set_not_in_main() {
        let mut input: ItemMod = parse_quote! {
            mod test {
                fn example() {
                    WgslOutput::set::<CollisionResult>(idx, val);
                }
            }
        };

        let custom_types = vec![CustomType::new(
            &format_ident!("CollisionResult"),
            CustomTypeKind::OutputArray,
            TokenStream::new(),
        )];

        transform_wgsl_helper_methods(&custom_types, &mut input, false);
    }
    #[test]

    fn test_output_set() {
        let mut input: ItemMod = parse_quote! {
            mod test {
                fn main() {
                    WgslOutput::set::<CollisionResult>(idx, val);
                }
            }
        };
        let expected_output =
            "mod test { fn main () { collisionresult_output_array [idx] = val ; } }";

        let custom_types = vec![CustomType::new(
            &format_ident!("CollisionResult"),
            CustomTypeKind::OutputArray,
            TokenStream::new(),
        )];

        transform_wgsl_helper_methods(&custom_types, &mut input, false);
        let result = input.to_token_stream().to_string();

        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }
    #[test]
    #[should_panic(
        expected = "WGSL helpers that read from inputs or write to outputs (`bevy_gpu_compute_core::wgsl_helpers`) can only be used inside the main function. It is technically possible to pass in entire input arrays, configs, or output arrays to helper functions, but considering the performance implications, it is not recommended. Instead interact with your inputs and outputs in the main function and pass in only the necessary data to the helper functions."
    )]
    fn test_config_get_outside_main() {
        let mut input: ItemMod = parse_quote! {
            mod test {
                fn example() {
                    let t = WgslConfigInput::get::<Position>();
                }
            }
        };

        let custom_types = vec![CustomType::new(
            &format_ident!("Position"),
            CustomTypeKind::Uniform,
            TokenStream::new(),
        )];

        transform_wgsl_helper_methods(&custom_types, &mut input, false);
    }
    #[test]
    fn test_config_get() {
        let mut input: ItemMod = parse_quote! {
            mod test {
                fn main() {
                    let t = WgslConfigInput::get::<Position>();
                }
            }
        };
        let expected_output = "mod test { fn main () { let t = position ; } }";

        let custom_types = vec![CustomType::new(
            &format_ident!("Position"),
            CustomTypeKind::Uniform,
            TokenStream::new(),
        )];

        transform_wgsl_helper_methods(&custom_types, &mut input, false);
        let result = input.to_token_stream().to_string();

        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }
}
