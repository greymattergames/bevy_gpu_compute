use super::output_array::WgslOutputArray;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct WgslWgpuBinding {
    pub group_num: u32,
    pub entry_num: u32,
    pub buffer_type: WgpuBufferType,
    pub access: WgpuBufferAccessMode,
    pub name: String,
    pub type_decl: String,
}
impl std::fmt::Display for WgslWgpuBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.buffer_type == WgpuBufferType::Uniform {
            return writeln!(
                f,
                "@group({}) @binding({}) var<{}> {}: {};",
                self.group_num, self.entry_num, self.buffer_type, self.name, self.type_decl
            );
        }
        writeln!(
            f,
            "@group({}) @binding({}) var<{}, {}> {}: {};",
            self.group_num,
            self.entry_num,
            self.buffer_type,
            self.access,
            self.name,
            self.type_decl
        )
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
            type_decl,
        }
    }
    pub fn output_array(group_num: u32, entry_num: u32, name: String, type_decl: String) -> Self {
        WgslWgpuBinding {
            group_num,
            entry_num,
            buffer_type: WgpuBufferType::Storage,
            access: WgpuBufferAccessMode::ReadWrite,
            name,
            type_decl,
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
impl std::fmt::Display for WgpuBufferType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WgpuBufferType::Storage => write!(f, "storage"),
            WgpuBufferType::Uniform => write!(f, "uniform"),
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
impl std::fmt::Display for WgpuBufferAccessMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WgpuBufferAccessMode::Read => write!(f, "read"),
            WgpuBufferAccessMode::ReadWrite => write!(f, "read_write"),
        }
    }
}
