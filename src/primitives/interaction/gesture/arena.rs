use bevy::prelude::*;
use std::collections::HashMap;

use super::drag::GestureDragEvent;
use super::tap::GestureTapEvent;
use crate::primitives::interaction::{
    InteractionConfig,
    InteractionEventType,
    PointerCaptureRequest,
    PointerId,
    PointerReleaseRequest,
    PointerVelocityTracker,
    PressTracker,
    UiPointerEvent,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum RecognizerState {
    Possible,
    Accepted,
    Rejected,
    Cancelled,
    Completed,
}

#[derive(Clone, Copy, Debug)]
struct PointerGestureState {
    pointer_id: PointerId,
    down_target: Entity,
    down_position: Vec2,
    latest_position: Vec2,
    down_time_secs: f64,
    drag_state: RecognizerState,
    tap_state: RecognizerState,
    drag_started: bool,
}

#[derive(Resource, Default)]
pub struct GestureArenaState {
    by_pointer: HashMap<PointerId, PointerGestureState>,
    last_tap: Option<(Entity, f64, Vec2)>,
}

#[derive(Resource, Default, Clone, Debug)]
pub struct GestureArenaDebugFrame {
    pub lines: Vec<String>,
}

pub fn update_gesture_arena(
    config: Res<InteractionConfig>,
    mut arena: ResMut<GestureArenaState>,
    mut debug: ResMut<GestureArenaDebugFrame>,
    mut events: MessageReader<UiPointerEvent>,
    press_tracker: Res<PressTracker>,
    velocity: Res<PointerVelocityTracker>,
    mut capture_requests: MessageWriter<PointerCaptureRequest>,
    mut release_requests: MessageWriter<PointerReleaseRequest>,
    mut tap_writer: MessageWriter<GestureTapEvent>,
    mut drag_writer: MessageWriter<GestureDragEvent>,
) {
    debug.lines.clear();

    for event in events.read() {
        if event.context.phase != crate::primitives::interaction::InteractionEventPhase::Target {
            continue;
        }

        match event.event_type {
            InteractionEventType::PointerDown => {
                arena.by_pointer.insert(
                    event.pointer.pointer_id,
                    PointerGestureState {
                        pointer_id: event.pointer.pointer_id,
                        down_target: event.context.target,
                        down_position: event.pointer.window_position,
                        latest_position: event.pointer.window_position,
                        down_time_secs: event.pointer.timestamp_secs,
                        drag_state: RecognizerState::Possible,
                        tap_state: RecognizerState::Possible,
                        drag_started: false,
                    },
                );
            }
            InteractionEventType::PointerMove => {
                let Some(state) = arena.by_pointer.get_mut(&event.pointer.pointer_id) else {
                    continue;
                };

                let movement = event.pointer.window_position.distance(state.down_position);
                state.latest_position = event.pointer.window_position;

                if !state.drag_started && movement >= config.drag_start_distance {
                    state.drag_started = true;
                    state.drag_state = RecognizerState::Accepted;
                    state.tap_state = RecognizerState::Rejected;

                    capture_requests.write(PointerCaptureRequest {
                        pointer_id: state.pointer_id,
                        entity: state.down_target,
                    });

                    let v = velocity.velocity(state.pointer_id);
                    drag_writer.write(GestureDragEvent {
                        pointer_id: state.pointer_id,
                        pointer_type: event.pointer.pointer_type,
                        target: state.down_target,
                        start_position: state.down_position,
                        current_position: state.latest_position,
                        delta: event.pointer.delta,
                        total_delta: state.latest_position - state.down_position,
                        velocity: v.px_per_sec,
                        ended: false,
                        cancelled: false,
                    });
                } else if state.drag_started {
                    let v = velocity.velocity(state.pointer_id);
                    drag_writer.write(GestureDragEvent {
                        pointer_id: state.pointer_id,
                        pointer_type: event.pointer.pointer_type,
                        target: state.down_target,
                        start_position: state.down_position,
                        current_position: state.latest_position,
                        delta: event.pointer.delta,
                        total_delta: state.latest_position - state.down_position,
                        velocity: v.px_per_sec,
                        ended: false,
                        cancelled: false,
                    });
                }
            }
            InteractionEventType::PointerUp => {
                let Some(state) = arena.by_pointer.remove(&event.pointer.pointer_id) else {
                    continue;
                };

                let duration = (event.pointer.timestamp_secs - state.down_time_secs) as f32;
                let moved = state.latest_position.distance(state.down_position);

                if state.drag_started {
                    let v = velocity.velocity(state.pointer_id);
                    drag_writer.write(GestureDragEvent {
                        pointer_id: state.pointer_id,
                        pointer_type: event.pointer.pointer_type,
                        target: state.down_target,
                        start_position: state.down_position,
                        current_position: state.latest_position,
                        delta: event.pointer.delta,
                        total_delta: state.latest_position - state.down_position,
                        velocity: v.px_per_sec,
                        ended: true,
                        cancelled: false,
                    });
                    release_requests.write(PointerReleaseRequest {
                        pointer_id: state.pointer_id,
                        entity: Some(state.down_target),
                    });
                } else if duration <= config.click_max_duration_secs
                    && moved <= config.click_max_distance
                    && press_tracker.pressed_target(state.pointer_id).is_none()
                {
                    let count = if let Some((last_target, last_time, last_position)) = arena.last_tap {
                        if last_target == state.down_target
                            && (event.pointer.timestamp_secs - last_time)
                                <= config.double_click_duration_secs as f64
                            && state.latest_position.distance(last_position) <= config.click_max_distance
                        {
                            2
                        } else {
                            1
                        }
                    } else {
                        1
                    };

                    tap_writer.write(GestureTapEvent {
                        pointer_id: state.pointer_id,
                        target: state.down_target,
                        position: state.latest_position,
                        count,
                    });
                    arena.last_tap = Some((state.down_target, event.pointer.timestamp_secs, state.latest_position));
                }
            }
            InteractionEventType::PointerCancel => {
                if let Some(state) = arena.by_pointer.remove(&event.pointer.pointer_id) {
                    if state.drag_started {
                        let v = velocity.velocity(state.pointer_id);
                        drag_writer.write(GestureDragEvent {
                            pointer_id: state.pointer_id,
                            pointer_type: event.pointer.pointer_type,
                            target: state.down_target,
                            start_position: state.down_position,
                            current_position: state.latest_position,
                            delta: event.pointer.delta,
                            total_delta: state.latest_position - state.down_position,
                            velocity: v.px_per_sec,
                            ended: false,
                            cancelled: true,
                        });
                    }

                    release_requests.write(PointerReleaseRequest {
                        pointer_id: state.pointer_id,
                        entity: Some(state.down_target),
                    });
                }
            }
            _ => {}
        }

        debug.lines.push(format!(
            "pointer={:?} event={:?} target={:?}",
            event.pointer.pointer_id,
            event.event_type,
            event.context.target
        ));
    }
}
