use crate::task::task_components::configuration::iteration_space::IterationSpace;

use super::gpu_workgroup_sizes::GpuWorkgroupSizes;

/**
 * Dependent on IterationSpace and WorkgroupSizes
 */
#[derive(Debug)]
pub struct GpuWorkgroupSpace {
    x: u32,
    y: u32,
    z: u32,
}

impl GpuWorkgroupSpace {
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z }
    }
    pub fn from_iter_space_and_wrkgrp_sizes(
        iter_space: &IterationSpace,
        wg_sizes: &GpuWorkgroupSizes,
    ) -> Self {
        let x = (iter_space.x() as f32 / wg_sizes.x() as f32).ceil() as u32;
        let y = (iter_space.y() as f32 / wg_sizes.y() as f32).ceil() as u32;
        let z = (iter_space.z() as f32 / wg_sizes.z() as f32).ceil() as u32;
        Self::new(x, y, z)
    }
    pub fn x(&self) -> u32 {
        self.x
    }
    pub fn y(&self) -> u32 {
        self.y
    }
    pub fn z(&self) -> u32 {
        self.z
    }
}
