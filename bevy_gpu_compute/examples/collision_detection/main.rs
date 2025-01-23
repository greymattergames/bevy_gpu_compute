use bevy::{
    DefaultPlugins,
    app::{App, AppExit, Startup, Update},
    log,
    prelude::{
        Commands, EventReader, EventWriter, IntoSystemConfigs, Local, Query, Res, ResMut, Resource,
    },
    render::renderer::RenderDevice,
};
use bevy_gpu_compute::prelude::*;
mod visuals;
use visuals::{BoundingCircleComponent, ColorHandles, spawn_camera, spawn_entities};

fn main() {
    let mut binding = App::new();
    let _app = binding
        .add_plugins(DefaultPlugins)
        .add_plugins(BevyGpuComputePlugin::default())
        .init_resource::<ColorHandles>()
        .init_resource::<State>()
        .add_systems(
            Startup,
            (spawn_camera, spawn_entities, create_task, modify_task).chain(),
        )
        .add_systems(Update, (modify_task_config_inputs, run_task))
        .add_systems(Update, (handle_task_results, exit_and_show_results).chain())
        .run();
}

const SPAWN_RANGE_MIN: i32 = -1;
const SPAWN_RANGE_MAX: i32 = 1;
const ENTITY_RADIUS: f32 = 1.;
const EXIT_AFTER_FRAMES: u32 = 2;

#[derive(Resource)]
struct State {
    pub run_id: u128,
    pub num_entities: u32,
    pub collision_count: usize,
}
impl Default for State {
    fn default() -> Self {
        State {
            run_id: 0,
            num_entities: 0,
            collision_count: 0,
        }
    }
}

#[wgsl_shader_module]
mod collision_detection_module {
    use bevy_gpu_compute_core::wgsl_helpers::*;
    use bevy_gpu_compute_macro::*;

    const MY_CONST: u32 = 10;
    #[wgsl_config]
    struct Config {
        pub radius_multiplier: f32,
    }
    //todo prevent giving any module-level stuff a "pub" visibility
    #[wgsl_input_array]
    struct Position {
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
        counter_value: u32,
        is_collision: i32,
        dist_squared: f32,
        rad_sum_sq: f32,
        rad_mult: f32,
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
        let radius_sum = (current_radius + other_radius)
            * WgslConfigInput::get::<Config>().radius_multiplier
            * MY_CONST as f32;
        let rad_sum_sq = radius_sum * radius_sum;
        // index = y * width + x
        let debug_index = other_entity * WgslVecInput::vec_len::<Radius>() + current_entity;
        let is_collision = dist_squared < rad_sum_sq;
        WgslOutput::set::<MyDebugInfo>(debug_index, MyDebugInfo {
            entity1: current_entity,
            entity2: other_entity,
            counter_value: WgslOutput::len::<CollisionResult>(),
            is_collision: is_collision as i32,
            dist_squared: dist_squared,
            rad_sum_sq: rad_sum_sq,
            rad_mult: WgslConfigInput::get::<Config>().radius_multiplier,
        });
        if is_collision {
            WgslOutput::push::<CollisionResult>(CollisionResult {
                entity1: current_entity,
                entity2: other_entity,
            });
        }
    }
}

fn create_task(mut gpu_task_creator: BevyGpuComputeTaskCreator) {
    let initial_iteration_space = IterationSpace::new(
        // set incorrectly here, just so that we can demonstrate changing it later
        100, 100, 1,
    );
    //* There are two methods of creating the MaxOutputLengths config object: */
    // Method 1:
    let max_output_lengths: MaxOutputLengths =
        collision_detection_module::MaxOutputLengthsBuilder::new()
            .set_collision_result(100)
            .set_my_debug_info(100)
            .into();
    // Method 2:
    let mut alternate_max_output_lengths = MaxOutputLengths::empty();
    initial_max_output_lengths.set("CollisionResult", 100);
    initial_max_output_lengths.set("MyDebugInfo", 100);
    //
    gpu_task_creator.create_task_from_rust_shader::<collision_detection_module::Types>(
        "collision_detection", //todo, ensure name is unique
        collision_detection_module::parsed(),
        initial_iteration_space,
        max_output_lengths,
    );
}
fn delete_task(mut gpu_task_deleter: BevyGpuComputeTaskDeleter) {
    let task = gpu_task_deleter.delete("collision_detection");
}
fn modify_task(mut gpu_tasks: GpuTaskRunner, state: Res<State>) {
    let max_output_lengths: MaxOutputLengths =
        collision_detection_module::MaxOutputLengthsBuilder::new()
            .set_collision_result((num_entities * num_entities) as usize)
            .set_my_debug_info((num_entities * num_entities) as usize)
            .into();
    let iteration_space =
        IterationSpace::new(state.num_entities as usize, state.num_entities as usize, 1);
    let pending_commands = gpu_tasks
        .task("collision_detection")
        .mutate(Some(iteration_space), Some(max_output_lengths));
    gpu_tasks.run_commands(pending_commands);
}
fn modify_task_config_inputs(mut count: Local<u32>, mut gpu_tasks: GpuTaskRunner) {
    let radius_multiplier =
        (EXIT_AFTER_FRAMES as i32 - *count as i32) as f32 / EXIT_AFTER_FRAMES as f32;
    log::info!("rad_mult: {}", radius_multiplier);
    // below needs to simplify
    let mut config = ConfigInputData::<collision_detection_module::Types>::empty();
    config.set_input0(collision_detection_module::Config { radius_multiplier });

    //todo better api: let configs = collision_detection_module::ConfigsBuilder::new().set_config( collision_detection_module::Config {radius_multiplier} ).into();
    let commands = gpu_tasks
        .task("collision_detection")
        .set_config_inputs(config);
    gpu_tasks.run_commands(commands);

    *count += 1;
}

fn run_task(
    mut gpu_tasks: GpuTaskRunner,
    mut state: ResMut<State>,
    entities: Query<&BoundingCircleComponent>,
) {
    let task = gpu_tasks.task("collision_detection");
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
    //todo better api:
    let input_data = collision_detection_module::InputDataBuilder::new()
        .set_position(
            entities
                .iter()
                .map(|e| collision_detection_module::Position {
                    v: Vec2F32::new(e.0.center.x, e.0.center.y),
                }),
        )
        .set_radius(entities.iter().map(|e| e.0.radius()))
        .into();
    // working api below
    task.set_inputs(input_data);
    gpu_tasks.run_commands(task);
}

fn handle_task_results(mut gpu_task_results: GpuTaskReader, mut state: ResMut<State>) {
    let results =
        gpu_task_results.latest_results::<collision_detection_module::Types>("collision_detection");
    // log::info!("results: {:?}", results);
    if let Ok(results) = results {
        let debug_results: Vec<collision_detection_module::MyDebugInfo> = results
            .get_output1()
            .unwrap()
            .into_iter()
            .cloned()
            .collect();
        log::info!("debug results: {:?}", debug_results);
        let t = results.get_output1().unwrap();
        //fully type-safe results
        let collision_results: Vec<collision_detection_module::CollisionResult> = results
            .get_output0()
            .unwrap()
            .into_iter()
            .cloned()
            .collect();
        // your logic here
        let count = collision_results.len();
        log::info!("collisions this frame: {}", count);
        log::info!("collision_results: {:?}", collision_results);
        state.collision_count += count;
    }
}

/**
 * WHAT BEVY ACCESS DO WE ACTUALLY REQUIRE?
 * render::render_resource::Buffer, but its just a type we can use anywhere - TYPE
 * RENDER DEVICE (render::renderer::RenderDevice,) - RESOURCE
 * render_resource::BindGroup, another static type - TYPE
 * RenderQueue, (render::render_graph::RenderQueue) - RESOURCE
 * render::render_resource::BindGroupLayout - TYPE
 */

/// The [`SystemParam`] struct can contain any types that can also be included in a
/// system function signature.
///
/// In this example, it includes a query and a mutable resource.

// when the local variable "count" goes above a certain number (representing frame count), exit the app
fn exit_and_show_results(mut count: Local<u32>, state: Res<State>, mut exit: EventWriter<AppExit>) {
    if *count > EXIT_AFTER_FRAMES {
        log::info!("collisions count: {}", state.collision_count);
        exit.send(AppExit::Success);
    }
    *count += 1;
}
