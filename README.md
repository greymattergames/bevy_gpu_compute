# Bevy GPU Compute

Supercharge your computationally-expensive Bevy applications with GPU acceleration - no graphics programming knowledge required!

This library enables you to easily offload computationally intensive tasks to the GPU using pure Rust code. No need to learn WGSL, WGPU or other graphics concepts.

## Key Features

- ⚡ Write GPU compute shaders in pure Rust
- 🔄 Automatic resource management - no manual GPU buffer handling  
- 🎮 Seamless integration with Bevy ECS
- 🛠️ Simple declarative API using attributes

## Fast!
#### 50% better performance compared to CPU, in this real-world performance comparison: [gpu_collision_detection_bevy](https://github.com/Sheldonfrith/gpu_collision_detection_bevy).


![image](https://github.com/user-attachments/assets/8e1b7eb4-f705-4e37-ac22-59214bad2ac1)


## Quick Start

1. Add to your project:

`cargo add bevy_gpu_compute`
AND
`cargo add bevy_gpu_compute_core`

3. Define your compute shader in Rust:
```rust
#[wgsl_shader_module]
mod collision_detection_module {
    use bevy_gpu_compute_core::wgsl_helpers::*;
    use bevy_gpu_compute_macro::*;

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
        let radius_sum = (current_radius + other_radius);
        let rad_sum_sq = radius_sum * radius_sum;
        let is_collision = dist_squared < rad_sum_sq;
        if is_collision {
            WgslOutput::push::<CollisionResult>(CollisionResult {
                entity1: current_entity,
                entity2: other_entity,
            });
        }
    }
}
```
3. Use it in your Bevy systems:
```rust
fn create_task(mut gpu_task_creator: BevyGpuComputeTaskCreator) {
    let initial_iteration_space = IterationSpace::new(100, 100, 1);
    let initial_max_output_lengths = collision_detection_module::MaxOutputLengthsBuilder::new()
        .set_collision_result(100)
        .finish();
    gpu_task_creator.create_task_from_rust_shader::<collision_detection_module::Types>(
        "collision_detection", // ensure name is unique
        collision_detection_module::parsed(),
        initial_iteration_space,
        initial_max_output_lengths,
    );
}

fn run_task(mut gpu_tasks: GpuTaskRunner, entities: Query<&BoundingCircleComponent>) {
    let input_data = collision_detection_module::InputDataBuilder::new()
        .set_position(// your input data here
        )
        .set_radius(// your input data here
        )
        .into();
    let task = gpu_tasks
        .task("collision_detection")
        .set_inputs(input_data)
        .run();
    gpu_tasks.run_commands(task);
}

fn handle_task_results(mut gpu_task_reader: GpuTaskReader, mut state: ResMut<State>) {
    let results = gpu_task_reader
        .latest_results::<collision_detection_module::OutputDataBuilder>("collision_detection");
    if let Ok(results) = results {
        //fully type-safe results
        let collision_results = results.collision_result.unwrap();
        // your logic here
    }
}

```
(See `bevy_gpu_compute/examples` for fully functioning example bevy apps.)

# Core Concepts
## Iteration Space & Position
The GPU processes work in parallel across an N-dimensional grid (1D, 2D, or 3D). Think of it like this:
```rust
// CPU (Sequential):
for x in 0..width {
    for y in 0..height {
        process(x, y);
    }
}

// GPU (Parallel):
fn main(pos: WgslIterationPosition) {
    let x = pos.x; // Current X position
    let y = pos.y; // Current Y position
    process(x, y); // Runs in parallel!
}
```
The IterationSpace defines the total size of this grid. For example:

`IterationSpace::new(1000, 1, 1)` - Process 1000 items in 1D

`IterationSpace::new(100, 100, 1)` - Process 10,000 items in 2D (useful for pairwise comparisons)

`IterationSpace::new(10, 10, 10)` - Process 1000 items in 3D (useful for spatial algorithms)

## Input Types
### Config Inputs (Uniforms)
Constants that apply to all parallel computations:
```rust
#[wgsl_config]
struct Settings {
    threshold: f32,
    multiplier: f32,
}
```
### Array Inputs
Collections of data to process in parallel:
```rust
#[wgsl_input_array]
struct Particle {
    position: Vec3F32,
    velocity: Vec3F32,
}
```

Becomes something like `Vec<Particle>` on the GPU.

## Output Types
### Fixed Arrays
When you know the exact output size:
```rust
#[wgsl_output_array]
struct GridCell {
    density: f32,
}
```

Becomes something like `[GridCell;N]` on the GPU.

### Dynamic Vectors
For variable-length results (like collision detection):
```rust
#[wgsl_output_vec]
struct Collision {
    entity1: u32,
    entity2: u32,
}
```

Becomes something like `Vec<Collision>` on the GPU.


## Architecture
The library consists of three crates:

`bevy_gpu_compute`: Main user-facing crate with Bevy integration

`bevy_gpu_compute_macro`: Converts Rust code to WGSL shaders, using proc-macro magic

`bevy_gpu_compute_core`: Shared types and utilities

## Performance Tips
- Prefer `wgsl_output_array` over `wgsl_output_vec` when you have an accurate idea of how many results you will be receiving
- Use built-in vector/matrix types, like `Vec3F32`, where possible

## Limitations

- Some Rust features like traits and generics are not supported in compute shaders
- Maximum output sizes must be specified upfront
- Limited to compute shaders (no graphics)
- Requires NIGHTLY Rust (probably, I haven't tested it on `stable`)
- Requires Bevy 15

## Contributing
Contributions are welcome! There's still a lot to do. Submit a pull request, and I will most likely approve it.
### Areas that need work:
- More examples
- Better error messages
- Documentation improvements
- `bevy_gpu_compute_macro` crate needs code cleanup
- `bevy_gpu_compute_macro` support more wgsl features like pointers
- Support batching strategies to work around max storage buffer size limitations
