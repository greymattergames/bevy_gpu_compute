#[derive(Clone, Debug)]

pub struct CustomTypeName {
    pub name: String,
    pub upper: String,
    pub lower: String,
}

impl CustomTypeName {
    pub fn new(name: &String) -> Self {
        let upper = name.to_uppercase();
        let lower = name.to_lowercase();
        Self {
            name: name.clone(),
            upper,
            lower,
        }
    }
    pub fn eq(&self, other: &String) -> bool {
        self.name.to_string() == *other
    }
    pub fn uniform(&self) -> String {
        self.lower.to_string()
    }
    pub fn input_array_length(&self) -> String {
        format!("{}_INPUT_ARRAY_LENGTH", self.upper)
    }
    pub fn input_array(&self) -> String {
        format!("{}_input_array", self.lower)
    }
    pub fn output_array_length(&self) -> String {
        format!("{}_OUTPUT_ARRAY_LENGTH", self.upper)
    }
    pub fn output_array(&self) -> String {
        format!("{}_output_array", self.lower)
    }
    pub fn counter(&self) -> String {
        format!("{}_counter", self.lower)
    }
    pub fn index(&self) -> String {
        format!("{}_output_array_index", self.lower)
    }
}
