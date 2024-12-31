use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;

use bevy::ecs::query::QueryData;
use bevy::prelude::Component;
use bevy::reflect::Tuple;
use bytemuck::Pod;

use super::input_data::InputData;

pub trait InputVectorTypesSpec {
    // Associated types for inputs and outputs - must implement Pod
    type Input0: Pod + Send + Sync;
    type Input1: Pod + Send + Sync;
    type Input2: Pod + Send + Sync;
    type Input3: Pod + Send + Sync;
    type Input4: Pod + Send + Sync;
    type Input5: Pod + Send + Sync;

    // Metadata for each input
}

pub struct BlankInputVectorTypesSpec {}
impl InputVectorTypesSpec for BlankInputVectorTypesSpec {
    type Input0 = ();
    type Input1 = ();
    type Input2 = ();
    type Input3 = ();
    type Input4 = ();
    type Input5 = ();
}
