use bevy::prelude::*;

#[derive(Resource, Clone, Copy, Debug)]
pub struct InteractionDebugSettings {
    pub log_events: bool,
    pub log_routing: bool,
    pub log_gesture_arena: bool,
}

impl Default for InteractionDebugSettings {
    fn default() -> Self {
        Self {
            log_events: false,
            log_routing: false,
            log_gesture_arena: false,
        }
    }
}

#[derive(Resource, Default, Clone, Debug)]
pub struct InteractionDebugSnapshot {
    pub pointer_position: Option<Vec2>,
    pub current_target: Option<Entity>,
    pub capture_target: Option<Entity>,
    pub hover_target: Option<Entity>,
    pub broad_phase_candidates: usize,
    pub narrow_phase_candidates: usize,
}
