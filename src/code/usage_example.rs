use std::collections::HashMap;

use bevy::{log::tracing_subscriber::filter::targets::Iter, prelude::{Commands, EventReader, Res}, reflect::Tuple, render::gpu_component_array_buffer};

use super::{compute_task::{component::GpuComputeTask, iteration_space_dependent_resources::iteration_space::IterationSpace, outputs::output_spec::OutputSpecs, wgsl_code::WgslCode}, manager_resource::{GpuCompute, GpuComputeBevyTaskType, GpuTaskFinishedEvent}};


pub struct ExampleTaskType {}
impl GpuComputeBevyTaskType for ExampleTaskType {
    type InType = (u32,f32,bool);
    type OutType = (bool, f64);
}


fn system(commands: &mut Commands,
    gpu_compute: Res<GpuCompute>,
    example_task_event_reader: EventReader<GpuTaskFinishedEvent<ExampleTaskType>>,
) {
  
    
    let task_name = gpu_compute.register_task::<ExampleTaskType>(
        commands,
         "example task".to_string(),// cannot change
        IterationSpace{
            x: 100,
            y: 10,
            z: 1,
        } ,// SHOULD be alterable
        WgslCode::from_file("./collision.wgsl","main".to_string()) ,// SHOULD be alterable
        inputs: ,// should NOT be alterable, just create a new task
        outputs: ,// should NOT be alterable, just create a new task
        max_num_outputs: , // SHOULD be alterabe
    );
    // example of deletion
    gpu_compute.delete_task::<ExampleTaskType>();
    // example of alteration
    gpu_compute.alter_task::<ExampleTaskType>().iteration_space(IterationSpace::new(4, 4, 4));
    // example of running the compute task
    let run_id = gpu_compute.run::<ExampleTaskType>("example task".to_string(), (1, 2.0, true));
    for results in  example_task_event_reader.read(){
        // todo handle the resuts
        if (results.id = run_id){
            // todo handle the results
            let resultType1 = results.get("output_name1");
        }
    }
}
