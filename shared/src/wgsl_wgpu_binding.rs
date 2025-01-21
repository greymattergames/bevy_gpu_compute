use std::str::FromStr;

use crate::wgsl_components::WgslOutputArray;
#[derive(Clone, Debug)]
pub struct WgslWgpuBinding {
    pub group_num: u32,
    pub entry_num: u32,
    pub buffer_type: WgpuBufferType,
    pub access: WgpuBufferAccessMode,
    pub name: String,
    pub type_decl: String,
}
impl ToString for WgslWgpuBinding {
    fn to_string(&self) -> String {
        if self.buffer_type == WgpuBufferType::Uniform {
            return format!(
                "@group({}) @binding({}) var<{}> {}: {};\n",
                self.group_num,
                self.entry_num,
                self.buffer_type.to_string(),
                self.name,
                self.type_decl
            );
        }
        return format!(
            "@group({}) @binding({}) var<{}, {}> {}: {};\n",
            self.group_num,
            self.entry_num,
            self.buffer_type.to_string(),
            self.access.to_string(),
            self.name,
            self.type_decl
        );
    }
}

impl WgslWgpuBinding {
    pub fn uniform(group_num: u32, entry_num: u32, name: String, type_decl: &str) -> Self {
        WgslWgpuBinding {
            group_num,
            entry_num,
            buffer_type: WgpuBufferType::Uniform,
            access: WgpuBufferAccessMode::Read,
            name,
            type_decl: type_decl.to_string(),
        }
    }
    pub fn input_array(group_num: u32, entry_num: u32, name: String, type_decl: String) -> Self {
        WgslWgpuBinding {
            group_num,
            entry_num,
            buffer_type: WgpuBufferType::Storage,
            access: WgpuBufferAccessMode::Read,
            name,
            type_decl: type_decl,
        }
    }
    pub fn output_array(group_num: u32, entry_num: u32, name: String, type_decl: String) -> Self {
        WgslWgpuBinding {
            group_num,
            entry_num,
            buffer_type: WgpuBufferType::Storage,
            access: WgpuBufferAccessMode::ReadWrite,
            name,
            type_decl: type_decl,
        }
    }

    pub fn counter(
        entry_number: u32,
        out_array: &WgslOutputArray,
        out_array_binding: &WgslWgpuBinding,
    ) -> Self {
        assert!(
            out_array.atomic_counter_name.is_some(),
            "Atomic counter name must be present if you want to create a counter binding"
        );
        WgslWgpuBinding {
            group_num: out_array_binding.group_num,
            entry_num: entry_number,
            buffer_type: WgpuBufferType::Storage,
            access: WgpuBufferAccessMode::ReadWrite,
            name: out_array.atomic_counter_name.as_ref().unwrap().clone(),
            type_decl: "atomic<u32>".to_string(),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum WgpuBufferType {
    Storage,
    Uniform,
}
impl FromStr for WgpuBufferType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "storage" => Ok(WgpuBufferType::Storage),
            "uniform" => Ok(WgpuBufferType::Uniform),
            _ => Err("Invalid buffer access type".to_string()),
        }
    }
}
impl ToString for WgpuBufferType {
    fn to_string(&self) -> String {
        match self {
            WgpuBufferType::Storage => "storage".to_string(),
            WgpuBufferType::Uniform => "uniform".to_string(),
        }
    }
}
#[derive(Clone, Debug)]
pub enum WgpuBufferAccessMode {
    Read,
    ReadWrite,
}
impl FromStr for WgpuBufferAccessMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "read" => Ok(WgpuBufferAccessMode::Read),
            "read_write" => Ok(WgpuBufferAccessMode::ReadWrite),
            _ => Err("Invalid buffer access mode".to_string()),
        }
    }
}
impl ToString for WgpuBufferAccessMode {
    fn to_string(&self) -> String {
        match self {
            WgpuBufferAccessMode::Read => "read".to_string(),
            WgpuBufferAccessMode::ReadWrite => "read_write".to_string(),
        }
    }
}
