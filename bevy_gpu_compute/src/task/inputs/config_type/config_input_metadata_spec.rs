use bevy_gpu_compute_core::{
    ConfigInputTypesSpec, wgsl::shader_custom_type_name::ShaderCustomTypeName,
};

#[derive(Copy, Clone)]
pub struct ConfigInputMetadataDefinition<'a> {
    pub binding_number: u32,
    pub name: &'a ShaderCustomTypeName,
}
#[derive(Clone, Debug)]
pub struct ConfigInputMetadata {
    bytes: usize,
    binding_number: u32,
    name: ShaderCustomTypeName,
}

impl ConfigInputMetadata {
    pub fn new(bytes: usize, binding_number: u32, name: ShaderCustomTypeName) -> Self {
        ConfigInputMetadata {
            bytes,
            binding_number,
            name,
        }
    }
    pub fn get_bytes(&self) -> usize {
        self.bytes
    }
    pub fn get_binding_number(&self) -> u32 {
        self.binding_number
    }
    pub fn name(&self) -> &ShaderCustomTypeName {
        &self.name
    }
}

#[derive(Clone)]
pub struct ConfigInputsMetadataSpec {
    input0: Option<ConfigInputMetadata>,
    input1: Option<ConfigInputMetadata>,
    input2: Option<ConfigInputMetadata>,
    input3: Option<ConfigInputMetadata>,
    input4: Option<ConfigInputMetadata>,
    input5: Option<ConfigInputMetadata>,
}

impl Default for ConfigInputsMetadataSpec {
    fn default() -> Self {
        Self::empty()
    }
}

impl ConfigInputsMetadataSpec {
    pub fn empty() -> Self {
        ConfigInputsMetadataSpec {
            input0: None,
            input1: None,
            input2: None,
            input3: None,
            input4: None,
            input5: None,
        }
    }
    fn get_input<ST>(
        i: usize,
        definitions: [Option<ConfigInputMetadataDefinition>; 6],
    ) -> Option<ConfigInputMetadata> {
        if let Some(def) = definitions[i] {
            Some(ConfigInputMetadata::new(
                std::mem::size_of::<ST>(),
                def.binding_number,
                def.name.clone(),
            ))
        } else {
            None
        }
    }
    pub fn from_config_input_types_spec<T: ConfigInputTypesSpec>(
        definitions: [Option<ConfigInputMetadataDefinition>; 6],
    ) -> Self {
        Self {
            input0: Self::get_input::<T::Input0>(0, definitions),
            input1: Self::get_input::<T::Input1>(1, definitions),
            input2: Self::get_input::<T::Input2>(2, definitions),
            input3: Self::get_input::<T::Input3>(3, definitions),
            input4: Self::get_input::<T::Input4>(4, definitions),
            input5: Self::get_input::<T::Input5>(5, definitions),
        }
    }
    pub fn get_all_metadata(&self) -> [Option<&ConfigInputMetadata>; 6] {
        [
            self.input0.as_ref(),
            self.input1.as_ref(),
            self.input2.as_ref(),
            self.input3.as_ref(),
            self.input4.as_ref(),
            self.input5.as_ref(),
        ]
    }
    pub fn get_input0_metadata(&self) -> Option<&ConfigInputMetadata> {
        self.input0.as_ref()
    }
    pub fn get_input1_metadata(&self) -> Option<&ConfigInputMetadata> {
        self.input1.as_ref()
    }
    pub fn get_input2_metadata(&self) -> Option<&ConfigInputMetadata> {
        self.input2.as_ref()
    }
    pub fn get_input3_metadata(&self) -> Option<&ConfigInputMetadata> {
        self.input3.as_ref()
    }
    pub fn get_input4_metadata(&self) -> Option<&ConfigInputMetadata> {
        self.input4.as_ref()
    }
    pub fn get_input5_metadata(&self) -> Option<&ConfigInputMetadata> {
        self.input5.as_ref()
    }
    pub fn set_input0_metadata(&mut self, metadata: ConfigInputMetadata) {
        self.input0 = Some(metadata);
    }
    pub fn set_input1_metadata(&mut self, metadata: ConfigInputMetadata) {
        self.input1 = Some(metadata);
    }
    pub fn set_input2_metadata(&mut self, metadata: ConfigInputMetadata) {
        self.input2 = Some(metadata);
    }
    pub fn set_input3_metadata(&mut self, metadata: ConfigInputMetadata) {
        self.input3 = Some(metadata);
    }
    pub fn set_input4_metadata(&mut self, metadata: ConfigInputMetadata) {
        self.input4 = Some(metadata);
    }
    pub fn set_input5_metadata(&mut self, metadata: ConfigInputMetadata) {
        self.input5 = Some(metadata);
    }
}
