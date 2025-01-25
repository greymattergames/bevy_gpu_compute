use super::{InputTypesMetadataTrait, OutputTypesMetadataTrait};
/*
The types in this file are used by the main crate to provide a type-safe API to the end user
 */

pub trait TypesSpec {
    type ConfigInputTypes: InputTypesMetadataTrait;
    type InputArrayTypes: InputTypesMetadataTrait;
    type OutputArrayTypes: OutputTypesMetadataTrait;
}
