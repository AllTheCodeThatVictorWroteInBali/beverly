use bevy::prelude::*;
use std::collections::HashMap;

use super::{PointerId, UiPointerEvent};

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct HoverState {
    pub hovered: bool,
    pub contains_hovered_child: bool,
}

#[derive(Resource, Default)]
pub struct HoverTracker {
    by_pointer: HashMap<PointerId, Entity>,
}

impl HoverTracker {
    pub fn update(&mut self, pointer_id: PointerId, entity: Option<Entity>) -> Option<Entity> {
        match entity {
            Some(entity) => self.by_pointer.insert(pointer_id, entity),
            None => self.by_pointer.remove(&pointer_id),
        }
    }

    pub fn current(&self, pointer_id: PointerId) -> Option<Entity> {
        self.by_pointer.get(&pointer_id).copied()
    }
}

pub fn update_hover_states_from_events(
    mut events: MessageReader<UiPointerEvent>,
    mut hover_states: Query<&mut HoverState>,
) {
    for event in events.read() {
        match event.event_type {
            super::InteractionEventType::PointerEnter => {
                if let Ok(mut state) = hover_states.get_mut(event.context.current_target) {
                    state.hovered = true;
                }
            }
            super::InteractionEventType::PointerLeave => {
                if let Ok(mut state) = hover_states.get_mut(event.context.current_target) {
                    state.hovered = false;
                }
            }
            _ => {}
        }
    }
}
