use bevy::{log, prelude::Component};
use bevy_gpu_compute_core::{BlankTypesSpec, ConfigInputTypesSpec, TypesSpec};

pub trait ConfigInputDataTrait: Send + Sync {
    fn input_bytes(&self, index: usize) -> Option<&[u8]>;
}

#[derive(Component)]
pub struct ConfigInputData<T: TypesSpec> {
    input0: Option<<<T as TypesSpec>::ConfigInputTypes as ConfigInputTypesSpec>::Input0>,
    input1: Option<<<T as TypesSpec>::ConfigInputTypes as ConfigInputTypesSpec>::Input1>,
    input2: Option<<<T as TypesSpec>::ConfigInputTypes as ConfigInputTypesSpec>::Input2>,
    input3: Option<<<T as TypesSpec>::ConfigInputTypes as ConfigInputTypesSpec>::Input3>,
    input4: Option<<<T as TypesSpec>::ConfigInputTypes as ConfigInputTypesSpec>::Input4>,
    input5: Option<<<T as TypesSpec>::ConfigInputTypes as ConfigInputTypesSpec>::Input5>,
    _phantom: std::marker::PhantomData<T>,
}
impl<T: TypesSpec> std::fmt::Debug for ConfigInputData<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigInputData")
            .field("input0", &self.input0)
            .field("input1", &self.input1)
            .field("input2", &self.input2)
            .field("input3", &self.input3)
            .field("input4", &self.input4)
            .field("input5", &self.input5)
            .finish()
    }
}
impl Default for ConfigInputData<BlankTypesSpec> {
    fn default() -> Self {
        ConfigInputData {
            input0: None,
            input1: None,
            input2: None,
            input3: None,
            input4: None,
            input5: None,

            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: TypesSpec> ConfigInputData<T> {
    pub fn empty() -> Self {
        ConfigInputData {
            input0: None,
            input1: None,
            input2: None,
            input3: None,
            input4: None,
            input5: None,

            _phantom: std::marker::PhantomData,
        }
    }

    // Type-safe setters that take vectors of Pod types
    pub fn set_input0(
        &mut self,
        input: <<T as TypesSpec>::ConfigInputTypes as ConfigInputTypesSpec>::Input0,
    ) {
        self.input0 = Some(input);
    }

    pub fn set_input1(
        &mut self,
        input: <<T as TypesSpec>::ConfigInputTypes as ConfigInputTypesSpec>::Input1,
    ) {
        self.input1 = Some(input);
    }
    pub fn set_input2(
        &mut self,
        input: <<T as TypesSpec>::ConfigInputTypes as ConfigInputTypesSpec>::Input2,
    ) {
        self.input2 = Some(input);
    }
    pub fn set_input3(
        &mut self,
        input: <<T as TypesSpec>::ConfigInputTypes as ConfigInputTypesSpec>::Input3,
    ) {
        self.input3 = Some(input);
    }
    pub fn set_input4(
        &mut self,
        input: <<T as TypesSpec>::ConfigInputTypes as ConfigInputTypesSpec>::Input4,
    ) {
        self.input4 = Some(input);
    }
    pub fn set_input5(
        &mut self,
        input: <<T as TypesSpec>::ConfigInputTypes as ConfigInputTypesSpec>::Input5,
    ) {
        self.input5 = Some(input);
    }

    pub fn input0_bytes(&self) -> Option<&[u8]> {
        if let Some(data) = &self.input0 {
            Some(bytemuck::bytes_of(data))
        } else {
            None
        }
    }

    pub fn input1_bytes(&self) -> Option<&[u8]> {
        if let Some(data) = &self.input1 {
            Some(bytemuck::bytes_of(data))
        } else {
            None
        }
    }
    pub fn input2_bytes(&self) -> Option<&[u8]> {
        if let Some(data) = &self.input2 {
            Some(bytemuck::bytes_of(data))
        } else {
            None
        }
    }
    pub fn input3_bytes(&self) -> Option<&[u8]> {
        if let Some(data) = &self.input3 {
            Some(bytemuck::bytes_of(data))
        } else {
            None
        }
    }
    pub fn input4_bytes(&self) -> Option<&[u8]> {
        if let Some(data) = &self.input4 {
            Some(bytemuck::bytes_of(data))
        } else {
            None
        }
    }
    pub fn input5_bytes(&self) -> Option<&[u8]> {
        if let Some(data) = &self.input5 {
            Some(bytemuck::bytes_of(data))
        } else {
            None
        }
    }
}

// impl<T: TypesSpec + Send + Sync> ConfigInputDataTrait for ConfigInputData<T> {
//     fn input_bytes(&self, index: usize) -> Option<&[u8]> {
//         log::info!("input_bytes index: {}", index);
//         match index {
//             0 => self.input0_bytes(),
//             1 => self.input1_bytes(),
//             2 => self.input2_bytes(),
//             3 => self.input3_bytes(),
//             4 => self.input4_bytes(),
//             5 => self.input5_bytes(),
//             _ => None,
//         }
//     }
// }
