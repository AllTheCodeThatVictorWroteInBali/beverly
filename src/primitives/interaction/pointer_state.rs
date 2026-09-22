use bevy::prelude::*;
use std::collections::HashMap;

use super::{PointerButtons, PointerEvent, PointerId, PointerType};

#[derive(Clone, Copy, Debug, Default)]
pub struct PointerTrackingState {
    pub pointer_type: PointerType,
    pub latest_position: Vec2,
    pub latest_delta: Vec2,
    pub buttons: PointerButtons,
    pub pressure: f32,
    pub timestamp_secs: f64,
}

#[derive(Resource, Default)]
pub struct PointerFrameState {
    by_pointer: HashMap<PointerId, PointerTrackingState>,
}

impl PointerFrameState {
    pub fn update(&mut self, event: &PointerEvent) {
        let state = self.by_pointer.entry(event.pointer_id).or_default();
        state.pointer_type = event.pointer_type;
        state.latest_position = event.window_position;
        state.latest_delta = event.delta;
        state.buttons = event.buttons;
        state.pressure = event.pressure;
        state.timestamp_secs = event.timestamp_secs;
    }

    pub fn remove(&mut self, pointer_id: PointerId) {
        self.by_pointer.remove(&pointer_id);
    }

    pub fn get(&self, pointer_id: PointerId) -> Option<&PointerTrackingState> {
        self.by_pointer.get(&pointer_id)
    }

    pub fn active_pointer_ids(&self) -> impl Iterator<Item = PointerId> + '_ {
        self.by_pointer.keys().copied()
    }
}
