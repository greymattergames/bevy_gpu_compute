use bevy::{
    DefaultPlugins,
    app::App,
    prelude::{Commands, Component, EventReader, Query, Res, ResMut, Resource},
};
use bytemuck::{Pod, Zeroable};
use gpu_compute_bevy::{
    resource::GpuAcceleratedBevy,
    run_ids::GpuAcceleratedBevyRunIds,
    task::{
        events::GpuComputeTaskSuccessEvent,
        inputs::{
            input_data::InputData,
            input_vector_metadata_spec::{InputVectorMetadataDefinition, InputVectorsMetadataSpec},
        },
        outputs::definitions::{
            output_vector_metadata_spec::{
                OutputVectorMetadataDefinition, OutputVectorsMetadataSpec,
            },
            type_erased_output_data::TypeErasedOutputData,
        },
        task_components::task_run_id::TaskRunId,
        task_specification::{
            iteration_space::IterationSpace, max_output_vector_lengths::MaxOutputVectorLengths,
            task_specification::ComputeTaskSpecification,
        },
        wgsl_code::WgslCode,
    },
};

use rust_to_wgsl::*;
use shared::{
    misc_types::{InputVectorTypesSpec, OutputVectorTypesSpec},
    wgsl_in_rust_helpers::*,
};

fn main() {
    // todo setup minimum functioning bevy example
    let mut binding = App::new();
    let _app = binding
        .add_plugins(DefaultPlugins)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(PerformanceMetrics::new(run_config.num_frames_to_test))
        .init_resource::<SysInfo>()
        .insert_resource(run_config.clone())
        .add_plugins(GraphicsPlugin)
        .add_systems(
            Startup,
            (setup, spawn_entities, setup_position_cache).chain(),
        )
        .add_plugins(CollisionDetectionPlugin {
            method: collision_detection_type,
            run_config,
        })
        .add_systems(PreUpdate, (move_entities_deterministic,).chain())
        .add_systems(
            Update,
            (process_collisions, track_performance_and_exit).chain(),
        )
        .run();
}

#[wgsl_shader_module]
mod collision_detection_module {
    use rust_to_wgsl::*;
    use shared::wgsl_in_rust_helpers::*;

    /// unused, just for demonstration
    const MY_CONST: bool = true;
    /// unused, just for demonstration
    #[wgsl_config]
    struct Config {
        time: f32,
        resolution: Vec2F32,
    }
    #[wgsl_input_array]
    struct Position {
        //todo, check that the 'pub' is either removed or valid in wgsl, is necessary in rust
        pub v: Vec2F32,
    }
    #[wgsl_input_array]
    type Radius = f32;
    #[wgsl_output_vec]
    struct CollisionResult {
        entity1: u32,
        entity2: u32,
    }
    fn calculate_distance_squared(p1: Vec2F32, p2: Vec2F32) -> f32 {
        let dx = p1.x - p2[0];
        let dy = p1.y - p2[1];
        return dx * dx + dy * dy;
    }
    fn main(iter_pos: WgslIterationPosition) {
        let current_entity = iter_pos.x;
        let other_entity = iter_pos.y;
        // Early exit conditions
        let out_of_bounds = current_entity >= WgslVecInput::vec_len::<Position>()
            || other_entity >= WgslVecInput::vec_len::<Position>();
        if out_of_bounds || current_entity == other_entity || current_entity >= other_entity {
            return;
        }
        let current_radius = WgslVecInput::vec_val::<Radius>(current_entity);
        let other_radius = WgslVecInput::vec_val::<Radius>(other_entity);
        if current_radius <= 0.0 || other_radius <= 0.0 {
            return;
        }
        let current_pos = WgslVecInput::vec_val::<Position>(current_entity);
        let other_pos = WgslVecInput::vec_val::<Position>(other_entity);
        let dist_squared = calculate_distance_squared(current_pos.v, other_pos.v);
        let radius_sum = current_radius + other_radius;
        if dist_squared < radius_sum * radius_sum {
            WgslOutput::push::<CollisionResult>(CollisionResult {
                entity1: current_entity,
                entity2: other_entity,
            });
        }
    }
}

fn example_task_creation_system(
    mut commands: Commands,
    mut gpu_acc_bevy: ResMut<GpuAcceleratedBevy>,
) {
    let task_name = "example task".to_string();
    let initial_iteration_space = IterationSpace::new(100, 10, 1);
    let initial_max_output_lengths = MaxOutputVectorLengths::new(vec![10, 30, 100]);
    let task_spec = ComputeTaskSpecification::from_shader::<collision_detection_module::Types>(
        collision_detection_module::parsed(),
        initial_iteration_space,
        initial_max_output_lengths.clone(),
    );
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
    mut task_specifications: Query<&mut ComputeTaskSpecification>,
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
    let mut input_data = InputData::<collision_detection_module::Types>::empty();
    input_data.set_input0(vec![collision_detection_module::Position {
        v: Vec2F32::new(0.3, 0.3),
    }]);
    input_data.set_input1(vec![0.3]);
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
            let results =
                task.result::<collision_detection_module::Types>(specific_run_id.0, out_datas);
            if let Some(results) = results {
                //fully type-safe results
                let result_0 = results.get_output0().unwrap();
                let result_1 = results.get_output1().unwrap();
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
    let result_option =
        task.result::<collision_detection_module::Types>(specific_run_id.0, out_datas);
    if let Some(result) = result_option {
        // fully type-safe results
        let result_0 = result.get_output0().unwrap();
        let result_1 = result.get_output1().unwrap();
        // your logic here
    }
}
