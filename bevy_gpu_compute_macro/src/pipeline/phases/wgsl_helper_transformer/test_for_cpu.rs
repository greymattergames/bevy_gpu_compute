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
                fn example() {
                    let x = WgslVecInput::vec_len::<Position>();
                }
            }
        };
        let expected_output =
            "mod test { fn example () { let x = position_input_array . len () as u32 ; } }";
        let custom_types = vec![CustomType::new(
            &format_ident!("Position"),
            CustomTypeKind::InputArray,
            TokenStream::new(),
        )];
        transform_wgsl_helper_methods(&custom_types, &mut input, true);
        let result = input.to_token_stream().to_string();
        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }

    #[test]
    fn test_vec_val() {
        let mut input: ItemMod = parse_quote! {
            mod test {

                fn example() {
                    let x = WgslVecInput::vec_val::<Radius>(5);
                }
            }
        };
        let expected_output =
            "mod test { fn example () { let x = radius_input_array [5 as usize] ; } }";

        let custom_types = vec![CustomType::new(
            &format_ident!("Radius"),
            CustomTypeKind::InputArray,
            TokenStream::new(),
        )];

        transform_wgsl_helper_methods(&custom_types, &mut input, true);
        let result = input.to_token_stream().to_string();
        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }

    #[test]
    fn test_push() {
        let mut input: ItemMod = parse_quote! {
            mod test {
                fn example() {
                    WgslOutput::push::<CollisionResult>(value);
                }
            }
        };

        let expected_output =
            "mod test { fn example () { collisionresult_output_array . push (value) ; } }";

        let custom_types = vec![CustomType::new(
            &format_ident!("CollisionResult"),
            CustomTypeKind::OutputVec,
            TokenStream::new(),
        )];

        transform_wgsl_helper_methods(&custom_types, &mut input, true);
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
            "mod test { fn example () { let x = collisionresult_output_array . len () as u32 ; } }";

        let custom_types = vec![CustomType::new(
            &format_ident!("CollisionResult"),
            CustomTypeKind::OutputVec,
            TokenStream::new(),
        )];

        transform_wgsl_helper_methods(&custom_types, &mut input, true);
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
        let expected_output =
            "mod test { fn example () { let x = collisionresult_output_array . len () as u32 ; } }";

        let custom_types = vec![CustomType::new(
            &format_ident!("CollisionResult"),
            CustomTypeKind::OutputVec,
            TokenStream::new(),
        )];

        transform_wgsl_helper_methods(&custom_types, &mut input, true);
        let result = input.to_token_stream().to_string();

        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }

    #[test]
    fn test_output_set() {
        let mut input: ItemMod = parse_quote! {
            mod test {
                fn example() {
                    WgslOutput::set::<CollisionResult>(idx, val);
                }
            }
        };
        let expected_output =
            "mod test { fn example () { collisionresult_output_array [idx as usize] = val ; } }";

        let custom_types = vec![CustomType::new(
            &format_ident!("CollisionResult"),
            CustomTypeKind::OutputArray,
            TokenStream::new(),
        )];

        transform_wgsl_helper_methods(&custom_types, &mut input, true);
        let result = input.to_token_stream().to_string();

        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }
    #[test]
    fn test_config_get() {
        let mut input: ItemMod = parse_quote! {
            mod test {
                fn example() {
                    let t = WgslConfigInput::get::<Position>();
                }
            }
        };
        let expected_output = "mod test { fn example () { let t = position ; } }";

        let custom_types = vec![CustomType::new(
            &format_ident!("Position"),
            CustomTypeKind::Uniform,
            TokenStream::new(),
        )];

        transform_wgsl_helper_methods(&custom_types, &mut input, true);
        let result = input.to_token_stream().to_string();

        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }
}
