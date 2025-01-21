use bevy::prelude::Resource;

#[derive(Resource)]

pub struct GpuAcceleratedBevyRunIds {
    last_id: u128,
}
impl Default for GpuAcceleratedBevyRunIds {
    fn default() -> Self {
        GpuAcceleratedBevyRunIds { last_id: 0 }
    }
}
impl GpuAcceleratedBevyRunIds {
    pub fn get_next(&mut self) -> u128 {
        self.last_id += 1;
        self.last_id
    }
}
