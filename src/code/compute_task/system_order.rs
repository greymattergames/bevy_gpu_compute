//! let mut batched_collision_detection_schedule =
// Schedule::new(BatchedCollisionDetectionSchedule);
// batched_collision_detection_schedule.add_systems(
// (
//     initialize_batch,
//     update_wgsl_consts,
//     update_pipeline,
//     convert_collidables_to_wgsl_types,
//     create_buffers,
//     create_bind_group,
//     dispatch_to_gpu,
//     get_results_count_from_gpu,
//     read_results_from_gpu,
//     finish_batch,
// )
//     .chain(),
// );
// app.add_schedule(batched_collision_detection_schedule)
// .add_systems(Startup, setup_single_batch_resources);
// }
// }
