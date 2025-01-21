#[derive(Clone, Debug, PartialEq)]

pub struct ShaderCustomTypeName {
    name: String,
    upper: String,
    lower: String,
    input_array_length: String,
    input_array: String,
    output_array_length: String,
    output_array: String,
    counter: String,
    uniform: String,
}

impl ShaderCustomTypeName {
    pub fn new(name: &str) -> Self {
        let upper = name.to_uppercase();
        let lower = name.to_lowercase();
        Self {
            name: name.to_string(),
            upper: upper.clone(),
            lower: lower.clone(),
            input_array_length: format!("{}_INPUT_ARRAY_LENGTH", upper.clone()),
            input_array: format!("{}_input_array", lower.clone()),
            output_array_length: format!("{}_OUTPUT_ARRAY_LENGTH", upper),
            output_array: format!("{}_output_array", lower),
            counter: format!("{}_counter", lower),
            uniform: lower.clone(),
        }
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn upper(&self) -> &String {
        &self.upper
    }
    pub fn lower(&self) -> &String {
        &self.lower
    }
    pub fn input_array_length(&self) -> String {
        self.input_array_length.clone()
    }
    pub fn input_array(&self) -> String {
        self.input_array.clone()
    }

    pub fn output_array_length(&self) -> String {
        self.output_array_length.clone()
    }
    pub fn output_array(&self) -> String {
        self.output_array.clone()
    }

    pub fn counter(&self) -> String {
        self.counter.clone()
    }
    pub fn uniform(&self) -> String {
        self.uniform.clone()
    }
}
