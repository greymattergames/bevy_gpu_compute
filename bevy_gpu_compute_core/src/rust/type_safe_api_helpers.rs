use bytemuck::Pod;
/*
The types in this file are used by the main crate to provide a type-safe API to the end user
 */

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
/// used in the input/output types below to indicate an input/output index is not used
pub struct _INTERNAL_UNUSED {}

pub trait TypesSpec {
    type InputConfigTypes: InputConfigTypesSpec;
    type InputArrayTypes: InputVectorTypesSpec;
    type OutputArrayTypes: OutputVectorTypesSpec;
}
#[derive(Debug)]

pub struct BlankInputVectorTypesSpec {}
impl InputVectorTypesSpec for BlankInputVectorTypesSpec {
    type Input0 = ();
    type Input1 = ();
    type Input2 = ();
    type Input3 = ();
    type Input4 = ();
    type Input5 = ();
}
#[derive(Debug)]

pub struct BlankInputConfigTypesSpec {}
impl InputConfigTypesSpec for BlankInputConfigTypesSpec {
    type Input0 = ();
    type Input1 = ();
    type Input2 = ();
    type Input3 = ();
    type Input4 = ();
    type Input5 = ();
}
#[derive(Debug)]

pub struct BlankOutputVectorTypesSpec {}
impl OutputVectorTypesSpec for BlankOutputVectorTypesSpec {
    type Output0 = ();
    type Output1 = ();
    type Output2 = ();
    type Output3 = ();
    type Output4 = ();
    type Output5 = ();
}
#[derive(Debug)]

pub struct BlankTypesSpec {}
impl TypesSpec for BlankTypesSpec {
    type InputConfigTypes = BlankInputConfigTypesSpec;
    type InputArrayTypes = BlankInputVectorTypesSpec;
    type OutputArrayTypes = BlankOutputVectorTypesSpec;
}

pub trait InputConfigTypesSpec {
    type Input0: Pod + Send + Sync + std::fmt::Debug;
    type Input1: Pod + Send + Sync + std::fmt::Debug;
    type Input2: Pod + Send + Sync + std::fmt::Debug;
    type Input3: Pod + Send + Sync + std::fmt::Debug;
    type Input4: Pod + Send + Sync + std::fmt::Debug;
    type Input5: Pod + Send + Sync + std::fmt::Debug;
}

pub trait InputVectorTypesSpec {
    type Input0: Pod + Send + Sync + std::fmt::Debug;
    type Input1: Pod + Send + Sync + std::fmt::Debug;
    type Input2: Pod + Send + Sync + std::fmt::Debug;
    type Input3: Pod + Send + Sync + std::fmt::Debug;
    type Input4: Pod + Send + Sync + std::fmt::Debug;
    type Input5: Pod + Send + Sync + std::fmt::Debug;
}

pub trait OutputVectorTypesSpec {
    type Output0: Pod + Send + Sync + std::fmt::Debug;
    type Output1: Pod + Send + Sync + std::fmt::Debug;
    type Output2: Pod + Send + Sync + std::fmt::Debug;
    type Output3: Pod + Send + Sync + std::fmt::Debug;
    type Output4: Pod + Send + Sync + std::fmt::Debug;
    type Output5: Pod + Send + Sync + std::fmt::Debug;
}
