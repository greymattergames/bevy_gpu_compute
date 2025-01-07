use bevy::prelude::Component;

#[derive(Component)]
pub struct TaskRunId(pub u128);
impl Default for TaskRunId {
    fn default() -> Self {
        TaskRunId(0)
    }
}
