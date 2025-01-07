use bevy::prelude::Resource;
use sysinfo::System;

#[derive(Resource)]
pub struct RamLimit {
    pub total_mem: u64,
}

impl Default for RamLimit {
    fn default() -> Self {
        let mut sys = System::new_all();
        // First we update all information of our `System` struct.
        sys.refresh_all();
        RamLimit {
            total_mem: sys.total_memory(),
        }
    }
}
