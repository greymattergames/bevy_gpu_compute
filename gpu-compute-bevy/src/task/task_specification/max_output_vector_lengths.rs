use std::{collections::HashMap, time::SystemTime};

use bevy::prelude::Component;

#[derive(Debug, Clone, PartialEq)]
/**
### These vectors lengths are very important for overall performance, the lower the better
#### But if they are too low they will cut off valid output data

*/
pub struct MaxOutputLengths {
    pub unique_id: usize,
    map: HashMap<String, usize>,
}
impl Default for MaxOutputLengths {
    fn default() -> Self {
        Self {
            unique_id: 0,
            map: HashMap::default(),
        }
    }
}

impl MaxOutputLengths {
    pub fn new(map: HashMap<String, usize>) -> Self {
        Self {
            unique_id: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as usize,
            map: map,
        }
    }

    pub fn get(&self, output_item_type_name: String) -> usize {
        return self.map[&output_item_type_name];
    }
    pub fn get_map(&self) -> &HashMap<String, usize> {
        return &self.map;
    }
}
