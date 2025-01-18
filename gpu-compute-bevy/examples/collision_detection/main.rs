use std::collections::HashMap;

use bevy::{
    DefaultPlugins,
    app::{App, AppExit, Startup, Update},
    asset::{Assets, RenderAssetUsages},
    log,
    math::Vec2,
    prelude::{
        Camera2d, Commands, Component, EventReader, EventWriter, IntoSystemConfigs, Mesh, Mesh2d,
        OrthographicProjection, Query, Res, ResMut, Resource, Transform,
    },
    render::renderer::RenderDevice,
    sprite::MeshMaterial2d,
};
use bytemuck::{Pod, Zeroable};
use gpu_compute_bevy::{
    GpuAcceleratedBevyPlugin,
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
            iteration_space::IterationSpace, max_output_vector_lengths::MaxOutputLengths,
            task_specification::ComputeTaskSpecification,
        },
        wgsl_code::WgslCode,
    },
};
mod visuals;
use rust_to_wgsl::*;
use shared::{
    misc_types::{InputVectorTypesSpec, OutputVectorTypesSpec},
    wgsl_in_rust_helpers::*,
};
use visuals::{ColorHandles, spawn_camera, spawn_entities};

fn main() {
    let mut binding = App::new();
    let _app = binding
        .add_plugins(DefaultPlugins)
        .add_plugins(GpuAcceleratedBevyPlugin::default())
        .init_resource::<ColorHandles>()
        .init_resource::<State>()
        .add_event::<GpuComputeTaskSuccessEvent>()
        .add_systems(
            Startup,
            (spawn_camera, spawn_entities, create_task, modify_task).chain(),
        )
        .add_systems(
            Update,
            (
                run_task,
                handle_task_results,
                delete_task,
                exit_and_show_results,
            )
                .chain(),
        )
        .run();
}

const SPAWN_RANGE_MIN: i32 = -20;
const SPAWN_RANGE_MAX: i32 = 20;
const ENTITY_RADIUS: f32 = 5.;

#[derive(Resource)]
struct State {
    pub run_id: u128,
    pub num_entities: u32,
    pub length: u32,
    pub collisions: Vec<collision_detection_module::CollisionResult>,
}
impl Default for State {
    fn default() -> Self {
        State {
            run_id: 0,
            num_entities: 0,
            length: (SPAWN_RANGE_MAX - SPAWN_RANGE_MIN).abs() as u32,
            collisions: Vec::new(),
        }
    }
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

fn create_task(
    mut commands: Commands,
    mut gpu_acc_bevy: ResMut<GpuAcceleratedBevy>,
    gpu: Res<RenderDevice>,
) {
    let task_name = "collision_detection".to_string();
    let initial_iteration_space = IterationSpace::new(
        // set incorrectly here, just so that we can demonstrate changing it in "alter_task"
        100, 10, 1,
    );
    let mut initial_max_output_lengths = MaxOutputLengths::empty();
    initial_max_output_lengths.set("CollisionResult", 100);
    gpu_acc_bevy.create_task_from_rust_shader::<collision_detection_module::Types>(
        &task_name,
        &mut commands,
        &gpu,
        collision_detection_module::parsed(),
        initial_iteration_space,
        initial_max_output_lengths,
    );
}
fn delete_task(mut commands: Commands, mut gpu_acc_bevy: ResMut<GpuAcceleratedBevy>) {
    let task = gpu_acc_bevy.task(&"collision_detection".to_string());
    task.delete(&mut commands);
}
fn modify_task(
    mut commands: Commands,
    mut gpu_acc_bevy: ResMut<GpuAcceleratedBevy>,
    mut task_specifications: Query<&mut ComputeTaskSpecification>,
    state: Res<State>,
) {
    let task = gpu_acc_bevy.task(&"collision_detection".to_string());
    // specify the correct iter space and output maxes
    if let Ok(mut spec) = task_specifications.get_mut(task.entity) {
        let mut max_output_lengths = spec.output_array_lengths().clone();
        max_output_lengths.set("CollisionResult", (state.length * state.length) as usize);
        spec.mutate(
            &mut commands,
            task.entity,
            Some(IterationSpace::new(
                state.length as usize,
                state.length as usize,
                1,
            )),
            Some(max_output_lengths),
            None,
        );
    }
}
fn run_task(
    mut commands: Commands,
    mut gpu_compute: ResMut<GpuAcceleratedBevy>,
    mut task_run_ids: ResMut<GpuAcceleratedBevyRunIds>,
    mut state: ResMut<State>,
) {
    let task = gpu_compute.task(&"collision_detection".to_string());
    let mut input_data = InputData::<collision_detection_module::Types>::empty();
    input_data.set_input0(vec![collision_detection_module::Position {
        v: Vec2F32::new(0.3, 0.3),
    }]);
    input_data.set_input1(vec![0.3]);
    let run_id = task.run(&mut commands, input_data, task_run_ids);
    state.run_id = run_id;
}

fn handle_task_results(
    gpu_compute: ResMut<GpuAcceleratedBevy>,
    mut event_reader: EventReader<GpuComputeTaskSuccessEvent>,
    out_datas: Query<(&TaskRunId, &TypeErasedOutputData)>,
    mut state: ResMut<State>,
) {
    let task = gpu_compute.task(&"collision_detection".to_string());
    // reading events ensures that the results exist
    for ev in event_reader.read() {
        if ev.id == state.run_id {
            // here we get the actula result
            let results =
                task.result::<collision_detection_module::Types>(state.run_id, &out_datas);
            if let Some(results) = results {
                //fully type-safe results
                let collision_results: Vec<collision_detection_module::CollisionResult> = results
                    .get_output0()
                    .unwrap()
                    .into_iter()
                    .cloned()
                    .collect();
                // your logic here
                state.collisions = collision_results;
            }
        }
    }
}

fn exit_and_show_results(state: Res<State>, mut exit: EventWriter<AppExit>) {
    log::info!("collisions count: {}", state.collisions.len());
    exit.send(AppExit::Success);
}
