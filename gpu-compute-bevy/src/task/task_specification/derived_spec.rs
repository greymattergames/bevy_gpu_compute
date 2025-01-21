use crate::task::task_components::task_max_output_bytes::TaskMaxOutputBytes;

use super::{gpu_workgroup_sizes::GpuWorkgroupSizes, gpu_workgroup_space::GpuWorkgroupSpace};

#[derive(Default, Debug)]
pub struct ComputeTaskDerivedSpec {
    gpu_workgroup_space: GpuWorkgroupSpace,
    task_max_output_bytes: TaskMaxOutputBytes,
    workgroup_sizes: GpuWorkgroupSizes,
}

impl ComputeTaskDerivedSpec {
    pub fn new(
        gpu_workgroup_space: GpuWorkgroupSpace,
        task_max_output_bytes: TaskMaxOutputBytes,
        workgroup_sizes: GpuWorkgroupSizes,
    ) -> Self {
        ComputeTaskDerivedSpec {
            gpu_workgroup_space,
            task_max_output_bytes,
            workgroup_sizes,
        }
    }
    pub fn gpu_workgroup_space(&self) -> &GpuWorkgroupSpace {
        &self.gpu_workgroup_space
    }
    pub fn task_max_output_bytes(&self) -> &TaskMaxOutputBytes {
        &self.task_max_output_bytes
    }
    pub fn workgroup_sizes(&self) -> &GpuWorkgroupSizes {
        &self.workgroup_sizes
    }
    pub fn _lib_only_set_task_max_output_bytes(
        &mut self,
        new_task_max_output_bytes: TaskMaxOutputBytes,
    ) {
        self.task_max_output_bytes = new_task_max_output_bytes;
    }
    pub fn _lib_only_set_gpu_workgroup_space(
        &mut self,
        new_gpu_workgroup_space: GpuWorkgroupSpace,
    ) {
        self.gpu_workgroup_space = new_gpu_workgroup_space;
    }
    pub fn _lib_only_set_workgroup_sizes(&mut self, new_workgroup_sizes: GpuWorkgroupSizes) {
        self.workgroup_sizes = new_workgroup_sizes;
    }
}
