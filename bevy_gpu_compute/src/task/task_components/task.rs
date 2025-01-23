use bevy::{
    log,
    prelude::Component,
    render::{
        render_resource::{BindGroup, BindGroupLayout, Buffer},
        renderer::RenderDevice,
    },
};
use bevy_gpu_compute_core::{
    TypeErasedArrayInputData, TypeErasedArrayOutputData, TypeErasedConfigInputData,
};
use wgpu::PipelineLayout;

use crate::task::{
    compute_pipeline::cache::PipelineLruCache,
    inputs::array_type::lengths::InputArrayDataLengths,
    task_specification::{
        gpu_workgroup_space::GpuWorkgroupSpace, task_specification::ComputeTaskSpecification,
    },
};

/**
A task can only run once per run of the BevyGpuComputeRunTaskSet system set
By default this means once per frame
*/

pub struct BuvyGpuComputeTaskBuffers {
    pub output: Vec<Buffer>,
    pub output_count: Vec<Buffer>,
    pub output_staging: Vec<Buffer>,
    pub output_count_staging: Vec<Buffer>,
    pub input: Vec<Buffer>,
    pub config_input: Vec<Buffer>,
}
impl Default for BuvyGpuComputeTaskBuffers {
    fn default() -> Self {
        BuvyGpuComputeTaskBuffers {
            output: Vec::new(),
            output_count: Vec::new(),
            output_staging: Vec::new(),
            output_count_staging: Vec::new(),
            input: Vec::new(),
            config_input: Vec::new(),
        }
    }
}
#[derive(Component)]
pub struct BevyGpuComputeTask {
    name: String,
    pub spec: ComputeTaskSpecification,
    pub pipeline_cache: PipelineLruCache,
    pub pipeline_layout: Option<wgpu::PipelineLayout>,
    pub bind_group_layout: Option<BindGroupLayout>,
    pub buffers: BuvyGpuComputeTaskBuffers,
    pub num_gpu_workgroups_required: GpuWorkgroupSpace,

    // other stuff
    pub bind_group: Option<BindGroup>,
    pub config_input_data: Option<TypeErasedConfigInputData>,
    pub input_data: Option<TypeErasedArrayInputData>,
    pub output_data: Option<TypeErasedArrayOutputData>,
    pub input_array_lengths: Option<InputArrayDataLengths>,
}

impl BevyGpuComputeTask {
    pub fn new(render_device: &RenderDevice, name: &str, spec: ComputeTaskSpecification) -> Self {
        let mut n = BevyGpuComputeTask {
            name: name.to_string(),
            spec,
            pipeline_cache: PipelineLruCache::default(),
            pipeline_layout: None,
            bind_group_layout: None,
            buffers: BuvyGpuComputeTaskBuffers::default(),
            num_gpu_workgroups_required: GpuWorkgroupSpace::default(),
            bind_group: None,
            config_input_data: None,
            input_data: None,
            output_data: None,
            input_array_lengths: None,
        };
        n.setup_static_fields(render_device);
        n
    }
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn setup_static_fields(&mut self, render_device: &RenderDevice) {
        log::info!("Setting up new tasks");
        let bind_group_layouts = self.get_bind_group_layouts(&render_device);
        let pipeline_layout = self.get_pipeline_layout(&render_device, &bind_group_layouts);
        self.bind_group_layout = Some(bind_group_layouts);
        self.pipeline_layout = Some(pipeline_layout);
    }

    fn get_pipeline_layout(
        &self,
        render_device: &RenderDevice,
        bind_group_layouts: &BindGroupLayout,
    ) -> PipelineLayout {
        let pipeline_layout =
            render_device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&self.name),
                bind_group_layouts: &[&bind_group_layouts],
                push_constant_ranges: &[],
            });
        pipeline_layout
    }
    fn get_bind_group_layouts(&self, render_device: &RenderDevice) -> BindGroupLayout {
        let config_input_spec = self.spec.config_input_metadata_spec();
        let input_spec = self.spec.input_vectors_metadata_spec();
        let output_spec = self.spec.output_vectors_metadata_spec();
        let mut layouts = Vec::new();
        config_input_spec
            .get_all_metadata()
            .iter()
            .for_each(|spec| {
                if let Some(s) = spec {
                    layouts.push(self.create_bind_group_layout_entry(
                        s.get_binding_number(),
                        true,
                        true,
                    ));
                }
            });
        input_spec.get_all_metadata().iter().for_each(|spec| {
            if let Some(s) = spec {
                layouts.push(self.create_bind_group_layout_entry(
                    s.get_binding_number(),
                    true,
                    false,
                ));
            }
        });
        output_spec.get_all_metadata().iter().for_each(|spec| {
            if let Some(s) = spec {
                layouts.push(self.create_bind_group_layout_entry(
                    s.get_binding_number(),
                    false,
                    false,
                ));
                if s.get_include_count() {
                    layouts.push(self.create_bind_group_layout_entry(
                        s.get_count_binding_number().unwrap(),
                        false,
                        false,
                    ));
                }
            }
        });
        log::info!("Layouts: {:?}", layouts);
        // Create bind group layout once
        let bind_group_layouts =
            render_device.create_bind_group_layout(Some(self.name.as_str()), &layouts);
        bind_group_layouts
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
