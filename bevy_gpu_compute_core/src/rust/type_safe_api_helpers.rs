use bytemuck::Pod;

use super::{InputTypesMetadataTrait, OutputTypesMetadataTrait};
/*
The types in this file are used by the main crate to provide a type-safe API to the end user
 */

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
/// used in the input/output types below to indicate an input/output index is not used
pub struct _INTERNAL_UNUSED {}

pub trait TypesSpec {
    type ConfigInputTypes: InputTypesMetadataTrait;
    type InputArrayTypes: InputTypesMetadataTrait;
    type OutputArrayTypes: OutputTypesMetadataTrait;
}
