use std::hash::{Hash, Hasher};

use shared::wgsl_shader_module::IterSpaceDimmension;

#[derive(Hash, Copy, Debug, Clone)]
/**
Repersenents the max values of the iterators in wgsl for each dimmension.

For example:
```wgsl
@compute @workgroup_size(WORKGROUP_SIZE)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let current_x = global_id.x; // will be less than or equal to IterationSpace.x
    let current_y = global_id.y; // will be less than or equal to IterationSpace.y
    let current_z = global_id.z; // will be less than or equal to IterationSpace.z
```
*/
pub struct IterationSpace {
    x: usize,
    y: usize,
    z: usize,
    num_dimmensions: IterSpaceDimmension,
}
impl Default for IterationSpace {
    fn default() -> Self {
        IterationSpace::new_unsafe(1, 1, 1)
    }
}

impl IterationSpace {
    /// faster, but with no input validation, make sure each dimmension is greater than 0
    pub fn new_unsafe(x: usize, y: usize, z: usize) -> Self {
        let num_dimmensions = if z > 1 {
            IterSpaceDimmension::ThreeD
        } else if y > 1 {
            IterSpaceDimmension::TwoD
        } else {
            IterSpaceDimmension::OneD
        };
        IterationSpace {
            x,
            y,
            z,
            num_dimmensions,
        }
    }
    /// checks if each dimmension is greater than 0
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        if x == 0 || y == 0 || z == 0 {
            panic!("Each dimmension must be greater than 0");
        }
        let num_dimmensions = if x > 1 && y > 1 && z > 1 {
            IterSpaceDimmension::ThreeD
        } else if x > 1 && y > 1 {
            IterSpaceDimmension::TwoD
        } else {
            IterSpaceDimmension::OneD
        };
        IterationSpace {
            x,
            y,
            z,
            num_dimmensions,
        }
    }
    /// used for pipeline cache
    pub fn get_hash(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
    pub fn num_dimmensions(&self) -> IterSpaceDimmension {
        self.num_dimmensions
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
