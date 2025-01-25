/// whenever `#[` is detected, remove everything up to the next `]` using regex, also remove the hash+square brackets
/// remove these specific strings: `#[wgsl_config]`, `#[wgsl_input_array]`, `#[wgsl_output_array]` and `#[wgsl_output_vec]`, but allow any number of whitespaces or newlines between the square brackets and the attribute name
pub fn remove_internal_attributes(file: String) -> String {
    let re = regex::Regex::new(r"#\[\s*wgsl_config\s*\]|\s*#\[\s*wgsl_input_array\s*\]|\s*#\[\s*wgsl_output_array\s*\]|\s*#\[\s*wgsl_output_vec\s*\]").unwrap();
    re.replace_all(&file, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_internal_attributes() {
        let input = r#"
        #[wgsl_config]
        #[wgsl_input_array]
        #[wgsl_output_array]
        #[wgsl_output_vec]
        #[valid]
        #[wgsl_config]#[wgsl_input_array]#[wgsl_output_array]#[wgsl_output_vec]
        "#;
        let expected = r#"
        
        #[valid]
        
        "#;
        let result = remove_internal_attributes(input.to_string());
        assert_eq!(result, expected);
    }
}
