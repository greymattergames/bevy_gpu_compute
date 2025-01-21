/// whenever `#[` is detected, remove everything up to the next `]` using regex, also remove the hash+square brackets
pub fn remove_attributes(file: String) -> String {
    let re = regex::Regex::new(r"#\[[^\]]*\]").unwrap();
    re.replace_all(&file, "").to_string()
}
