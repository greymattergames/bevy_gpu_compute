use std::collections::HashMap;

use bevy::{
    log::tracing_subscriber::filter::targets::Iter,
    prelude::{Commands, Component, EventReader, Query, Res, ResMut, Resource},
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
            output_data::TypeErasedOutputData,
            output_metadata_spec::{OutputVectorMetadataDefinition, OutputVectorMetadataSpec},
            output_spec::OutputVectorTypesSpec,
        },
        wgsl_code::WgslCode,
    },
    manager_resource::{GpuAcceleratedBevyRunIds, GpuCompute, GpuComputeBevyTaskType},
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

    mut task_run_ids: ResMut<GpuAcceleratedBevyRunIds>,
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
    task.set_iteration_space(commands, IterationSpace { x: 10, y: 10, z: 1 });
    // example of running the compute task

    let mut input_data = InputData::<ExampleTaskInputType>::empty();
    input_data.set_input0(vec![0.3]);
    input_data.set_input1(vec![[0u8; 10]]);
    input_data.set_input2(vec![0]);
    let cross_task_run_id = task.run::<ExampleTaskType>(commands, input_data, task_run_ids);
}

#[derive(Resource)]
struct RunID(pub u128);
fn example_results_handling_system(
    mut gpu_compute: ResMut<GpuCompute>,
    mut event_reader: EventReader<GpuComputeTaskSuccessEvent>,
    out_datas: &Query<(&TaskRunId, &TypeErasedOutputData)>,
    specific_run_id: Res<RunID>,
) {
    let task = gpu_compute.task(&"example task".to_string());
    for ev in event_reader.read() {
        if ev.id == specific_run_id.0 {
            // this ensures that the results exist
            let results = task.result::<ExampleTaskType>(specific_run_id.0, out_datas);
            if let Some(results) = results {
                let result_1: Vec<[f64; 2]> = results.get_output1().unwrap().into();
                let result_0 = results.get_output0();
                // handle
            }
        }
    }
}
