use bytemuck::Pod;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]

pub struct _INTERNAL_UNUSED {}

// //   ! todo!("Re enable the type constraints in misc_types");
pub trait InputConfigTypesSpec {
    type Input0: Pod + Send + Sync;
    type Input1: Pod + Send + Sync;
    type Input2: Pod + Send + Sync;
    type Input3: Pod + Send + Sync;
    type Input4: Pod + Send + Sync;
    type Input5: Pod + Send + Sync;
}

pub trait InputVectorTypesSpec {
    type Input0: Pod + Send + Sync;
    type Input1: Pod + Send + Sync;
    type Input2: Pod + Send + Sync;
    type Input3: Pod + Send + Sync;
    type Input4: Pod + Send + Sync;
    type Input5: Pod + Send + Sync;
}

pub trait OutputVectorTypesSpec {
    type Output0: Pod + Send + Sync;
    type Output1: Pod + Send + Sync;
    type Output2: Pod + Send + Sync;
    type Output3: Pod + Send + Sync;
    type Output4: Pod + Send + Sync;
    type Output5: Pod + Send + Sync;
}
// pub trait InputConfigTypesSpec {
//     type Input0;
//     type Input1;
//     type Input2;
//     type Input3;
//     type Input4;
//     type Input5;
// }

// pub trait InputVectorTypesSpec {
//     type Input0;
//     type Input1;
//     type Input2;
//     type Input3;
//     type Input4;
//     type Input5;
// }

// pub trait OutputVectorTypesSpec {
//     type Output0;
//     type Output1;
//     type Output2;
//     type Output3;
//     type Output4;
//     type Output5;
// }
