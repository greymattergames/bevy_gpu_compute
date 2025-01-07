use bevy::prelude::{Commands, Component, EventReader, Query, Res, ResMut, Resource};
use bytemuck::{Pod, Zeroable};

use crate::task::task_specification::{
    max_output_vector_lengths::MaxOutputVectorLengths, task_specification::TaskUserSpecification,
};

use super::{
    resource::GpuAcceleratedBevy,
    run_ids::GpuAcceleratedBevyRunIds,
    task::{
        events::GpuComputeTaskSuccessEvent,
        inputs::{
            input_data::InputData,
            input_vector_metadata_spec::{InputVectorMetadataDefinition, InputVectorsMetadataSpec},
            input_vector_types_spec::InputVectorTypesSpec,
        },
        outputs::definitions::{
            output_vector_metadata_spec::{
                OutputVectorMetadataDefinition, OutputVectorsMetadataSpec,
            },
            output_vector_types_spec::OutputVectorTypesSpec,
            type_erased_output_data::TypeErasedOutputData,
        },
        task_components::task_run_id::TaskRunId,
        task_specification::iteration_space::IterationSpace,
        wgsl_code::WgslCode,
    },
};
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Unused(u8);

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

fn create_task_example_system(
    mut commands: Commands,
    mut gpu_acc_bevy: ResMut<GpuAcceleratedBevy>,
) {
    let task_name = "example task".to_string();
    let initial_iteration_space = IterationSpace::new(100, 10, 1);
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
    let task_spec = TaskUserSpecification::new(
        InputVectorsMetadataSpec::from_input_vector_types_spec::<ExampleTaskInputType>(
            input_definitions,
        ),
        // todo, ensure that this conforms with the provided input type, right now depends on which binding numbers are set
        OutputVectorsMetadataSpec::from_output_vector_types_spec::<ExampleTaskOutputType>(
            output_definitions,
        ),
        initial_iteration_space,
        MaxOutputVectorLengths::new(vec![10, 30, 100]),
        WgslCode::from_file("./collision.wgsl", "main".to_string()), // SHOULD be alterable
    );
    let task = gpu_acc_bevy.create_task(&mut commands, &task_name, task_spec);
}
fn delete_task_example_system(
    mut commands: Commands,
    mut gpu_acc_bevy: ResMut<GpuAcceleratedBevy>,
) {
    let task = gpu_acc_bevy.task(&"example task".to_string());
    // example of deletion
    task.delete(&mut commands);
}
fn alter_task_example_system(
    mut commands: Commands,
    mut gpu_acc_bevy: ResMut<GpuAcceleratedBevy>,
    mut task_specifications: Query<&mut TaskUserSpecification>,
) {
    let task = gpu_acc_bevy.task(&"example task".to_string());
    // example of alteration
    if let Ok(mut spec) = task_specifications.get_mut(task.entity) {
        spec.set_iteration_space(
            &mut commands,
            task.entity,
            IterationSpace::new_unsafe(10, 10, 1),
        );
    }
}
fn run_task_example_system(
    mut commands: Commands,
    mut gpu_compute: ResMut<GpuAcceleratedBevy>,
    mut task_run_ids: ResMut<GpuAcceleratedBevyRunIds>,
) {
    let task = gpu_compute.task(&"example task".to_string());
    let mut input_data = InputData::<ExampleTaskInputType>::empty();
    input_data.set_input0(vec![0.3]);
    input_data.set_input1(vec![[0u8; 10]]);
    input_data.set_input2(vec![0]);
    let run_id = task.run(&mut commands, input_data, task_run_ids);
}

#[derive(Resource)]
struct RunID(pub u128);

fn handle_results_example_system_with_assurance_of_run_success(
    mut gpu_compute: ResMut<GpuAcceleratedBevy>,
    mut event_reader: EventReader<GpuComputeTaskSuccessEvent>,
    out_datas: &Query<(&TaskRunId, &TypeErasedOutputData)>,
    specific_run_id: Res<RunID>,
) {
    let task = gpu_compute.task(&"example task".to_string());
    for ev in event_reader.read() {
        if ev.id == specific_run_id.0 {
            // this ensures that the results exist
            let results = task.result::<ExampleTaskOutputType>(specific_run_id.0, out_datas);
            if let Some(results) = results {
                let result_1: Vec<[f64; 2]> = results.get_output1().unwrap().into();
                let result_0 = results.get_output0();
                // your logic here
            }
        }
    }
}
fn handle_results_example_system_no_assurances(
    mut gpu_compute: ResMut<GpuAcceleratedBevy>,
    out_datas: &Query<(&TaskRunId, &TypeErasedOutputData)>,
    specific_run_id: Res<RunID>,
) {
    let task = gpu_compute.task(&"example task".to_string());
    let result_option = task.result::<ExampleTaskOutputType>(specific_run_id.0, out_datas);
    if let Some(result) = result_option {
        let result_1: Vec<[f64; 2]> = result.get_output1().unwrap().into();
        let result_0 = result.get_output0();
        // your logic here
    }
}
