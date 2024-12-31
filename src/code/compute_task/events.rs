use bevy::prelude::{Entity, Event};

#[derive(Event)]
pub struct ComputeTaskNameChangeEvent {
    pub entity: Entity,
    pub new_name: String,
}

app.add_event::<ComputeTaskNameChangeEvent>();