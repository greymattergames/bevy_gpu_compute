
use shared::wgsl_shader_module::IterSpaceDimmension;

use super::iteration_space::IterationSpace;

#[derive(Clone, PartialEq, Debug)]
/// Defaults should generally not be altered. Based on this resource: https://developer.arm.com/documentation/101897/0303/Compute-shading/Workgroup-sizes
pub struct GpuWorkgroupSizes {
    x: usize,
    y: usize,
    z: usize,
    num_dimmensions: usize,
}

impl Default for GpuWorkgroupSizes {
    fn default() -> Self {
        Self {
            x: 64,
            y: 1,
            z: 1,
            num_dimmensions: 1,
        }
    }
}

impl GpuWorkgroupSizes {
    pub fn num_dimmensions(&self) -> usize {
        self.num_dimmensions
    }
    pub fn from_iter_space(iter_space: &IterationSpace) -> Self {
        let num_dimmensions = iter_space.num_dimmensions();
        if num_dimmensions == IterSpaceDimmension::ThreeD {
            Self {
                x: 4,
                y: 4,
                z: 4,
                num_dimmensions: 3,
            }
        } else if num_dimmensions == IterSpaceDimmension::TwoD {
            Self {
                x: 8,
                y: 8,
                z: 1,
                num_dimmensions: 2,
            }
        } else {
            Self {
                x: 64,
                y: 1,
                z: 1,
                num_dimmensions: 1,
            }
        }
    }
    pub fn three_d() -> Self {
        Self {
            x: 4,
            y: 4,
            z: 4,
            num_dimmensions: 3,
        }
    }
    pub fn two_d() -> Self {
        Self {
            x: 8,
            y: 8,
            z: 1,
            num_dimmensions: 2,
        }
    }
    pub fn one_d() -> Self {
        Self {
            x: 64,
            y: 1,
            z: 1,
            num_dimmensions: 1,
        }
    }
    pub fn custom_use_at_own_risk(x: usize, y: usize, z: usize, num_dimmensions: usize) -> Self {
        Self {
            x,
            y,
            z,
            num_dimmensions,
        }
    }
    pub fn x(&self) -> usize {
        self.x
    }
    pub fn y(&self) -> usize {
        self.y
    }
    pub fn z(&self) -> usize {
        self.z
    }
}
