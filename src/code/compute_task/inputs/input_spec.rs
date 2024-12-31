use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;

use bevy::ecs::query::QueryData;
use bevy::prelude::Component;
use bevy::reflect::Tuple;
use bytemuck::Pod;

use super::input_data::InputData;

#[derive(Clone, Debug)]
pub struct InputVectorMetadata {
    pub bytes: usize,
    pub binding_number: u32,
    // pub skip_validation: bool,
}

pub trait InputVectorTypesSpec {
    // Associated types for inputs and outputs - must implement Pod
    type Input1: Pod + Send + Sync;
    type Input2: Pod + Send + Sync;
    type Input3: Pod + Send + Sync;
    type Input4: Pod + Send + Sync;
    type Input5: Pod + Send + Sync;
    type Input6: Pod + Send + Sync;

    // Metadata for each input
    const INPUT1_METADATA: Option<InputVectorMetadata>;
    const INPUT2_METADATA: Option<InputVectorMetadata>;
    const INPUT3_METADATA: Option<InputVectorMetadata>;
    const INPUT4_METADATA: Option<InputVectorMetadata>;
    const INPUT5_METADATA: Option<InputVectorMetadata>;
    const INPUT6_METADATA: Option<InputVectorMetadata>;
}

pub struct BlankInputVectorTypesSpec {}
impl InputVectorTypesSpec for BlankInputVectorTypesSpec {
    type Input1 = ();
    type Input2 = ();
    type Input3 = ();
    type Input4 = ();
    type Input5 = ();
    type Input6 = ();

    const INPUT1_METADATA: Option<InputVectorMetadata> = None;
    const INPUT2_METADATA: Option<InputVectorMetadata> = None;
    const INPUT3_METADATA: Option<InputVectorMetadata> = None;
    const INPUT4_METADATA: Option<InputVectorMetadata> = None;
    const INPUT5_METADATA: Option<InputVectorMetadata> = None;
    const INPUT6_METADATA: Option<InputVectorMetadata> = None;
}
