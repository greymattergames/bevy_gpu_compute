use bevy::render::render_resource::Buffer;

#[derive(Default)]

pub struct TaskBuffers {
    pub input: Vec<Buffer>,
    pub config: Vec<Buffer>,
    pub output: OutputBuffers,
}

#[derive(Default)]
pub struct OutputBuffers {
    pub main: Vec<Buffer>,
    pub staging: Vec<Buffer>,
    pub count: Vec<Buffer>,
    pub count_staging: Vec<Buffer>,
}
