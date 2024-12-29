use bevy::prelude::ResMut;

use crate::code::gpu_power_user::multi_batch_manager::resources::GpuCollisionBatchManager;

pub fn finish_batch(mut batch_manager: ResMut<GpuCollisionBatchManager>) {
    batch_manager.current_batch_job += 1;
}
