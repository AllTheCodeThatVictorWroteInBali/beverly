use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::rendering::Surface;

use super::action::{ActionBinding, DisabledInteraction, derive_activation_actions};
use super::capture::{
    PointerCaptureMap,
    PointerCaptureRequest,
    PointerReleaseRequest,
    apply_capture_requests,
    cleanup_capture_for_removed_entities,
};
use super::debug::{InteractionDebugSettings, InteractionDebugSnapshot};
use super::event::{
    InteractionEventContext,
    InteractionEventPhase,
    InteractionEventType,
    PointerButtonState,
    PointerButtons,
    PointerEvent,
    PointerEventModifiers,
    PointerEventRequest,
    PointerId,
    PointerType,
    UiPointerEvent,
};
use super::gesture::{
    GestureArenaDebugFrame,
    GestureArenaState,
    GestureDragEvent,
    GestureTapEvent,
    update_gesture_arena,
};
use super::hit_shape::HitShape;
use super::hit_test::{
    HitBehavior,
    HitTestNode,
    UiHitNode,
    UiHitTargetCache,
    UiHitTestDebugFrame,
    hit_test_primary,
};
use super::hover::{HoverState, HoverTracker, update_hover_states_from_events};
use super::pointer_state::PointerFrameState;
use super::press::{PressTracker, PressedState, update_pressed_states};
use super::velocity::PointerVelocityTracker;
use bevy::input_focus::{FocusCause, InputFocus};

#[derive(Resource, Clone, Copy, Debug)]
pub struct InteractionConfig {
    pub click_max_duration_secs: f32,
    pub click_max_distance: f32,
    pub double_click_duration_secs: f32,
    pub drag_start_distance: f32,
    pub long_press_secs: f32,
}

impl Default for InteractionConfig {
    fn default() -> Self {
        Self {
            // Slightly higher tap window to better tolerate slower clicks.
            click_max_duration_secs: 0.42,
            click_max_distance: 8.0,
            double_click_duration_secs: 0.28,
            drag_start_distance: 8.0,
            long_press_secs: 0.45,
        }
    }
}

#[derive(Resource, Default)]
struct LastCursorPosition {
    window_position: Option<Vec2>,
}

pub struct InteractionPlugin;

/// Shared PreUpdate stages. Legacy widget consumers still run in Update.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum UiActionSystems {
    Pointer,
    Keyboard,
    Apply,
}

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InteractionConfig>()
            .init_resource::<PointerFrameState>()
            .init_resource::<PointerCaptureMap>()
            .init_resource::<HoverTracker>()
            .init_resource::<PressTracker>()
            .init_resource::<PointerVelocityTracker>()
            .init_resource::<UiHitTargetCache>()
            .init_resource::<UiHitTestDebugFrame>()
            .init_resource::<InteractionDebugSettings>()
            .init_resource::<InteractionDebugSnapshot>()
            .init_resource::<GestureArenaState>()
            .init_resource::<GestureArenaDebugFrame>()
            .init_resource::<LastCursorPosition>()
            .add_message::<PointerEventRequest>()
            .add_message::<UiPointerEvent>()
            .add_message::<PointerCaptureRequest>()
            .add_message::<PointerReleaseRequest>()
            .add_message::<super::action::InteractionActionEvent>()
            .add_message::<GestureTapEvent>()
            .add_message::<GestureDragEvent>()
            .add_systems(
                PreUpdate,
                (
                    bootstrap_interactive_nodes,
                    normalize_mouse_input,
                    emit_pointer_cancel_on_window_focus_loss,
                    apply_capture_requests,
                    route_pointer_events,
                    update_hover_states_from_events,
                    update_pressed_states,
                    update_gesture_arena,
                    derive_activation_actions,
                    sync_button_interaction_from_states,
                    cleanup_capture_for_removed_entities,
                )
                    .chain()
                    .in_set(UiActionSystems::Pointer)
                    .after(bevy::input::InputSystems)
                    .after(bevy::ui::UiSystems::Focus)
                    .before(bevy::input_focus::InputFocusSystems::Dispatch),
            );
    }
}

fn emit_pointer_cancel_on_window_focus_loss(
    windows: Query<&Window, With<PrimaryWindow>>,
    pointer_state: Res<PointerFrameState>,
    mut writer: MessageWriter<PointerEventRequest>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    if !window.focused {
        for pointer_id in pointer_state.active_pointer_ids() {
            if let Some(state) = pointer_state.get(pointer_id) {
                writer.write(PointerEventRequest {
                    event_type: InteractionEventType::PointerCancel,
                    pointer: PointerEvent {
                        pointer_id,
                        pointer_type: state.pointer_type,
                        window_position: state.latest_position,
                        screen_position: state.latest_position,
                        delta: Vec2::ZERO,
                        buttons: state.buttons,
                        button_state: PointerButtonState::Released,
                        pressure: state.pressure,
                        tilt: Vec2::ZERO,
                        timestamp_secs: state.timestamp_secs,
                        modifiers: PointerEventModifiers::default(),
                    },
                });
            }
        }
    }
}

fn normalize_mouse_input(
    mut writer: MessageWriter<PointerEventRequest>,
    time: Res<Time>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut last: ResMut<LastCursorPosition>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    let Some(window_position) = window.cursor_position() else {
        return;
    };

    let previous = last.window_position.unwrap_or(window_position);
    let delta = window_position - previous;
    last.window_position = Some(window_position);

    let modifiers = PointerEventModifiers {
        shift: keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight),
        ctrl: keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight),
        alt: keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight),
        command: keyboard.pressed(KeyCode::SuperLeft) || keyboard.pressed(KeyCode::SuperRight),
    };

    let buttons = PointerButtons {
        primary: mouse_buttons.pressed(MouseButton::Left),
        secondary: mouse_buttons.pressed(MouseButton::Right),
        middle: mouse_buttons.pressed(MouseButton::Middle),
    };

    let base = PointerEvent {
        pointer_id: PointerId(0),
        pointer_type: PointerType::Mouse,
        window_position,
        screen_position: window_position,
        delta,
        buttons,
        button_state: PointerButtonState::None,
        pressure: if buttons.primary { 1.0 } else { 0.5 },
        tilt: Vec2::ZERO,
        timestamp_secs: time.elapsed_secs_f64(),
        modifiers,
    };

    writer.write(PointerEventRequest {
        event_type: InteractionEventType::PointerMove,
        pointer: base,
    });

    if mouse_buttons.just_pressed(MouseButton::Left) {
        let mut down = base;
        down.button_state = PointerButtonState::Pressed;
        writer.write(PointerEventRequest {
            event_type: InteractionEventType::PointerDown,
            pointer: down,
        });
    }

    if mouse_buttons.just_released(MouseButton::Left) {
        let mut up = base;
        up.button_state = PointerButtonState::Released;
        writer.write(PointerEventRequest {
            event_type: InteractionEventType::PointerUp,
            pointer: up,
        });
    }
}

#[derive(Clone, Copy, Debug)]
struct RoutedEvent {
    event_type: InteractionEventType,
    pointer: PointerEvent,
    target: Entity,
    local_position: Vec2,
}

fn route_pointer_events(
    mut events: MessageReader<PointerEventRequest>,
    mut writer: MessageWriter<UiPointerEvent>,
    mut pointer_state: ResMut<PointerFrameState>,
    mut hover_tracker: ResMut<HoverTracker>,
    mut press_tracker: ResMut<PressTracker>,
    mut velocity_tracker: ResMut<PointerVelocityTracker>,
    mut focus: ResMut<InputFocus>,
    capture_map: Res<PointerCaptureMap>,
    mut release_writer: MessageWriter<PointerReleaseRequest>,
    mut hit_cache: ResMut<UiHitTargetCache>,
    mut hit_debug: ResMut<UiHitTestDebugFrame>,
    parent_query: Query<&ChildOf>,
    query: Query<HitTestNode>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    for request in events.read() {
        let pointer = request.pointer;
        pointer_state.update(&pointer);
        velocity_tracker.push_sample(pointer.pointer_id, pointer.timestamp_secs, pointer.window_position);

        let captured = capture_map.captured_entity(pointer.pointer_id);
        let hit = windows.single().ok().and_then(|window| {
            hit_test_primary(pointer.window_position, window.scale_factor(), &query, &parent_query)
        });

        hit_debug.pointer = pointer.window_position;
        hit_debug.final_target = hit.map(|value| value.entity);
        hit_debug.capture_target = captured;

        let target = captured.or_else(|| hit.map(|value| value.entity));
        let local_position = hit.map(|value| value.local_position).unwrap_or(Vec2::ZERO);

        if request.event_type == InteractionEventType::PointerMove {
            let previous = hover_tracker.current(pointer.pointer_id);
            let next = hit.map(|value| value.entity);
            if previous != next {
                if let Some(previous_entity) = previous {
                    emit_one(
                        &mut writer,
                        InteractionEventType::PointerLeave,
                        pointer,
                        previous_entity,
                        previous_entity,
                        Vec2::ZERO,
                        InteractionEventPhase::Target,
                    );
                }
                if let Some(next_entity) = next {
                    emit_one(
                        &mut writer,
                        InteractionEventType::PointerEnter,
                        pointer,
                        next_entity,
                        next_entity,
                        local_position,
                        InteractionEventPhase::Target,
                    );
                }
                hover_tracker.update(pointer.pointer_id, next);
            }
            hit_cache.last_primary_target = next;
        }

        let Some(target) = target else {
            if request.event_type == InteractionEventType::PointerCancel {
                pointer_state.remove(pointer.pointer_id);
                press_tracker.release(pointer.pointer_id);
                velocity_tracker.clear_pointer(pointer.pointer_id);
            }
            continue;
        };

        let routed = RoutedEvent {
            event_type: request.event_type,
            pointer,
            target,
            local_position,
        };

        route_with_phases(routed, &parent_query, &mut writer);

        match request.event_type {
            InteractionEventType::PointerDown => {
                press_tracker.press(pointer.pointer_id, target);
                focus.set(target, FocusCause::Navigated);
            }
            InteractionEventType::PointerUp | InteractionEventType::PointerCancel => {
                press_tracker.release(pointer.pointer_id);
                release_writer.write(PointerReleaseRequest {
                    pointer_id: pointer.pointer_id,
                    entity: None,
                });
                if request.event_type == InteractionEventType::PointerCancel {
                    pointer_state.remove(pointer.pointer_id);
                    velocity_tracker.clear_pointer(pointer.pointer_id);
                }
            }
            _ => {}
        }
    }
}

fn route_with_phases(
    routed: RoutedEvent,
    parent_query: &Query<&ChildOf>,
    writer: &mut MessageWriter<UiPointerEvent>,
) {
    let ancestry = ancestry_path(routed.target, parent_query);

    for entity in ancestry.iter().copied() {
        emit_one(
            writer,
            routed.event_type,
            routed.pointer,
            routed.target,
            entity,
            routed.local_position,
            InteractionEventPhase::Capture,
        );
    }

    emit_one(
        writer,
        routed.event_type,
        routed.pointer,
        routed.target,
        routed.target,
        routed.local_position,
        InteractionEventPhase::Target,
    );

    for entity in ancestry.into_iter().rev() {
        emit_one(
            writer,
            routed.event_type,
            routed.pointer,
            routed.target,
            entity,
            routed.local_position,
            InteractionEventPhase::Bubble,
        );
    }

    if routed.event_type == InteractionEventType::PointerMove {
        emit_one(
            writer,
            InteractionEventType::PointerOver,
            routed.pointer,
            routed.target,
            routed.target,
            routed.local_position,
            InteractionEventPhase::Target,
        );
    }
}

fn ancestry_path(target: Entity, parent_query: &Query<&ChildOf>) -> Vec<Entity> {
    let mut path = Vec::new();
    let mut cursor = Some(target);

    while let Some(entity) = cursor {
        cursor = parent_query.get(entity).ok().map(ChildOf::parent);
        if let Some(parent) = cursor {
            path.push(parent);
        }
    }

    path.reverse();
    path
}

fn emit_one(
    writer: &mut MessageWriter<UiPointerEvent>,
    event_type: InteractionEventType,
    pointer: PointerEvent,
    target: Entity,
    current_target: Entity,
    local_position: Vec2,
    phase: InteractionEventPhase,
) {
    writer.write(UiPointerEvent {
        event_type,
        pointer,
        context: InteractionEventContext {
            target,
            current_target,
            phase,
            local_position,
        },
        propagation_stopped: false,
        default_prevented: false,
    });
}

fn bootstrap_interactive_nodes(
    mut commands: Commands,
    query: Query<(Entity, Option<&Surface>, Option<&HitShape>), Added<Button>>,
) {
    for (entity, surface, has_hit_shape) in &query {
        let mut entity_commands = commands.entity(entity);
        entity_commands.insert((
            UiHitNode,
            HoverState::default(),
            PressedState::default(),
            HitBehavior::Auto,
            ActionBinding {
                action: super::action::InteractionAction::Activate,
            },
        ));

        if has_hit_shape.is_none() {
            if let Some(surface) = surface {
                entity_commands.insert(HitShape::from_render_shape(surface.shape));
            } else {
                entity_commands.insert(HitShape::LayoutRect);
            }
        }
    }
}

fn sync_button_interaction_from_states(
    mut buttons: Query<(
        Entity,
        &mut Interaction,
        Option<&HoverState>,
        Option<&PressedState>,
        Option<&DisabledInteraction>,
    ), With<Button>>,
) {
    for (_, mut interaction, hover, pressed, disabled) in &mut buttons {
        if disabled.is_some() {
            interaction.set_if_neq(Interaction::None);
            continue;
        }

        if pressed.map(|state| state.pressed).unwrap_or(false) {
            interaction.set_if_neq(Interaction::Pressed);
        } else if hover.map(|state| state.hovered).unwrap_or(false) {
            interaction.set_if_neq(Interaction::Hovered);
        } else {
            interaction.set_if_neq(Interaction::None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interaction_config_thresholds_are_sane() {
        let config = InteractionConfig::default();
        assert!(config.click_max_duration_secs > 0.0);
        assert!(config.click_max_distance > 0.0);
        assert!(config.drag_start_distance >= config.click_max_distance);
    }
}
