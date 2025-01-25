use crate::wgsl::shader_custom_type_name::ShaderCustomTypeName;

#[derive(Clone, Debug)]
pub struct InputTypeMetadata {
    pub bytes: usize,
    pub binding_number: u32,
    pub name: ShaderCustomTypeName,
}
pub trait InputTypesMetadataTrait {
    fn get_all() -> Vec<InputTypeMetadata>;
}

#[derive(Clone, Debug)]
pub struct OutputTypeMetadata {
    pub bytes: usize,
    pub binding_number: u32,
    pub include_count: bool,
    pub count_binding_number: Option<u32>,
    pub name: ShaderCustomTypeName,
}

pub trait OutputTypesMetadataTrait {
    fn get_all() -> Vec<OutputTypeMetadata>;
}
