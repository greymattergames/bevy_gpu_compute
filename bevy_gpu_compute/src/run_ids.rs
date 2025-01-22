use bevy::prelude::Resource;

#[derive(Resource)]

pub struct BevyGpuComputeRunIds {
    last_id: u128,
}
impl Default for BevyGpuComputeRunIds {
    fn default() -> Self {
        BevyGpuComputeRunIds { last_id: 0 }
    }
}
impl BevyGpuComputeRunIds {
    pub fn get_next(&mut self) -> u128 {
        self.last_id += 1;
        self.last_id
    }
}
