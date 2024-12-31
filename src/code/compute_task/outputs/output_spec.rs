// pub struct GpuAccBevyComputeTaskOutputSpec {
//     // each has a string label
//     // each has an optional bind number for correct association in the wgsl, otherwise they are numbered in order, with inputs coming before outputs
//     // each has an optional count output variable, which is used to determine how much memory is ACTUALLY needed for the output buffer (instead of just using the maximum)
//     pub label: String,
//     pub binding_number: Option<u32>,
//     pub item_bytes: usize,
//     pub converter: Box<dyn Fn(&[u8]) -> Box<dyn Any + Send + Sync> + Send + Sync>,
// }

// #[derive(Component)]
// pub struct GpuAccBevyComputeTaskOutputSpecs(pub Vec<GpuAccBevyComputeTaskOutputSpec>);

// testing its use

// fn testFunc() {
//     //  I want to convert an array of structs
//     struct ResultItem {
//         value: u32,
//         value2: f32,
//     }
//     struct Results {
//         results: [ResultItem],
//     }
//     let output_spec = GpuAccBevyComputeTaskOutputSpec {
//         label: "test".to_string(),
//         binding_number: Some(0),
//         item_bytes: 4,
//         converter: create_buffer_view_converter(),
//     };
// }
use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;

use bevy::prelude::Component;
use bytemuck::Pod;

pub trait OutputVectorTypesSpec {
    type Output0: Pod + Send + Sync;
    type Output1: Pod + Send + Sync;
    type Output2: Pod + Send + Sync;
    type Output3: Pod + Send + Sync;
    type Output4: Pod + Send + Sync;
    type Output5: Pod + Send + Sync;
}

pub struct BlankOutputVectorTypesSpec {}
impl OutputVectorTypesSpec for BlankOutputVectorTypesSpec {
    type Output0 = ();
    type Output1 = ();
    type Output2 = ();
    type Output3 = ();
    type Output4 = ();
    type Output5 = ();
}
