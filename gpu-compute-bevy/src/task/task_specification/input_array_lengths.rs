use std::collections::HashMap;

#[derive(Default, Debug)]

pub struct ComputeTaskInputArrayLengths {
    pub by_index: [Option<usize>; 6],
}
