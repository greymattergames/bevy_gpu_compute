use std::collections::HashMap;

use bevy::{
    log::tracing_subscriber::filter::targets::Iter,
    prelude::{Commands, Component, EventReader, Query, Res, ResMut},
    reflect::Tuple,
    render::gpu_component_array_buffer,
};
use bytemuck::{Pod, Zeroable};

use super::{
    compute_task::{
        component::TaskRunId,
        events::GpuComputeTaskSuccessEvent,
        inputs::{
            input_data::{self, InputData},
            input_metadata_spec::{
                InputVectorMetadata, InputVectorMetadataDefinition, InputVectorMetadataSpec,
            },
            input_spec::InputVectorTypesSpec,
        },
        iteration_space_dependent_components::{
            iteration_space::IterationSpace, max_output_vector_lengths::MaxOutputVectorLengths,
        },
        outputs::{
            output_metadata_spec::{OutputVectorMetadataDefinition, OutputVectorMetadataSpec},
            output_spec::OutputVectorTypesSpec,
        },
        wgsl_code::WgslCode,
    },
    manager_resource::{GpuCompute, GpuComputeBevyTaskType},
};
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Unused(u8);

#[derive(Component)]
pub struct ExampleTaskInputType {}
impl InputVectorTypesSpec for ExampleTaskInputType {
    type Input0 = f64;
    type Input1 = [u8; 10];
    type Input2 = u8;
    type Input3 = Unused;
    type Input4 = Unused;
    type Input5 = Unused;
}
pub struct ExampleTaskOutputType {}
impl OutputVectorTypesSpec for ExampleTaskOutputType {
    type Output0 = u8;
    type Output1 = [f64; 2];
    type Output2 = Unused;
    type Output3 = Unused;
    type Output4 = Unused;
    type Output5 = Unused;
}
pub struct ExampleTaskType {}
impl GpuComputeBevyTaskType for ExampleTaskType {
    type InType = ExampleTaskInputType;
    type OutType = ExampleTaskOutputType;
}

fn system(
    commands: &mut Commands,
    mut gpu_compute: ResMut<GpuCompute>,
    mut example_task_event_reader: EventReader<
        GpuComputeTaskSuccessEvent<<ExampleTaskType as GpuComputeBevyTaskType>::OutType>,
    >,
    mut task_run_ids: Query<&mut TaskRunId>,
) {
    let task_name = "example task".to_string();
    let initial_iteration_space = IterationSpace {
        x: 100,
        y: 10,
        z: 1,
    };
    let input_definitions = [
        Some(&InputVectorMetadataDefinition { binding_number: 0 }),
        Some(&InputVectorMetadataDefinition { binding_number: 1 }),
        Some(&InputVectorMetadataDefinition { binding_number: 2 }),
        None,
        None,
        None,
    ];
    let output_definitions = [
        Some(&OutputVectorMetadataDefinition {
            binding_number: 3,
            include_count: true,
            count_binding_number: Some(5),
        }),
        Some(&OutputVectorMetadataDefinition {
            binding_number: 4,
            include_count: false,
            count_binding_number: None,
        }),
        None,
        None,
        None,
        None,
    ];
    let task = gpu_compute.create_task::<ExampleTaskType>(
        commands,
        &task_name,
        initial_iteration_space,
        WgslCode::from_file("./collision.wgsl", "main".to_string()), // SHOULD be alterable
        InputVectorMetadataSpec::from_input_vector_types_spec::<
            <ExampleTaskType as GpuComputeBevyTaskType>::InType,
        >(input_definitions),
        // todo, ensure that this conforms with the provided input type, right now depends on which binding numbers are set
        OutputVectorMetadataSpec::from_output_vector_types_spec::<
            <ExampleTaskType as GpuComputeBevyTaskType>::OutType,
        >(output_definitions),
        MaxOutputVectorLengths::from_callback(&initial_iteration_space, |iteration_space| {
            vec![10, 30, iteration_space.x as usize]
        }),
    );
    let task2 = gpu_compute.task(&task_name);
    // example of deletion
    task.delete(commands);
    // example of alteration
    // todo make per-task
    task.set_iteration_space(commands, IterationSpace { x: 10, y: 10, z: 1 });
    // example of running the compute task

    let mut input_data = InputData::<ExampleTaskInputType>::empty();
    input_data.set_input0(vec![0.3]);
    input_data.set_input1(vec![[0u8; 10]]);
    input_data.set_input2(vec![0]);
    // todo, make per-task
    let run_id = task.run::<ExampleTaskType>(commands, input_data, task_run_ids);

    for results in example_task_event_reader.read() {
        // todo handle the resuts
        if results.id == run_id {
            // todo handle the results
            let result_1 = results.data.get_output1();
        }
    }
}
