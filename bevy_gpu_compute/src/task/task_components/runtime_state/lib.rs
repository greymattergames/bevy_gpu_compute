use bevy::{
    log,
    render::{
        render_resource::{BindGroup, BindGroupLayout},
        renderer::RenderDevice,
    },
};
use wgpu::PipelineLayout;

use crate::task::{
    compute_pipeline::pipeline_cache::PipelineLruCache,
    task_components::configuration::lib::TaskConfiguration,
};

use super::{
    gpu_workgroup_sizes::GpuWorkgroupSizes, gpu_workgroup_space::GpuWorkgroupSpace,
    max_output_bytes::MaxOutputBytes,
};

pub struct TaskRuntimeState {
    workgroup_sizes: GpuWorkgroupSizes,
    workgroup_space: GpuWorkgroupSpace,
    max_output_bytes: MaxOutputBytes,
    pipeline_cache: PipelineLruCache,
    bind_group: Option<BindGroup>,
    bind_group_layout: BindGroupLayout,
    pipeline_layout: PipelineLayout,
}

impl TaskRuntimeState {
    pub fn new(
        workgroup_sizes: GpuWorkgroupSizes,
        workgroup_space: GpuWorkgroupSpace,
        max_output_bytes: MaxOutputBytes,
        pipeline_cache: PipelineLruCache,
        bind_group: Option<BindGroup>,
        bind_group_layout: BindGroupLayout,
        pipeline_layout: PipelineLayout,
    ) -> Self {
        TaskRuntimeState {
            workgroup_sizes,
            workgroup_space,
            max_output_bytes,
            pipeline_cache,
            bind_group,
            bind_group_layout,
            pipeline_layout,
        }
    }
    pub fn workgroup_space(&self) -> &GpuWorkgroupSpace {
        &self.workgroup_space
    }
    pub fn max_output_bytes(&self) -> &MaxOutputBytes {
        &self.max_output_bytes
    }
    pub fn workgroup_sizes(&self) -> &GpuWorkgroupSizes {
        &self.workgroup_sizes
    }
    pub fn pipeline_cache(&self) -> &PipelineLruCache {
        &self.pipeline_cache
    }
    pub fn pipeline_cache_mut(&mut self) -> &mut PipelineLruCache {
        &mut self.pipeline_cache
    }
    pub fn bind_group(&self) -> &Option<BindGroup> {
        &self.bind_group
    }
    pub fn bind_group_mut(&mut self) -> &mut Option<BindGroup> {
        &mut self.bind_group
    }
    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }
    pub fn pipeline_layout(&self) -> &PipelineLayout {
        &self.pipeline_layout
    }
    pub fn _internal_set_max_output_bytes(&mut self, new_max_output_bytes: MaxOutputBytes) {
        self.max_output_bytes = new_max_output_bytes;
    }
    pub fn _internal_set_workgroup_space(&mut self, new_gpu_workgroup_space: GpuWorkgroupSpace) {
        self.workgroup_space = new_gpu_workgroup_space;
    }
    pub fn _internal_set_workgroup_sizes(&mut self, new_workgroup_sizes: GpuWorkgroupSizes) {
        self.workgroup_sizes = new_workgroup_sizes;
    }
}

pub struct TaskRuntimeStateBuilder<'a> {
    task_name: &'a str,
    render_device: &'a RenderDevice,
    task_configuration: &'a TaskConfiguration,
}

impl<'a> TaskRuntimeStateBuilder<'a> {
    pub fn new(
        render_device: &'a RenderDevice,
        task_name: &'a str,
        task_configuration: &'a TaskConfiguration,
    ) -> Self {
        TaskRuntimeStateBuilder {
            render_device,
            task_name,
            task_configuration,
        }
    }
    pub fn build(&mut self) -> TaskRuntimeState {
        let workgroup_sizes =
            GpuWorkgroupSizes::from_iter_space(self.task_configuration.iteration_space());
        let workgroup_space = GpuWorkgroupSpace::from_iter_space_and_wrkgrp_sizes(
            self.task_configuration.iteration_space(),
            &workgroup_sizes,
        );
        let max_output_bytes = MaxOutputBytes::from_max_lengths_and_spec(
            self.task_configuration.outputs().max_lengths(),
            self.task_configuration.outputs().arrays(),
        );
        let pipeline_cache = PipelineLruCache::default();
        let bind_group = None;
        let (bind_group_layout, pipeline_layout) = self.setup_static_runtime_state();
        TaskRuntimeState::new(
            workgroup_sizes,
            workgroup_space,
            max_output_bytes,
            pipeline_cache,
            bind_group,
            bind_group_layout,
            pipeline_layout,
        )
    }
    pub fn setup_static_runtime_state(&mut self) -> (BindGroupLayout, PipelineLayout) {
        let bind_group_layout = self.get_bind_group_layouts();
        let pipeline_layout = self.get_pipeline_layout(&bind_group_layout);
        (bind_group_layout, pipeline_layout)
    }

    fn get_pipeline_layout(&self, bind_group_layout: &BindGroupLayout) -> PipelineLayout {
        self.render_device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(self.task_name),
                bind_group_layouts: &[bind_group_layout],
                push_constant_ranges: &[],
            })
    }
    fn get_bind_group_layouts(&self) -> BindGroupLayout {
        let mut layouts = Vec::new();
        self.task_configuration
            .inputs()
            .configs()
            .iter()
            .for_each(|metadata| {
                layouts.push(self.create_bind_group_layout_entry(
                    metadata.binding_number,
                    true,
                    true,
                ));
            });
        self.task_configuration
            .inputs()
            .arrays()
            .iter()
            .for_each(|metadata| {
                layouts.push(self.create_bind_group_layout_entry(
                    metadata.binding_number,
                    true,
                    false,
                ));
            });
        self.task_configuration
            .outputs()
            .arrays()
            .iter()
            .for_each(|metadata| {
                layouts.push(self.create_bind_group_layout_entry(
                    metadata.binding_number,
                    false,
                    false,
                ));
                if metadata.include_count {
                    layouts.push(self.create_bind_group_layout_entry(
                        metadata.count_binding_number.unwrap(),
                        false,
                        false,
                    ));
                }
            });
        log::debug!("Bind group layouts: {:?}", layouts);
        self.render_device
            .create_bind_group_layout(Some(self.task_name), &layouts)
    }

    fn create_bind_group_layout_entry(
        &self,
        binding_number: u32,
        is_input: bool,
        is_uniform: bool,
    ) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: binding_number,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: if is_uniform {
                    wgpu::BufferBindingType::Uniform {}
                } else {
                    wgpu::BufferBindingType::Storage {
                        read_only: is_input,
                    }
                },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None, //only for textures
        }
    }
}
