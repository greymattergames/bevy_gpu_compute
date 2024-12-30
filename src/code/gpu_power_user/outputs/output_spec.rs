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

// This holds type information for a specific output
pub struct OutputSpec {
    pub item_bytes: usize,
    pub type_id: TypeId,
    pub binding_number: u32,
    pub include_count: bool,
    pub count_binding_number: Option<u32>,
}

// Modified to hold type information
#[derive(Component)]
pub struct OutputSpecs {
    /// Maps a label to a tuple of (item_bytes, type_id, binding_number)
    pub specs: HashMap<String, OutputSpec>,
}

// Builder-style API for users to register their types
impl OutputSpecs {
    pub fn new() -> Self {
        Self {
            specs: HashMap::new(),
        }
    }

    pub fn register<T: Pod + 'static>(
        &mut self,
        label: &str,
        binding_number: i32,
        include_counter: bool,
        counter_binding_number: Option<i32>,
    ) {
        let item_bytes = std::mem::size_of::<T>();
        self.specs.insert(label.to_string(), OutputSpec {
            item_bytes,
            type_id: TypeId::of::<Vec<T>>(),
            binding_number,
            include_count: include_counter,
            count_binding_number: counter_binding_number,
        });
    }
    pub fn validate_data(
        &self,
        // output_data: &OutputData
        //
    ) -> Result<(), String> {
        todo!("Implement this, below is how it is implement for input data");
        // for (label, spec) in self.specs.iter() {
        //     let data = output_data
        //         .data
        //         .get(label)
        //         .ok_or_else(|| format!("Missing input data for label: {}", label))?;

        //     if data.type_id() != spec.type_id {
        //         return Err(format!(
        //             "Type mismatch for {}: expected {:?}, got {:?}",
        //             label,
        //             spec.type_id,
        //             data.type_id()
        //         ));
        //     }
        // }
        // Ok(())
    }
}
