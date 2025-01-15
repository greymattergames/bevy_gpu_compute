use std::collections::HashMap;

use bevy::{
    prelude::{Commands, Component, Entity},
    utils::RandomState,
};
use shared::wgsl_components::{
    WORKGROUP_SIZE_X_VAR_NAME, WORKGROUP_SIZE_Y_VAR_NAME, WORKGROUP_SIZE_Z_VAR_NAME,
};

use crate::task::{
    events::{GpuComputeTaskChangeEvent, PipelineConstChangedEvent},
    task_specification::{
        gpu_workgroup_sizes::GpuWorkgroupSizes, max_output_vector_lengths::MaxOutputLengths,
    },
};

pub struct PipelineConstants {
    workgroup_sizes: GpuWorkgroupSizes,
    input_array_lengths: HashMap<String, f64>,
    output_array_lengths: MaxOutputLengths,
    version: u64,
}
impl PipelineConstants {
    pub fn empty() -> Self {
        Self {
            workgroup_sizes: GpuWorkgroupSizes::default(),
            input_array_lengths: HashMap::default(),
            output_array_lengths: MaxOutputLengths::default(),
            version: 0,
        }
    }
    pub fn set_workgroup_sizes(
        &mut self,
        commands: &mut Commands,
        entity: Entity,
        workgroup_sizes: GpuWorkgroupSizes,
    ) {
        if workgroup_sizes == self.workgroup_sizes {
            return;
        }
        self.workgroup_sizes = workgroup_sizes;
        self.version += 1;
        commands.send_event(PipelineConstChangedEvent::new(entity));
    }
    pub fn set_input_array_lengths(
        &mut self,
        commands: &mut Commands,
        entity: Entity,
        input_array_lengths: HashMap<String, f64>,
    ) {
        if input_array_lengths == self.input_array_lengths {
            return;
        }
        self.input_array_lengths = input_array_lengths;
        self.version += 1;
        commands.send_event(PipelineConstChangedEvent::new(entity));
    }
    pub fn set_output_array_lengths(
        &mut self,
        commands: &mut Commands,
        entity: Entity,
        output_array_lengths: MaxOutputLengths,
    ) {
        if output_array_lengths == self.output_array_lengths {
            return;
        }
        self.output_array_lengths = output_array_lengths;
        self.version += 1;
        commands.send_event(PipelineConstChangedEvent::new(entity));
    }
    pub fn version(&self) -> u64 {
        self.version
    }
    pub fn get_output_array_lengths(&self) -> &MaxOutputLengths {
        &self.output_array_lengths
    }
    pub fn get_input_array_lengths(&self) -> &HashMap<String, f64> {
        &self.input_array_lengths
    }
    pub fn get_workgroup_sizes(&self) -> &GpuWorkgroupSizes {
        &self.workgroup_sizes
    }

    pub fn as_hashmap(&self) -> HashMap<String, f64> {
        let mut n: HashMap<String, f64> = HashMap::new();

        n.insert(
            WORKGROUP_SIZE_X_VAR_NAME.to_string(),
            self.workgroup_sizes.x() as f64,
        );
        n.insert(
            WORKGROUP_SIZE_Y_VAR_NAME.to_string(),
            self.workgroup_sizes.y() as f64,
        );
        n.insert(
            WORKGROUP_SIZE_Z_VAR_NAME.to_string(),
            self.workgroup_sizes.z() as f64,
        );
        // input and output array lengths
        for (k, v) in &self.input_array_lengths {
            n.insert(k.clone(), *v);
        }
        for (k, v) in self.output_array_lengths.get_map() {
            n.insert(k.to_string(), *v as f64);
        }
        n
    }
}
