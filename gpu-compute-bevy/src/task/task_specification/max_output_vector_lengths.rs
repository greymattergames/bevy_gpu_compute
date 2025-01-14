use std::time::SystemTime;

use bevy::prelude::Component;

#[derive(Debug, Clone)]
/**
### These vectors lengths are very important for overall performance, the lower the better
#### But if they are too low they will cut off valid output data

*/
pub struct MaxOutputVectorLengths {
    pub unique_id: usize,
    map: Vec<usize>,
}
impl Default for MaxOutputVectorLengths {
    fn default() -> Self {
        Self {
            unique_id: 0,
            map: Vec::default(),
        }
    }
}

impl MaxOutputVectorLengths {
    pub fn new(map: Vec<usize>) -> Self {
        Self {
            unique_id: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as usize,
            map: map,
        }
    }

    pub fn get(&self, output_index: usize) -> usize {
        return self.map[output_index];
    }
}
