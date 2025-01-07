#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens;
    use syn::parse_quote;

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

        let custom_types = vec![String::from("Position")];
        let output = convert_special_helper_functions(&input, custom_types);
        let result = output.to_token_stream().to_string();
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

        let custom_types = vec![String::from("Radius")];
        let output = convert_special_helper_functions(&input, custom_types);
        let result = output.to_token_stream().to_string();
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

        let custom_types = vec![String::from("CollisionResult")];
        let expected_output = "mod test { fn example () { { let collisionresult_output_array_index = atomicAdd (& collisionresult_counter , 1u) ; if collisionresult_output_array_index < COLLISIONRESULT_OUTPUT_ARRAY_LENGTH { collisionresult_output_array [collisionresult_output_array_index] = value ; } } ; } }";
        let output = convert_special_helper_functions(&input, custom_types);
        let result = output.to_token_stream().to_string();
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
        let custom_types = vec![String::from("CollisionResult")];
        let expected_output =
            "mod test { fn example () { let x = COLLISIONRESULT_OUTPUT_ARRAY_LENGTH ; } }";

        let output = convert_special_helper_functions(&input, custom_types);
        let result = output.to_token_stream().to_string();
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
        let custom_types = vec![String::from("CollisionResult")];
        let expected_output =
            "mod test { fn example () { collisionresult_output_array [idx] = val ; } }";

        let output = convert_special_helper_functions(&input, custom_types);
        let result = output.to_token_stream().to_string();
        println!("{}", result);
        assert_eq!(
            result, expected_output,
            "Expected: {}\nGot: {}",
            expected_output, result
        );
    }
}
