use bevy::{
    DefaultPlugins,
    app::{App, AppExit, Startup, Update},
    log,
    prelude::{
        Commands, EventReader, EventWriter, IntoSystemConfigs, Query, Res, ResMut, Resource,
    },
    render::renderer::RenderDevice,
};
use gpu_compute_bevy::{
    GpuAcceleratedBevyPlugin, finished_gpu_tasks,
    resource::GpuAcceleratedBevy,
    run_ids::GpuAcceleratedBevyRunIds,
    starting_gpu_tasks,
    task::{
        events::GpuComputeTaskSuccessEvent,
        inputs::input_data::InputData,
        outputs::definitions::type_erased_output_data::TypeErasedOutputData,
        task_components::task_run_id::TaskRunId,
        task_specification::{
            iteration_space::IterationSpace, max_output_vector_lengths::MaxOutputLengths,
            task_specification::ComputeTaskSpecification,
        },
    },
};
mod visuals;
use bevy_gpu_compute_core::wgsl_in_rust_helpers::*;
use bevy_gpu_compute_macro::wgsl_shader_module;
use visuals::{BoundingCircleComponent, ColorHandles, spawn_camera, spawn_entities};

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
        .add_systems(Update, (run_task,).before(starting_gpu_tasks))
        .add_systems(
            Update,
            (handle_task_results, delete_task, exit_and_show_results)
                .after(finished_gpu_tasks)
                .chain(),
        )
        .run();
}

const SPAWN_RANGE_MIN: i32 = -2;
const SPAWN_RANGE_MAX: i32 = 2;
const ENTITY_RADIUS: f32 = 401.;

#[derive(Resource)]
struct State {
    pub run_id: u128,
    pub num_entities: u32,
    pub collisions: Vec<collision_detection_module::CollisionResult>,
}
impl Default for State {
    fn default() -> Self {
        State {
            run_id: 0,
            num_entities: 0,
            collisions: Vec::new(),
        }
    }
}

#[wgsl_shader_module]
mod collision_detection_module {
    use bevy_gpu_compute_core::wgsl_in_rust_helpers::*;
    use bevy_gpu_compute_macro::*;

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
    #[wgsl_output_array]
    struct MyDebugInfo {
        entity1: u32,
        entity2: u32,
        dist_squared: f32,
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
        // index = y * width + x
        let debug_index = other_entity * WgslVecInput::vec_len::<Radius>() + current_entity;
        WgslOutput::set::<MyDebugInfo>(debug_index, MyDebugInfo {
            entity1: current_entity,
            entity2: other_entity,
            dist_squared: dist_squared,
        });
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
        100, 100, 1,
    );
    let mut initial_max_output_lengths = MaxOutputLengths::empty();
    initial_max_output_lengths.set("CollisionResult", 100);
    initial_max_output_lengths.set("MyDebugInfo", 100);

    gpu_acc_bevy.create_task_from_rust_shader::<collision_detection_module::Types>(
        &task_name,
        &mut commands,
        &gpu,
        collision_detection_module::parsed(),
        initial_iteration_space,
        initial_max_output_lengths,
    );
}
fn delete_task(mut commands: Commands, gpu_acc_bevy: ResMut<GpuAcceleratedBevy>) {
    let task = gpu_acc_bevy.task(&"collision_detection".to_string());
    task.delete(&mut commands);
}
fn modify_task(
    mut commands: Commands,
    gpu_acc_bevy: ResMut<GpuAcceleratedBevy>,
    mut task_specifications: Query<&mut ComputeTaskSpecification>,
    state: Res<State>,
) {
    let task = gpu_acc_bevy.task(&"collision_detection".to_string());
    // specify the correct iter space and output maxes
    if let Ok(mut spec) = task_specifications.get_mut(task.entity) {
        let mut max_output_lengths = spec.output_array_lengths().clone();
        let num_entities = state.num_entities;
        max_output_lengths.set("CollisionResult", (num_entities * num_entities) as usize);
        max_output_lengths.set("MyDebugInfo", (num_entities * num_entities) as usize);
        spec.mutate(
            &mut commands,
            task.entity,
            Some(IterationSpace::new(
                state.num_entities as usize,
                state.num_entities as usize,
                1,
            )),
            Some(max_output_lengths),
            None,
        );
    }
}
fn run_task(
    mut commands: Commands,
    gpu_compute: ResMut<GpuAcceleratedBevy>,
    task_run_ids: ResMut<GpuAcceleratedBevyRunIds>,
    mut state: ResMut<State>,
    entities: Query<&BoundingCircleComponent>,
) {
    let task = gpu_compute.task(&"collision_detection".to_string());
    let mut input_data = InputData::<collision_detection_module::Types>::empty();
    input_data.set_input0(
        entities
            .iter()
            .map(|e| collision_detection_module::Position {
                v: Vec2F32::new(e.0.center.x, e.0.center.y),
            })
            .collect(),
    );
    input_data.set_input1(entities.iter().map(|e| e.0.radius()).collect());
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
            // log::info!("results: {:?}", results);
            if let Some(results) = results {
                // let debug_results: Vec<collision_detection_module::MyDebugInfo> = results
                //     .get_output1()
                //     .unwrap()
                //     .into_iter()
                //     .cloned()
                //     .collect();
                // log::info!("debug results: {:?}", debug_results);
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
