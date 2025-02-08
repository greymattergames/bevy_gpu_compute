use regex::Regex;

// since rust does not allow "var" instead of let we need to work with the final string, not the ast.
/// replace all "let mut" with "var", using regex, to allow for any number of whitespaces between them
pub fn replace_let_mut_with_var(s: &str) -> String {
    let pattern = Regex::new(r"let\s+mut\s+").unwrap();
    pattern.replace_all(s, "var ").to_string()
}
