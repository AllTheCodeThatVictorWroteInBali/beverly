// src/ui/textarea/mod.rs

mod component;
mod systems;

pub use component::Textarea;
pub use component::TextareaConfig;
pub use component::spawn_textarea;
pub use systems::textarea_focus_gained_system;
pub use systems::textarea_focus_lost_system;
pub use systems::textarea_focus_system;
pub use systems::textarea_keyboard_system;
pub use systems::textarea_visual_system;

use bevy::prelude::*;

#[derive(Message, Clone, Debug)]
pub struct TextareaChanged {
    pub entity: Entity,
    pub value: String,
}

impl TextareaChanged {
    pub fn new(entity: Entity, textarea: &Textarea) -> Self {
        Self {
            entity,
            value: textarea.value.clone(),
        }
    }
}

pub struct TextareaPlugin;

impl Plugin for TextareaPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TextareaChanged>()
            .add_observer(textarea_focus_gained_system)
            .add_observer(textarea_focus_lost_system)
            .add_systems(
                Update,
                (
                    textarea_focus_system,
                    textarea_keyboard_system,
                    textarea_visual_system,
                )
                    .chain(),
            );
    }
}
