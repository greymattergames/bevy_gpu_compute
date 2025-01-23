use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Default)]
pub struct InputArrayDataLengths {
    lengths_by_input_array_type_name: HashMap<String, usize>,
    hash: u64,
}

impl InputArrayDataLengths {
    pub fn new(lengths_by_input_array_type_name: HashMap<String, usize>) -> Self {
        let hash = Self::hash_map(&lengths_by_input_array_type_name);

        InputArrayDataLengths {
            lengths_by_input_array_type_name,
            hash,
        }
    }
    pub fn hash_map(map: &HashMap<String, usize>) -> u64 {
        let mut hasher = DefaultHasher::new();
        for (key, value) in map {
            key.hash(&mut hasher);
            value.hash(&mut hasher);
        }
        let hash = hasher.finish();
        hash
    }
    pub fn get(&self, input_array_type_name: &str) -> Option<&usize> {
        self.lengths_by_input_array_type_name
            .get(input_array_type_name)
    }
    pub fn update_and_return_new_hash_if_changed(
        &mut self,
        new_lengths_by_input_array_type_name: HashMap<String, usize>,
    ) -> Option<u64> {
        let new_hash = Self::hash_map(&new_lengths_by_input_array_type_name);
        if new_hash == self.hash {
            return None;
        } else {
            self.lengths_by_input_array_type_name = new_lengths_by_input_array_type_name;
            self.hash = new_hash;
            return Some(new_hash);
        }
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }
}
