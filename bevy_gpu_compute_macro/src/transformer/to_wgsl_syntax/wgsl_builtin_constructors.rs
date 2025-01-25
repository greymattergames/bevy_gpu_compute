use regex::Regex;

/**  for converting vec and mat types
*Vectors are declared with the form vecN<T> , where N is the number of elements in the vector, and T is the element type.
* Matrices are declared with the form matCxR<f32> , where C is the number of columns in the matrix, R is the number of rows in the matrix.
* vectors are accesed like arrays ... my_vec = vec3(1.0, 2.0, 3.0); my_vec[0] = 1.0;
* matrices are accesed wierdly: my_mat = mat2x2(vec2(1.0, 2.0), vec2(3.0, 4.0)); my_mat[1,0] = 3.0;

## creation:
 Vec3I32::new(1, 2, 3) => vec3<i32>(1, 2, 3)
 Mat4x2Bool::new(Vec2Bool::new(true, false), Vec2Bool::new(false, true), Vec2Bool::new(true, true), Vec2Bool::new(false, false)) => mat4x2<bool>(vec2<bool>(true, false), vec2<bool>(false, true), vec2<bool>(true, true), vec2<bool>(false, false))

## creation if conversion already happened:
 vec3<i32>::new(1, 2, 3) => vec3<i32>(1, 2, 3)
    mat4x2<bool>::new(vec2<bool>::new(true, false), vec2<bool>::new(false, true), vec2<bool>::new(true, true), vec2<bool>::new(false, false)) => mat4x2<bool>::new(vec2<bool>(true, false), vec2<bool>::new(false, true), vec2<bool>::new(true, true), vec2<bool>::new(false, false))

we have these different type conversions:
 "Vec2I32" => parse_quote!(vec2<i32>),
            "Vec2U32" => parse_quote!(vec3<u32>),
            "Vec2F32" => parse_quote!(vec3<f32>),
            "Vec2F16" => parse_quote!(vec3<f16>),
            "Vec2Bool" => parse_quote!(vec3<bool>),
            "Vec3I32" => parse_quote!(vec3<i32>),
            "Vec3U32" => parse_quote!(vec3<u32>),
            "Vec3F32" => parse_quote!(vec3<f32>),
            "Vec3F16" => parse_quote!(vec3<f16>),
            "Vec3Bool" => parse_quote!(vec3<bool>),
            "Vec4I32" => parse_quote!(vec4<i32>),
            "Vec4U32" => parse_quote!(vec4<u32>),
            "Vec4F32" => parse_quote!(vec4<f32>),
            "Vec4F16" => parse_quote!(vec4<f16>),
            "Vec4Bool" => parse_quote!(vec4<bool>),
            "Mat2x2I32" => parse_quote!(mat2x2<i32>),
            "Mat2x2U32" => parse_quote!(mat2x2<u32>),
            "Mat2x2F32" => parse_quote!(mat2x2<f32>),
            "Mat2x2F16" => parse_quote!(mat2x2<f16>),
            "Mat2x2Bool" => parse_quote!(mat2x2<bool>),
            "Mat2x3I32" => parse_quote!(mat3x3<i32>),
            "Mat2x3U32" => parse_quote!(mat3x3<u32>),
            "Mat2x3F32" => parse_quote!(mat3x3<f32>),
            "Mat2x3F16" => parse_quote!(mat3x3<f16>),
            "Mat2x3Bool" => parse_quote!(mat3x3<bool>),
            "Mat2x4I32" => parse_quote!(mat2x4<i32>),
            "Mat2x4U32" => parse_quote!(mat2x4<u32>),
            "Mat2x4F32" => parse_quote!(mat2x4<f32>),
            "Mat2x4F16" => parse_quote!(mat2x4<f16>),
            "Mat2x4Bool" => parse_quote!(mat2x4<bool>),
            "Mat3x2I32" => parse_quote!(mat3x2<i32>),
            "Mat3x2U32" => parse_quote!(mat3x2<u32>),
            "Mat3x2F32" => parse_quote!(mat3x2<f32>),
            "Mat3x2F16" => parse_quote!(mat3x2<f16>),
            "Mat3x2Bool" => parse_quote!(mat3x2<bool>),
            "Mat3x3I32" => parse_quote!(mat3x3<i32>),
            "Mat3x3U32" => parse_quote!(mat3x3<u32>),
            "Mat3x3F32" => parse_quote!(mat3x3<f32>),
            "Mat3x3F16" => parse_quote!(mat3x3<f16>),
            "Mat3x3Bool" => parse_quote!(mat3x3<bool>),
            "Mat3x4I32" => parse_quote!(mat3x4<i32>),
            "Mat3x4U32" => parse_quote!(mat3x4<u32>),
            "Mat3x4F32" => parse_quote!(mat3x4<f32>),
            "Mat3x4F16" => parse_quote!(mat3x4<f16>),
            "Mat3x4Bool" => parse_quote!(mat3x4<bool>),
            "Mat4x2I32" => parse_quote!(mat4x2<i32>),
            "Mat4x2U32" => parse_quote!(mat4x2<u32>),
            "Mat4x2F32" => parse_quote!(mat4x2<f32>),
            "Mat4x2F16" => parse_quote!(mat4x2<f16>),
            "Mat4x2Bool" => parse_quote!(mat4x2<bool>),
            "Mat4x3I32" => parse_quote!(mat4x3<i32>),
            "Mat4x3U32" => parse_quote!(mat4x3<u32>),
            "Mat4x3F32" => parse_quote!(mat4x3<f32>),
            "Mat4x3F16" => parse_quote!(mat4x3<f16>),
            "Mat4x3Bool" => parse_quote!(mat4x3<bool>),
            "Mat4x4I32" => parse_quote!(mat4x4<i32>),
            "Mat4x4U32" => parse_quote!(mat4x4<u32>),
            "Mat4x4F32" => parse_quote!(mat4x4<f32>),
            "Mat4x4F16" => parse_quote!(mat4x4<f16>),
            "Mat4x4Bool" => parse_quote!(mat4x4<bool>),


# access:
NO CHANGE TO ACCESS


*/
pub fn convert_wgsl_builtin_constructors(wgsl_code: String) -> String {
    process_string_recursively(&wgsl_code)
}

fn convert_single_constructor(input: &str) -> Option<(String, bool)> {
    // Modified regex to handle whitespace around ::
    let re = Regex::new(r"(?:(Vec|Mat)([234])(?:x([234]))?((?:I32|U32|F32|F16|Bool))|(?:vec|mat)([234])(?:x([234]))?\s*<\s*((?:i32|u32|f32|f16|bool))\s*>)\s*::\s*new").unwrap();

    if let Some(caps) = re.captures(input) {
        // Rest of the function remains the same
        if let (Some(type_kind), Some(first_dim), second_dim, Some(type_str)) =
            (caps.get(1), caps.get(2), caps.get(3), caps.get(4))
        {
            let type_str = match type_str.as_str() {
                "I32" => "i32",
                "U32" => "u32",
                "F32" => "f32",
                "F16" => "f16",
                "Bool" => "bool",
                _ => panic!("Unsupported type"),
            };

            let prefix = match type_kind.as_str() {
                "Vec" => format!("vec{}<{}>", first_dim.as_str(), type_str),
                "Mat" => format!(
                    "mat{}x{}<{}>",
                    first_dim.as_str(),
                    second_dim.map_or(first_dim.as_str(), |m| m.as_str()),
                    type_str
                ),
                _ => unreachable!(),
            };
            Some((prefix, true))
        }
        // Handle WGSL format (vec3<f32>)
        else if let (Some(first_dim), second_dim, Some(type_str)) =
            (caps.get(5), caps.get(6), caps.get(7))
        {
            let prefix = if second_dim.is_some() {
                format!(
                    "mat{}x{}<{}>",
                    first_dim.as_str(),
                    second_dim.unwrap().as_str(),
                    type_str.as_str()
                )
            } else {
                format!("vec{}<{}>", first_dim.as_str(), type_str.as_str())
            };
            Some((prefix, true))
        } else {
            None
        }
    } else {
        None
    }
}
fn process_string_recursively(input: &str) -> String {
    let mut result = String::new();
    let mut current_constructor = String::new();
    let mut current_args = String::new();
    let mut paren_depth = 0;
    let mut pos = 0;
    let chars: Vec<char> = input.chars().collect();

    while pos < chars.len() {
        let c = chars[pos];
        match c {
            '(' => {
                paren_depth += 1;
                if paren_depth == 1 {
                    // Process any constructor before the opening parenthesis
                    if !current_constructor.is_empty() {
                        if let Some((prefix, _)) =
                            convert_single_constructor(current_constructor.trim())
                        {
                            result.push_str(&prefix);
                        } else {
                            result.push_str(&current_constructor);
                        }
                        current_constructor.clear();
                    }
                    result.push(c);
                } else {
                    current_args.push(c);
                }
            }
            ')' => {
                if paren_depth == 1 {
                    // Process accumulated arguments
                    if !current_args.is_empty() {
                        let processed = process_string_recursively(&current_args);
                        result.push_str(&processed);
                        current_args.clear();
                    }
                    result.push(c);
                    paren_depth -= 1;
                } else {
                    current_args.push(c);
                    paren_depth -= 1;
                }
            }
            ' ' | '\t' | '\n' => {
                if paren_depth == 0 {
                    // Instead of processing immediately, add the whitespace to the constructor
                    current_constructor.push(c);
                } else {
                    current_args.push(c);
                }
            }
            ',' => {
                if paren_depth == 0 {
                    if !current_constructor.is_empty() {
                        if let Some((prefix, _)) =
                            convert_single_constructor(current_constructor.trim())
                        {
                            result.push_str(&prefix);
                        } else {
                            result.push_str(&current_constructor);
                        }
                        current_constructor.clear();
                    }
                    result.push(c);
                } else {
                    current_args.push(c);
                }
            }
            _ => {
                if paren_depth == 0 {
                    current_constructor.push(c);
                } else {
                    current_args.push(c);
                }
            }
        }
        pos += 1;
    }

    // Handle any remaining content
    if !current_constructor.is_empty() {
        if let Some((prefix, _)) = convert_single_constructor(current_constructor.trim()) {
            result.push_str(&prefix);
        } else {
            result.push_str(&current_constructor);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_conversion() {
        let input = "Vec3F32::new(1.0, 2.0, 3.0)".to_string();
        let expected = "vec3<f32>(1.0, 2.0, 3.0)";
        assert_eq!(convert_wgsl_builtin_constructors(input), expected);
    }

    #[test]
    fn test_vector_conversion_bool() {
        let input = "Vec3Bool::new(false, true, false)".to_string();
        let expected = "vec3<bool>(false, true, false)";
        assert_eq!(convert_wgsl_builtin_constructors(input), expected);
    }

    #[test]
    fn test_matrix_conversion() {
        let input = "Mat2x2F32::new(Vec2F32::new(1.0, 2.0), Vec2F32::new(3.0, 4.0))".to_string();
        let expected = "mat2x2<f32>(vec2<f32>(1.0, 2.0),vec2<f32>(3.0, 4.0))";
        assert_eq!(convert_wgsl_builtin_constructors(input), expected);
    }
    #[test]
    fn test_matrix_conversion2() {
        let input = "Mat3x4Bool::new(Vec4Bool::new(Vec2Bool::new(true, false)))".to_string();
        let expected = "mat3x4<bool>(vec4<bool>(vec2<bool>(true, false)))";
        assert_eq!(convert_wgsl_builtin_constructors(input), expected);
    }
    #[test]
    fn test_already_converted_types() {
        let input = "vec3<f32>::new(1.0, 2.0, 3.0)".to_string();
        let expected = "vec3<f32>(1.0, 2.0, 3.0)";
        assert_eq!(convert_wgsl_builtin_constructors(input), expected);

        let input = "mat2x2<bool>::new(vec2<bool>::new(true, false), vec2<bool>::new(false, true))"
            .to_string();
        let expected = "mat2x2<bool>(vec2<bool>(true, false),vec2<bool>(false, true))";
        assert_eq!(convert_wgsl_builtin_constructors(input), expected);
    }
    #[test]
    fn test_triple_nested_conversion() {
        let input = "Mat3x4Bool::new(Vec4Bool::new(Vec2Bool::new(true, false), Vec2Bool::new(false, true), Vec2Bool::new(true, true), Vec2Bool::new(false, false)), Vec4Bool::new(Vec2Bool::new(true, false), Vec2Bool::new(false, true), Vec2Bool::new(true, true), Vec2Bool::new(false, false)), Vec4Bool::new(Vec2Bool::new(true, false), Vec2Bool::new(false, true), Vec2Bool::new(true, true), Vec2Bool::new(false, false)))"
            .to_string();
        let expected = "mat3x4<bool>(vec4<bool>(vec2<bool>(true, false),vec2<bool>(false, true),vec2<bool>(true, true),vec2<bool>(false, false)),vec4<bool>(vec2<bool>(true, false),vec2<bool>(false, true),vec2<bool>(true, true),vec2<bool>(false, false)),vec4<bool>(vec2<bool>(true, false),vec2<bool>(false, true),vec2<bool>(true, true),vec2<bool>(false, false)))";
        assert_eq!(convert_wgsl_builtin_constructors(input), expected);
    }

    #[test]
    fn test_nested_vector_conversion() {
        let input = "Mat2x2F32::new(Vec2F32::new(1.0, Vec2F32::new(2.0)), Vec2F32::new(3.0, 4.0))"
            .to_string();
        let expected = "mat2x2<f32>(vec2<f32>(1.0,vec2<f32>(2.0)),vec2<f32>(3.0, 4.0))";
        assert_eq!(convert_wgsl_builtin_constructors(input), expected);
    }
    #[test]
    fn test_partial_nested() {
        let input =
            "Mat2x2F32::new(Vec2F32::new(1.0, Vec2F32::new(2.0)), vec2<f32>(3.0, 4.0))".to_string();
        let expected = "mat2x2<f32>(vec2<f32>(1.0,vec2<f32>(2.0)), vec2<f32>(3.0, 4.0))";
        assert_eq!(convert_wgsl_builtin_constructors(input), expected);
    }

    #[test]
    fn test_complex_string_conversion() {
        let partially_converted_input = "fn main(){
        struct MyStruct { x: Vec3F32, y: Mat2x4Bool };
        struct MyOuterStruct { z: MyStruct };
        let obj = MyOuterStruct ( MyStruct ( Vec3F32::new(1.0, 2.0, 3.0), mat2x4<bool>::new(Vec2Bool::new(true, false), Vec2Bool::new(false, true), vec2<bool>::new(true, true), Vec2Bool::new(false, false)) ) ); }"
        .to_string();
        let expected_output = "fn main(){
        struct MyStruct { x: Vec3F32, y: Mat2x4Bool };
        struct MyOuterStruct { z: MyStruct };
        let obj = MyOuterStruct ( MyStruct (vec3<f32>(1.0, 2.0, 3.0),mat2x4<bool>(vec2<bool>(true, false),vec2<bool>(false, true),vec2<bool>(true, true),vec2<bool>(false, false)) ) ); }".to_string();
        let output = convert_wgsl_builtin_constructors(partially_converted_input);
        assert_eq!(output, expected_output);
    }
    #[test]
    fn test_whole_module() {
        let input = "fn main(@builtin(global_invocation_id) global_id:vec3<u32>) {
                let obj = TStruct (
                    1.0, 
                    Vec3F32 :: new(2.0, 3.0, 4.0)
                ); return;
                }";
        let expected = "fn main(@builtin(global_invocation_id) global_id:vec3<u32>) {
                let obj = TStruct (
                    1.0,vec3<f32>(2.0, 3.0, 4.0)
                ); return;
                }";
        assert_eq!(
            convert_wgsl_builtin_constructors(input.to_string()),
            expected
        );
    }
}
