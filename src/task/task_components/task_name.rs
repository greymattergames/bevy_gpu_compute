use bevy::prelude::Component;

#[derive(Component)]
pub struct TaskName(String);
impl Default for TaskName {
    fn default() -> Self {
        TaskName("unitialized task".to_string())
    }
}

impl TaskName {
    pub fn new(name: &str) -> Self {
        TaskName(name.to_string())
    }
    pub fn get(&self) -> &str {
        &self.0
    }
}
