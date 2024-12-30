use std::any::Any;

use bytemuck::Pod;

pub fn create_buffer_view_converter()
-> Box<dyn Fn(&[u8]) -> Box<dyn Any + Send + Sync> + Send + Sync> {
    Box::new(|data: &[u8]| -> Box<dyn Any + Send + Sync> { Box::new(data[0] as u32) })
}

pub struct BufferViewConverter {
    data: Vec<u8>,
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
    pub fn get<T: Pod>(&self) -> Option<T> {
        // Some(bytemuck::from_bytes(&self.data))
        if self.data.len() != std::mem::size_of::<T>() {
            return None;
        }
        Some(bytemuck::pod_read_unaligned(&self.data))
    }
}
