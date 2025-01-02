use bevy::prelude::Component;

/**
 * Dependent on IterationSpace and WorkgroupSizes
 */
#[derive(Component)]
pub struct GpuWorkgroupSpace {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}
impl Default for GpuWorkgroupSpace {
    fn default() -> Self {
        Self { x: 1, y: 1, z: 1 }
    }
}
