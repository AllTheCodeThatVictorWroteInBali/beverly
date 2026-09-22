use bevy::prelude::*;
use std::collections::HashMap;

use super::{PointerId, UiPointerEvent};

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct PressedState {
    pub pressed: bool,
}

#[derive(Resource, Default)]
pub struct PressTracker {
    by_pointer: HashMap<PointerId, Entity>,
}

impl PressTracker {
    pub fn press(&mut self, pointer_id: PointerId, entity: Entity) {
        self.by_pointer.insert(pointer_id, entity);
    }

    pub fn release(&mut self, pointer_id: PointerId) -> Option<Entity> {
        self.by_pointer.remove(&pointer_id)
    }

    pub fn pressed_target(&self, pointer_id: PointerId) -> Option<Entity> {
        self.by_pointer.get(&pointer_id).copied()
    }
}

pub fn update_pressed_states(
    mut events: MessageReader<UiPointerEvent>,
    mut pressed_states: Query<&mut PressedState>,
) {
    for event in events.read() {
        match event.event_type {
            super::InteractionEventType::PointerDown => {
                if let Ok(mut state) = pressed_states.get_mut(event.context.current_target) {
                    state.pressed = true;
                }
            }
            super::InteractionEventType::PointerUp | super::InteractionEventType::PointerCancel => {
                if let Ok(mut state) = pressed_states.get_mut(event.context.current_target) {
                    state.pressed = false;
                }
            }
            _ => {}
        }
    }
}
