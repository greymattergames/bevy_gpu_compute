pub struct GpuAccBevyComputeTaskInputSpec {
    // each has a string label
    // each has an optional bind number for correct association in the wgsl, otherwise they are numbered in order, with inputs coming before outputs
    pub input_buffers: Vec<(String, Option<u32>)>,
}
