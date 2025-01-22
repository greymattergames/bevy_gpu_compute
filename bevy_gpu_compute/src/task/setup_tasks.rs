use bevy::{
    log,
    prelude::{Commands, EventReader, Query, Res},
    render::{render_resource::BindGroupLayout, renderer::RenderDevice},
};
use wgpu::PipelineLayout;

use super::{
    compute_pipeline::pipeline_layout::PipelineLayoutComponent,
    events::GpuAcceleratedTaskCreatedEvent,
    inputs::{
        array_type::input_vector_metadata_spec::InputVectorsMetadataSpec,
        config_type::config_input_metadata_spec::ConfigInputsMetadataSpec,
    },
    outputs::definitions::output_vector_metadata_spec::OutputVectorsMetadataSpec,
    task_components::bind_group_layouts::BindGroupLayouts,
    task_specification::task_specification::ComputeTaskSpecification,
};

pub fn setup_new_tasks(
    mut commands: Commands,
    mut event_reader: EventReader<GpuAcceleratedTaskCreatedEvent>,
    specs: Query<&ComputeTaskSpecification>,
    render_device: Res<RenderDevice>,
) {
    log::info!("Setting up new tasks");
    event_reader.read().for_each(|ev| {
        let mut e_c = commands.entity(ev.entity);
        let spec = specs.get(ev.entity).unwrap();
        let bind_group_layouts = get_bind_group_layouts(
            &ev.task_name,
            &render_device,
            &spec.config_input_metadata_spec(),
            &spec.input_vectors_metadata_spec(),
            &spec.output_vectors_metadata_spec(),
        );
        let pipeline_layout =
            get_pipeline_layout(&ev.task_name, &render_device, &bind_group_layouts);
        log::info!("Task {} setup", ev.task_name);
        log::info!("Bind group layouts: {:?}", bind_group_layouts);
        log::info!("Pipeline layout: {:?}", pipeline_layout);
        e_c.insert(BindGroupLayouts(bind_group_layouts));
        e_c.insert(PipelineLayoutComponent(pipeline_layout));
    });
}

fn get_pipeline_layout(
    task_name: &str,
    render_device: &RenderDevice,
    bind_group_layouts: &BindGroupLayout,
) -> PipelineLayout {
    let pipeline_layout = render_device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(task_name),
        bind_group_layouts: &[&bind_group_layouts],
        push_constant_ranges: &[],
    });
    pipeline_layout
}
fn get_bind_group_layouts(
    task_name: &str,
    render_device: &RenderDevice,
    config_input_spec: &ConfigInputsMetadataSpec,
    input_spec: &InputVectorsMetadataSpec,
    output_spec: &OutputVectorsMetadataSpec,
) -> BindGroupLayout {
    let mut layouts = Vec::new();
    config_input_spec
        .get_all_metadata()
        .iter()
        .for_each(|spec| {
            if let Some(s) = spec {
                layouts.push(create_bind_group_layout_entry(
                    s.get_binding_number(),
                    true,
                    true,
                ));
            }
        });
    input_spec.get_all_metadata().iter().for_each(|spec| {
        if let Some(s) = spec {
            layouts.push(create_bind_group_layout_entry(
                s.get_binding_number(),
                true,
                false,
            ));
        }
    });
    output_spec.get_all_metadata().iter().for_each(|spec| {
        if let Some(s) = spec {
            layouts.push(create_bind_group_layout_entry(
                s.get_binding_number(),
                false,
                false,
            ));
            if s.get_include_count() {
                layouts.push(create_bind_group_layout_entry(
                    s.get_count_binding_number().unwrap(),
                    false,
                    false,
                ));
            }
        }
    });
    log::info!("Layouts: {:?}", layouts);
    // Create bind group layout once
    let bind_group_layouts = render_device.create_bind_group_layout(Some(task_name), &layouts);
    bind_group_layouts
}

fn create_bind_group_layout_entry(
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
            min_binding_size: None, //todo, this should be pre-calculated for performance reasons
        },
        count: None, //only for textures
    }
}
