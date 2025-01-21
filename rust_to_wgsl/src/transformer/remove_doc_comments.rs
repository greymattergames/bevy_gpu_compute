use regex::Regex;

/// Remove doc comments from rust source code
///
/// # Arguments
/// * `source` - The source code to process as a string
///
/// # Returns
/// * The processed source code with comments removed
pub fn remove_doc_comments(source: &str) -> String {
    let mut content = String::from(source);
    content = remove_doc_block_comments(&content);
    content = remove_doc_singleline_comments(&content);
    content
}

/// Remove documentation block comments (/** */ and /*! */) from source code
fn remove_doc_block_comments(source: &str) -> String {
    let doc_re = Regex::new(r"/\*[\*!].*?\*/").unwrap();
    doc_re.replace_all(source, "").to_string()
}

/// Remove documentation single line comments (/// and //!) from source code
fn remove_doc_singleline_comments(source: &str) -> String {
    let doc_re = Regex::new(r"(?m)///.*$|//!.*$").unwrap();
    doc_re.replace_all(source, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_doc_block_comments() {
        let input = "/** This is doc comment */\nfn test() {}\n/*! Module doc */";
        let expected = "\nfn test() {}\n";
        assert_eq!(remove_doc_block_comments(input), expected);
    }

    #[test]
    fn test_remove_doc_singleline_comments() {
        let input = "/// Doc comment\n//! Module doc\nfn test() {}";
        let expected = "\n\nfn test() {}";
        assert_eq!(remove_doc_singleline_comments(input), expected);
    }
    #[test]
    fn remove_both() {
        let input = "//! Module doc\n/** This is doc comment */\nfn test() {}\n/*! Module doc */";
        let expected = "\n\nfn test() {}\n";
        assert_eq!(remove_doc_comments(input), expected);
    }
}
