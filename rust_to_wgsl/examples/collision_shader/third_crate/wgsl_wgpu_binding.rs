use std::str::FromStr;

pub struct WgslWgpuBinding {
    pub group_num: u32,
    pub entry_num: u32,
    pub buffer_type: WgpuBufferType,
    pub access: WgpuBufferAccessMode,
    pub name: String,
    pub type_name: String,
}
impl ToString for WgslWgpuBinding {
    fn to_string(&self) -> String {
        return format!(
            "@group({}) @binding({}) var<{}, {}> {}: {};\n",
            self.group_num,
            self.entry_num,
            self.buffer_type.to_string(),
            self.access.to_string(),
            self.name,
            self.type_name
        );
    }
}
enum WgpuBufferType {
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
// string either "read" or "read_write"
enum WgpuBufferAccessMode {
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
