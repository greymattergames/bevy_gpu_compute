/// INPUTS:
/// all it needs to do is provide type-safe setter api
/// and then convert itself into TypeErasedConfigInputData

/// and also convert each of its vectors or fields into raw bytes using bytemuck, with a function call that takes the type name of that field as input

/// AND also get the lengths of each of the input arrays, also referenced by type name
///
///
/// OUTPUTS:
/// convert from bytes per output type, into the proper type for that output
/// Can be set by output type name, again
/// Provide type safe access methods
///
/// Provide traits that the proc macro must conform to
use bytemuck::Pod;
pub trait ConfigInputDataTrait {
    /// type-safe setter
    fn set<T: Pod + Send + Sync + std::fmt::Debug>(&mut self, input_name: &str, data: T);
    fn get_bytes(&self, input_name: &str) -> Option<&[u8]>;
}

pub trait ArrayInputDataTrait {
    fn set<T: Pod + Send + Sync + std::fmt::Debug>(&mut self, input_name: &str, data: Vec<T>);
    fn get_bytes(&self, input_name: &str) -> Option<&[u8]>;
    fn get_length(&self, input_name: &str) -> Option<usize>;
}

pub trait ArrayOutputDataTrait {
    fn set(&mut self, input_name: &str, bytes: &[u8]) -> Result<(), String>;
}

// example implementation
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MyConfig1 {
    pub my_field: f32,
    pub my_other_field: f32,
}
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MyConfig2 {
    pub my_field: f32,
    pub my_other_field: f32,
}
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MyConfig3 {
    pub my_field: u32,
    pub field_2: u32,
}

pub struct ConfigInputData {
    my_config_1_bytes: Vec<u8>,
    my_config_2_bytes: Vec<u8>,
    my_config_3_bytes: Vec<u8>,
}
impl ConfigInputDataTrait for ConfigInputData {
    fn set<T: Pod + Send + Sync + std::fmt::Debug>(&mut self, input_name: &str, data: T) {
        match input_name {
            "my_config_1" => self.my_config_1_bytes = bytemuck::bytes_of(&data).to_vec(),
            "my_config_2" => self.my_config_2_bytes = bytemuck::bytes_of(&data).to_vec(),
            "my_config_3" => self.my_config_3_bytes = bytemuck::bytes_of(&data).to_vec(),
            _ => panic!("unknown config input name {}", input_name),
        }
    }
    fn get_bytes(&self, input_name: &str) -> Option<&[u8]> {
        match input_name {
            "my_config_1" => Some(&self.my_config_1_bytes),
            "my_config_2" => Some(&self.my_config_2_bytes),
            "my_config_3" => Some(&self.my_config_3_bytes),
            _ => None,
        }
    }
}

impl ConfigInputData {
    pub fn new() -> Self {
        Self {
            my_config_1_bytes: vec![],
            my_config_2_bytes: vec![],
            my_config_3_bytes: vec![],
        }
    }
    pub fn set_my_config_1(&mut self, data: MyConfig1) {
        self.set("my_config_1", data);
    }
    pub fn set_my_config_2(&mut self, data: MyConfig2) {
        self.set("my_config_2", data);
    }
    pub fn set_my_config_3(&mut self, data: MyConfig3) {
        self.set("my_config_3", data);
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MyInput1 {
    pub my_field: f32,
    pub my_other_field: f32,
}
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MyInput2 {
    pub my_field: f32,
    pub my_other_field: f32,
}
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MyInput3 {
    pub my_field: u32,
    pub field_2: u32,
}

pub struct ArrayInputData {
    my_input_1_bytes: Vec<u8>,
    my_input_2_bytes: Vec<u8>,
    my_input_3_bytes: Vec<u8>,
    my_input_1_length: usize,
    my_input_2_length: usize,
    my_input_3_length: usize,
}
impl ArrayInputDataTrait for ArrayInputData {
    fn set<T: Pod + Send + Sync + std::fmt::Debug>(&mut self, input_name: &str, data: Vec<T>) {
        match input_name {
            "my_input_1" => {
                self.my_input_1_bytes = bytemuck::cast_slice(&data).to_vec(); //
                self.my_input_1_length = data.len();
            }
            "my_input_2" => {
                self.my_input_2_bytes = bytemuck::cast_slice(&data).to_vec();
                self.my_input_2_length = data.len();
            }
            "my_input_3" => {
                self.my_input_3_bytes = bytemuck::cast_slice(&data).to_vec();
                self.my_input_3_length = data.len();
            }
            _ => panic!("unknown array input name {}", input_name),
        }
    }
    fn get_bytes(&self, input_name: &str) -> Option<&[u8]> {
        match input_name {
            "my_input_1" => Some(&self.my_input_1_bytes),
            "my_input_2" => Some(&self.my_input_2_bytes),
            "my_input_3" => Some(&self.my_input_3_bytes),
            _ => None,
        }
    }

    fn get_length(&self, input_name: &str) -> Option<usize> {
        match input_name {
            "my_input_1" => Some(self.my_input_1_length),
            "my_input_2" => Some(self.my_input_2_length),
            "my_input_3" => Some(self.my_input_3_length),
            _ => None,
        }
    }
}

impl ArrayInputData {
    pub fn new() -> Self {
        Self {
            my_input_1_bytes: vec![],
            my_input_2_bytes: vec![],
            my_input_3_bytes: vec![],
            my_input_1_length: 0,
            my_input_2_length: 0,
            my_input_3_length: 0,
        }
    }
    pub fn set_my_input_1(&mut self, data: Vec<MyInput1>) {
        self.set("my_input_1", data);
    }
    pub fn set_my_input_2(&mut self, data: Vec<MyInput2>) {
        self.set("my_input_2", data);
    }
    pub fn set_my_input_3(&mut self, data: Vec<MyInput3>) {
        self.set("my_input_3", data);
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MyOutput1 {
    pub my_field: f32,
    pub my_other_field: f32,
}
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MyOutput2 {
    pub my_field: f32,
    pub my_other_field: f32,
}
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MyOutput3 {
    pub my_field: u32,
    pub field_2: u32,
}

pub struct ArrayOutputData {
    my_output_1: Option<Vec<MyOutput1>>,
    my_output_2: Option<Vec<MyOutput2>>,
    my_output_3: Option<Vec<MyOutput3>>,
}
impl ArrayOutputDataTrait for ArrayOutputData {
    fn set(&mut self, output_name: &str, bytes: &[u8]) -> Result<(), String> {
        match output_name {
            "my_output_1" => {
                if bytes.len() % std::mem::size_of::<MyOutput1>() != 0 {
                    return Err("Byte length not aligned with output type size".to_string());
                }
                if bytes.len() == 0 {
                    self.my_output_1 = Some(Vec::new());
                } else {
                    self.my_output_1 = Some(bytemuck::cast_slice(bytes).to_vec());
                }
            }
            "my_output_2" => {
                if bytes.len() % std::mem::size_of::<MyOutput2>() != 0 {
                    return Err("Byte length not aligned with output type size".to_string());
                }
                if bytes.len() == 0 {
                    self.my_output_2 = Some(Vec::new());
                } else {
                    self.my_output_2 = Some(bytemuck::cast_slice(bytes).to_vec());
                }
            }
            "my_output_3" => {
                if bytes.len() % std::mem::size_of::<MyOutput3>() != 0 {
                    return Err("Byte length not aligned with output type size".to_string());
                }
                if bytes.len() == 0 {
                    self.my_output_3 = Some(Vec::new());
                } else {
                    self.my_output_3 = Some(bytemuck::cast_slice(bytes).to_vec());
                }
            }
            _ => panic!("unknown array output name {}", output_name),
        }
        return Ok(());
    }
}

impl ArrayOutputData {
    pub fn empty() -> Self {
        Self {
            my_output_1: None,
            my_output_2: None,
            my_output_3: None,
        }
    }
    pub fn get_my_output_1(&self) -> Option<&Vec<MyOutput1>> {
        self.my_output_1.as_ref()
    }
    pub fn get_my_output_2(&self) -> Option<&Vec<MyOutput2>> {
        self.my_output_2.as_ref()
    }
    pub fn get_my_output_3(&self) -> Option<&Vec<MyOutput3>> {
        self.my_output_3.as_ref()
    }
}
