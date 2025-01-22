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
        .add_event::<GpuComputeTaskSuccessEvent>()
        .add_systems(
            Startup,
            (spawn_camera, spawn_entities, create_task, modify_task).chain(),
        )
        .add_systems(
            Update,
            (modify_task_config_inputs, run_task).before(starting_gpu_tasks),
        )
        .add_systems(
            Update,
            (handle_task_results, exit_and_show_results)
                .after(finished_gpu_tasks)
                .chain(),
        )
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

fn create_task(bevy_gpu_compute: BevyGpuCompute) {
    let task_name = "collision_detection";
    let initial_iteration_space = IterationSpace::new(
        // set incorrectly here, just so that we can demonstrate changing it later
        100, 100, 1,
    );
    let mut initial_max_output_lengths = MaxOutputLengths::empty();
    // todo, change these to be type safe, instead of passing in strings
    // todo, change to be like this: `collision_detection_module::MaxOutputLengths::new().set_collision_result(100).set_my_debug_info(100)`
    initial_max_output_lengths.set("CollisionResult", 100);
    initial_max_output_lengths.set("MyDebugInfo", 100);

    //todo, documentation about how to pass in the results of the proc macro here
    bevy_gpu_compute.create_task_from_rust_shader::<collision_detection_module::Types>(
        task_name,     //todo, ensure name is unique
        &mut commands, //todo  remove commands
        &gpu,          //todo remove need to pass in this
        collision_detection_module::parsed(),
        initial_iteration_space,
        initial_max_output_lengths,
    );
}
fn delete_task(bevy_gpu_compute: BevyGpuCompute) {
    bevy_gpu_compute.delete_task("collision_detection");
}
fn modify_task(bevy_gpu_compute: BevyGpuCompute, state: Res<State>) {
    //todo see above for better api
    let mut max_output_lengths = MaxOutputLengths::empty();
    let num_entities = state.num_entities;
    max_output_lengths.set("CollisionResult", (num_entities * num_entities) as usize);
    max_output_lengths.set("MyDebugInfo", (num_entities * num_entities) as usize);
    let iteration_space =
        IterationSpace::new(state.num_entities as usize, state.num_entities as usize, 1);
    bevy_gpu_compute.mutate_task(
        "collision_detection",
        &mut commands,            //todo remove need to pass in commands
        &mut task_specifications, // todo, remove need to pass in this and query it
        Some(iteration_space),
        Some(max_output_lengths),
        None, //todo remove this, as it should be set automatically at run
    );
}
fn modify_task_config_inputs(mut count: Local<u32>, bevy_gpu_compute: BevyGpuCompute) {
    let radius_multiplier =
        (EXIT_AFTER_FRAMES as i32 - *count as i32) as f32 / EXIT_AFTER_FRAMES as f32;
    log::info!("rad_mult: {}", radius_multiplier);
    // below needs to simplify
    let mut config = ConfigInputData::<collision_detection_module::Types>::empty();
    config.set_input0(collision_detection_module::Config { radius_multiplier });
    task.set_config_inputs(&mut commands, config);
    //todo, better api below:
    bevy_gpu_compute
        .task("collision_detection")
        .set_config_input("Config", collision_detection_module::Config {
            radius_multiplier,
        });
    *count += 1;
}

fn run_task(
    bevy_gpu_compute: BevyGpuCompute,
    mut state: ResMut<State>,
    entities: Query<&BoundingCircleComponent>,
) {
    let task = bevy_gpu_compute.task("collision_detection");
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
    task.set_input(
        "Position",
        entities
            .iter()
            .map(|e| collision_detection_module::Position {
                v: Vec2F32::new(e.0.center.x, e.0.center.y),
            }),
    );
    task.set_input("Radius", entities.iter().map(|e| e.0.radius()));
    //todo, ensure the task DOESNT run if we don't call this each frame
    let run_id = task.run(
        //todo remove commands
        &mut commands,
        //todo remove input data as it was set previously
        input_data,
        //todo, remove necesity to pass this in
        task_run_ids,
    );
    state.run_id = run_id;
}

fn handle_task_results(
    gpu_compute: ResMut<BevyGpuCompute>,
    mut event_reader: EventReader<GpuComputeTaskSuccessEvent>,
    out_datas: Query<(&TaskRunId, &TypeErasedOutputData)>,
    mut state: ResMut<State>,
) {
    let task = gpu_compute.task("collision_detection");
    //todo, or to remove from bevy entirely...
    //todo: task.result::<collision_detection_module::Types>(state.run_id) = Option<Outputdata<Types>>;
    // reading events ensures that the results exist
    for ev in event_reader.read() {
        if ev.id == state.run_id {
            log::info!("handling results for run id: {}", state.run_id);
            // here we get the actual result
            //todo, store the result data in the event itself to remove this secondary call
            let results = task.result::<collision_detection_module::Types>(
                state.run_id,
                &out_datas, //todo, remove the need to pass this in
            );
            // log::info!("results: {:?}", results);
            if let Some(results) = results {
                let debug_results: Vec<collision_detection_module::MyDebugInfo> = results
                    .get_output1()
                    .unwrap()
                    .into_iter()
                    .cloned()
                    .collect();
                log::info!("debug results: {:?}", debug_results);
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
