use std::any::Any;

use bevy::prelude::Component;
use bytemuck::Pod;
use wgpu::BufferView;

use super::wgsl_processable_types::WgslCollisionResult;

pub struct GpuAccBevyComputeTaskOutputSpec {
    // each has a string label
    // each has an optional bind number for correct association in the wgsl, otherwise they are numbered in order, with inputs coming before outputs
    // each has an optional count output variable, which is used to determine how much memory is ACTUALLY needed for the output buffer (instead of just using the maximum)
    pub label: String,
    pub binding_number: Option<u32>,
    pub item_bytes: usize,
    pub converter: Box<dyn Fn(&[u8]) -> Box<dyn Any + Send + Sync> + Send + Sync>,
}

#[derive(Component)]
pub struct GpuAccBevyComputeTaskOutputSpecs(pub Vec<GpuAccBevyComputeTaskOutputSpec>);

// testing its use

fn testFunc() {
    //  I want to convert an array of structs
    struct ResultItem {
        value: u32,
        value2: f32,
    }
    struct Results {
        results: [ResultItem],
    }
    let output_spec = GpuAccBevyComputeTaskOutputSpec {
        label: "test".to_string(),
        binding_number: Some(0),
        item_bytes: 4,
        converter: create_buffer_view_converter(),
    };
}
pub fn create_buffer_view_converter()
-> Box<dyn Fn(&[u8]) -> Box<dyn Any + Send + Sync> + Send + Sync> {
    Box::new(|data: &[u8]| -> Box<dyn Any + Send + Sync> { Box::new(data[0] as u32) })
}

// pub struct BufferViewConverter {
//     data: Box<dyn Any + Send + Sync>,
//     // pub converter: Box<dyn Fn(&[u8]) -> Box<dyn Any + Send + Sync> + Send + Sync>,
// }
// impl BufferViewConverter {
//     pub fn new(buffer_view: &BufferView<'_>) -> Self {
//         let converter = create_buffer_view_converter();
//         let data = converter(&buffer_view);
//         BufferViewConverter { data }
//     }
//     pub fn get<T: 'static>(&self) -> Option<&T> {
//         self.data.downcast_ref::<T>()
//     }
// }

pub struct BufferViewConverter {
    data: Vec<u8>, // Own the data instead of borrowing it
}

impl BufferViewConverter {
    pub fn new(buffer_view: &[u8]) -> Self {
        // Create a copy of the data
        let data = buffer_view.to_vec();
        BufferViewConverter { data }
    }

    pub fn get_vec<T: Pod>(&self) -> Option<Vec<T>> {
        Some(bytemuck::cast_slice(&self.data).to_vec())
    }
    pub fn get<T: Pod>(&self) -> Option<&T> {
        Some(bytemuck::from_bytes(&self.data))
    }
}
