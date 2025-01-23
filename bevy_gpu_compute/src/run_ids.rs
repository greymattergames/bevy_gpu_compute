use bevy::prelude::Component;

#[derive(Component)]

pub struct BevyGpuComputeRunIds {
    last_id: u128,
}
impl Default for BevyGpuComputeRunIds {
    fn default() -> Self {
        BevyGpuComputeRunIds { last_id: 0 }
    }
}
impl BevyGpuComputeRunIds {
    pub fn increment(&mut self) {
        self.last_id += 1;
    }
    pub fn get(&self) -> u128 {
        self.last_id
    }
}
