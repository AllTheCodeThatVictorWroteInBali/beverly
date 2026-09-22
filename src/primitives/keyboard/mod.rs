use std::collections::{HashSet, VecDeque};

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input_focus::InputFocus;
use bevy::prelude::*;

use crate::components::checkbox::Checkbox;
use crate::primitives::focus::{
    FocusDirection,
    FocusNavigationPolicy,
    FocusOrigin,
    FocusRequest,
    FocusScope,
    Focusable,
};
use crate::components::input::TextInput;
use crate::primitives::interaction::{
    InteractionAction,
    InteractionActionEvent,
    InteractionActionSource,
    UiActionSystems,
};
use crate::components::link::Link;
use crate::components::slider::Slider;
use crate::components::textarea::Textarea;
use crate::components::toggle::Toggle;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KeyboardModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub command: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KeyboardEventPhase {
    Capture,
    #[default]
    Target,
    Bubble,
}

#[derive(Message, Clone, Debug)]
pub struct KeyboardEventNormalized {
    pub id: u64,
    pub key_code: KeyCode,
    pub logical_key: Key,
    pub modifiers: KeyboardModifiers,
    pub state: ButtonState,
    pub repeat: bool,
    pub timestamp_secs: f64,
}

#[derive(Message, Clone, Debug)]
pub struct UiKeyboardEvent {
    pub event: KeyboardEventNormalized,
    pub target: Entity,
    pub current_target: Entity,
    pub phase: KeyboardEventPhase,
    pub propagation_stopped: bool,
    pub default_prevented: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShortcutScope {
    FocusedComponent,
    ActiveFocusScope,
    Window,
    Application,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShortcutTarget {
    Focused,
    Entity(Entity),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShortcutChord {
    pub key_code: KeyCode,
    pub modifiers: KeyboardModifiers,
}

impl ShortcutChord {
    pub fn new(key_code: KeyCode) -> Self {
        Self {
            key_code,
            modifiers: KeyboardModifiers::default(),
        }
    }

    pub fn with_modifiers(mut self, modifiers: KeyboardModifiers) -> Self {
        self.modifiers = modifiers;
        self
    }

    fn matches(&self, event: &KeyboardEventNormalized) -> bool {
        self.key_code == event.key_code && self.modifiers == event.modifiers
    }
}

#[derive(Clone, Debug)]
pub struct ShortcutRegistration {
    pub id: String,
    pub description: String,
    pub scope: ShortcutScope,
    pub scope_entity: Option<Entity>,
    pub chord: ShortcutChord,
    pub action: InteractionAction,
    pub target: ShortcutTarget,
    pub enabled: bool,
    pub consume: bool,
}

impl ShortcutRegistration {
    pub fn new(
        id: impl Into<String>,
        description: impl Into<String>,
        chord: ShortcutChord,
        action: InteractionAction,
    ) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            scope: ShortcutScope::Application,
            scope_entity: None,
            chord,
            action,
            target: ShortcutTarget::Focused,
            enabled: true,
            consume: true,
        }
    }

    pub fn with_scope(mut self, scope: ShortcutScope) -> Self {
        self.scope = scope;
        self
    }

    pub fn for_scope_entity(mut self, scope_entity: Entity) -> Self {
        self.scope_entity = Some(scope_entity);
        self
    }

    pub fn with_target(mut self, target: ShortcutTarget) -> Self {
        self.target = target;
        self
    }

    pub fn consume(mut self, consume: bool) -> Self {
        self.consume = consume;
        self
    }
}

#[derive(Message, Clone, Debug)]
pub struct RegisterShortcut {
    pub registration: ShortcutRegistration,
}

#[derive(Message, Clone, Debug)]
pub struct UnregisterShortcut {
    pub id: String,
}

#[derive(Message, Clone, Debug)]
pub struct ShortcutMatched {
    pub id: String,
    pub action: InteractionAction,
    pub target: Entity,
    pub scope: ShortcutScope,
    pub event_id: u64,
}

#[derive(Resource, Default)]
pub struct ShortcutRegistry {
    pub entries: Vec<ShortcutRegistration>,
}

#[derive(Resource, Default)]
struct KeyboardEventCounter(u64);

#[derive(Resource, Default)]
struct ConsumedKeyboardEvents(HashSet<u64>);

#[derive(Resource, Clone, Debug)]
pub struct KeyboardDebugTrace {
    pub enabled: bool,
    pub capacity: usize,
    pub routing: VecDeque<String>,
    pub shortcuts: VecDeque<String>,
    pub actions: VecDeque<String>,
}

impl Default for KeyboardDebugTrace {
    fn default() -> Self {
        let enabled = matches!(
            std::env::var("UI_KEYBOARD_DEBUG").ok().as_deref(),
            Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("on")
        );
        Self {
            enabled,
            capacity: 20,
            routing: VecDeque::new(),
            shortcuts: VecDeque::new(),
            actions: VecDeque::new(),
        }
    }
}

#[derive(Resource, Default)]
struct KeyboardPressTracker {
    space_pressed_target: Option<Entity>,
}

pub struct KeyboardPlugin;

impl Plugin for KeyboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<KeyboardPressTracker>()
            .init_resource::<KeyboardEventCounter>()
            .init_resource::<ConsumedKeyboardEvents>()
            .init_resource::<ShortcutRegistry>()
            .init_resource::<KeyboardDebugTrace>()
            .add_message::<KeyboardEventNormalized>()
            .add_message::<UiKeyboardEvent>()
            .add_message::<RegisterShortcut>()
            .add_message::<UnregisterShortcut>()
            .add_message::<ShortcutMatched>()
            .add_systems(
                PreUpdate,
                (
                    clear_consumed_keyboard_event_ids,
                    apply_shortcut_registration,
                    normalize_keyboard_input,
                    route_keyboard_input_to_focus,
                    process_shortcuts,
                    resolve_directional_focus_navigation,
                    derive_default_keyboard_actions,
                )
                    .chain()
                    .in_set(UiActionSystems::Keyboard)
                    .after(bevy::input::InputSystems)
                    .after(bevy::input_focus::InputFocusSystems::Dispatch)
                    .after(UiActionSystems::Pointer),
            );
    }
}

fn clear_consumed_keyboard_event_ids(mut consumed: ResMut<ConsumedKeyboardEvents>) {
    consumed.0.clear();
}

fn apply_shortcut_registration(
    mut registry: ResMut<ShortcutRegistry>,
    mut registrations: MessageReader<RegisterShortcut>,
    mut removals: MessageReader<UnregisterShortcut>,
) {
    for registration in registrations.read() {
        registry
            .entries
            .retain(|existing| existing.id != registration.registration.id);
        registry.entries.push(registration.registration.clone());
    }

    for removal in removals.read() {
        registry.entries.retain(|existing| existing.id != removal.id);
    }
}

fn normalize_keyboard_input(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut counter: ResMut<KeyboardEventCounter>,
    mut keyboard: MessageReader<KeyboardInput>,
    mut out: MessageWriter<KeyboardEventNormalized>,
) {
    let modifiers = KeyboardModifiers {
        shift: keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight),
        ctrl: keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight),
        alt: keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight),
        command: keys.pressed(KeyCode::SuperLeft) || keys.pressed(KeyCode::SuperRight),
    };

    for event in keyboard.read() {
        counter.0 = counter.0.saturating_add(1);
        out.write(KeyboardEventNormalized {
            id: counter.0,
            key_code: event.key_code,
            logical_key: event.logical_key.clone(),
            modifiers,
            state: event.state,
            repeat: event.repeat,
            timestamp_secs: time.elapsed_secs_f64(),
        });
    }
}

fn route_keyboard_input_to_focus(
    focus: Res<InputFocus>,
    mut keyboard: MessageReader<KeyboardEventNormalized>,
    mut routed: MessageWriter<UiKeyboardEvent>,
    mut debug: ResMut<KeyboardDebugTrace>,
    parent_query: Query<&ChildOf>,
) {
    let Some(target) = focus.get() else {
        return;
    };
    let capacity = debug.capacity;

    for event in keyboard.read() {
        let ancestry = ancestry_path(target, &parent_query);
        push_trace(
            &mut debug.routing,
            capacity,
            format!(
                "id={} key={:?} state={:?} repeat={} target={:?} path_len={}",
                event.id,
                event.key_code,
                event.state,
                event.repeat,
                target,
                ancestry.len()
            ),
        );

        for entity in ancestry.iter().copied() {
            routed.write(UiKeyboardEvent {
                event: event.clone(),
                target,
                current_target: entity,
                phase: KeyboardEventPhase::Capture,
                propagation_stopped: false,
                default_prevented: false,
            });
        }

        routed.write(UiKeyboardEvent {
            event: event.clone(),
            target,
            current_target: target,
            phase: KeyboardEventPhase::Target,
            propagation_stopped: false,
            default_prevented: false,
        });

        for entity in ancestry.into_iter().rev() {
            routed.write(UiKeyboardEvent {
                event: event.clone(),
                target,
                current_target: entity,
                phase: KeyboardEventPhase::Bubble,
                propagation_stopped: false,
                default_prevented: false,
            });
        }
    }
}

fn derive_default_keyboard_actions(
    mut keyboard: MessageReader<UiKeyboardEvent>,
    mut actions: MessageWriter<InteractionActionEvent>,
    mut press_tracker: ResMut<KeyboardPressTracker>,
    consumed: Res<ConsumedKeyboardEvents>,
    mut debug: ResMut<KeyboardDebugTrace>,
    text_inputs: Query<(), With<TextInput>>,
    textareas: Query<(), With<Textarea>>,
    links: Query<(), With<Link>>,
    checkboxes: Query<(), With<Checkbox>>,
    switches: Query<(), With<Toggle>>,
    sliders: Query<(), With<Slider>>,
    generic_buttons: Query<(), With<Button>>,
) {
    let capacity = debug.capacity;
    for key_event in keyboard.read() {
        if key_event.phase != KeyboardEventPhase::Target {
            continue;
        }

        if consumed.0.contains(&key_event.event.id) {
            continue;
        }

        let target = key_event.target;

        if text_inputs.contains(target) || textareas.contains(target) {
            continue;
        }

        if checkboxes.contains(target) {
            if key_event.event.state.is_pressed()
                && key_event.event.key_code == KeyCode::Space
                && !key_event.event.repeat
            {
                actions.write(InteractionActionEvent {
                    action: InteractionAction::Toggle,
                    target,
                    pointer_id: None,
                    source: InteractionActionSource::Keyboard,
                    consumed: false,
                });
                push_trace(
                    &mut debug.actions,
                    capacity,
                    format!("id={} default=Toggle target={:?}", key_event.event.id, target),
                );
            }
            continue;
        }

        if switches.contains(target) {
            if key_event.event.state.is_pressed()
                && matches!(key_event.event.key_code, KeyCode::Space | KeyCode::Enter)
                && !key_event.event.repeat
            {
                actions.write(InteractionActionEvent {
                    action: InteractionAction::Toggle,
                    target,
                    pointer_id: None,
                    source: InteractionActionSource::Keyboard,
                    consumed: false,
                });
                push_trace(
                    &mut debug.actions,
                    capacity,
                    format!("id={} default=Toggle target={:?}", key_event.event.id, target),
                );
            }
            continue;
        }

        if sliders.contains(target) {
            if !key_event.event.state.is_pressed() || key_event.event.repeat {
                continue;
            }

            let mapped = match key_event.event.key_code {
                KeyCode::ArrowLeft | KeyCode::ArrowDown | KeyCode::PageDown => {
                    Some(InteractionAction::Decrement)
                }
                KeyCode::ArrowRight | KeyCode::ArrowUp | KeyCode::PageUp => {
                    Some(InteractionAction::Increment)
                }
                KeyCode::Home => Some(InteractionAction::SetMinimum),
                KeyCode::End => Some(InteractionAction::SetMaximum),
                _ => None,
            };

            if let Some(action) = mapped {
                actions.write(InteractionActionEvent {
                    action,
                    target,
                    pointer_id: None,
                    source: InteractionActionSource::Keyboard,
                    consumed: false,
                });
                push_trace(
                    &mut debug.actions,
                    capacity,
                    format!("id={} default={:?} target={:?}", key_event.event.id, action, target),
                );
            }
            continue;
        }

        if links.contains(target) {
            if key_event.event.state.is_pressed()
                && matches!(key_event.event.key_code, KeyCode::Enter | KeyCode::NumpadEnter)
                && !key_event.event.repeat
            {
                actions.write(InteractionActionEvent {
                    action: InteractionAction::Activate,
                    target,
                    pointer_id: None,
                    source: InteractionActionSource::Keyboard,
                    consumed: false,
                });
                push_trace(
                    &mut debug.actions,
                    capacity,
                    format!("id={} default=Activate(link) target={:?}", key_event.event.id, target),
                );
            }
            continue;
        }

        if !generic_buttons.contains(target) {
            continue;
        }

        match (key_event.event.key_code, key_event.event.state) {
            (KeyCode::Enter | KeyCode::NumpadEnter, ButtonState::Pressed) if !key_event.event.repeat => {
                actions.write(InteractionActionEvent {
                    action: InteractionAction::Activate,
                    target,
                    pointer_id: None,
                    source: InteractionActionSource::Keyboard,
                    consumed: false,
                });
                push_trace(
                    &mut debug.actions,
                    capacity,
                    format!("id={} default=Activate(button-enter) target={:?}", key_event.event.id, target),
                );
            }
            (KeyCode::Space, ButtonState::Pressed) if !key_event.event.repeat => {
                press_tracker.space_pressed_target = Some(target);
            }
            (KeyCode::Space, ButtonState::Released) => {
                if press_tracker.space_pressed_target == Some(target) {
                    actions.write(InteractionActionEvent {
                        action: InteractionAction::Activate,
                        target,
                        pointer_id: None,
                        source: InteractionActionSource::Keyboard,
                        consumed: false,
                    });
                    push_trace(
                        &mut debug.actions,
                        capacity,
                        format!("id={} default=Activate(button-space) target={:?}", key_event.event.id, target),
                    );
                }
                press_tracker.space_pressed_target = None;
            }
            _ => {}
        }
    }
}

fn process_shortcuts(
    mut keyboard: MessageReader<UiKeyboardEvent>,
    focus: Res<InputFocus>,
    registry: Res<ShortcutRegistry>,
    mut consumed: ResMut<ConsumedKeyboardEvents>,
    mut actions: MessageWriter<InteractionActionEvent>,
    mut matched: MessageWriter<ShortcutMatched>,
    mut debug: ResMut<KeyboardDebugTrace>,
    parent_query: Query<&ChildOf>,
    scope_query: Query<&FocusScope>,
) {
    let focused = focus.get();
    let active_scope = focused.and_then(|entity| nearest_scope(entity, &parent_query, &scope_query));
    let capacity = debug.capacity;

    for key_event in keyboard.read() {
        if key_event.phase != KeyboardEventPhase::Target {
            continue;
        }
        if !key_event.event.state.is_pressed() || key_event.event.repeat {
            continue;
        }

        let mut best: Option<&ShortcutRegistration> = None;
        let mut best_rank = i32::MAX;

        for entry in &registry.entries {
            if !entry.enabled || !entry.chord.matches(&key_event.event) {
                continue;
            }
            if !scope_matches(entry, focused, active_scope, &parent_query) {
                continue;
            }

            let rank = shortcut_scope_rank(entry.scope);
            if rank < best_rank {
                best_rank = rank;
                best = Some(entry);
            }
        }

        let Some(entry) = best else {
            continue;
        };

        let target = match entry.target {
            ShortcutTarget::Focused => {
                let Some(target) = focused else {
                    continue;
                };
                target
            }
            ShortcutTarget::Entity(entity) => entity,
        };

        actions.write(InteractionActionEvent {
            action: entry.action,
            target,
            pointer_id: None,
            source: InteractionActionSource::Keyboard,
            consumed: entry.consume,
        });

        matched.write(ShortcutMatched {
            id: entry.id.clone(),
            action: entry.action,
            target,
            scope: entry.scope,
            event_id: key_event.event.id,
        });

        push_trace(
            &mut debug.shortcuts,
            capacity,
            format!(
                "id={} shortcut={} scope={:?} action={:?} target={:?}",
                key_event.event.id,
                entry.id,
                entry.scope,
                entry.action,
                target
            ),
        );

        if entry.consume {
            consumed.0.insert(key_event.event.id);
        }
    }
}

fn resolve_directional_focus_navigation(
    mut keyboard: MessageReader<UiKeyboardEvent>,
    focus: Res<InputFocus>,
    mut focus_requests: MessageWriter<FocusRequest>,
    mut debug: ResMut<KeyboardDebugTrace>,
    parent_query: Query<&ChildOf>,
    scope_query: Query<&FocusScope>,
    focusables: Query<(
        Entity,
        &ComputedNode,
        Option<&UiGlobalTransform>,
        Option<&InheritedVisibility>,
        Option<&Focusable>,
        Option<&crate::primitives::interaction::DisabledInteraction>,
    )>,
    text_inputs: Query<(), With<TextInput>>,
    textareas: Query<(), With<Textarea>>,
    sliders: Query<(), With<Slider>>,
) {
    let Some(current) = focus.get() else {
        return;
    };
    let capacity = debug.capacity;

    for key_event in keyboard.read() {
        if key_event.phase != KeyboardEventPhase::Target {
            continue;
        }
        if !key_event.event.state.is_pressed() || key_event.event.repeat {
            continue;
        }

        if text_inputs.contains(current) || textareas.contains(current) || sliders.contains(current) {
            continue;
        }

        let Some(direction) = key_to_focus_direction(key_event.event.key_code) else {
            continue;
        };

        let scope = nearest_scope(current, &parent_query, &scope_query);
        let policy = scope
            .and_then(|entity| scope_query.get(entity).ok().map(|value| value.navigation_policy))
            .unwrap_or(FocusNavigationPolicy::Sequential);

        if !matches!(
            policy,
            FocusNavigationPolicy::Directional | FocusNavigationPolicy::Composite
        ) {
            continue;
        }

        let Some(next) = directional_candidate(current, direction, scope, &focusables, &parent_query, &scope_query)
        else {
            push_trace(
                &mut debug.routing,
                capacity,
                format!(
                    "id={} directional={:?} from={:?} scope={:?} result=none",
                    key_event.event.id,
                    direction,
                    current,
                    scope
                ),
            );
            continue;
        };

        focus_requests.write(FocusRequest {
            target: next,
            origin: FocusOrigin::Keyboard,
        });

        push_trace(
            &mut debug.routing,
            capacity,
            format!(
                "id={} directional={:?} from={:?} to={:?} scope={:?}",
                key_event.event.id,
                direction,
                current,
                next,
                scope
            ),
        );
    }
}

fn directional_candidate(
    current: Entity,
    direction: FocusDirection,
    scope: Option<Entity>,
    focusables: &Query<(
        Entity,
        &ComputedNode,
        Option<&UiGlobalTransform>,
        Option<&InheritedVisibility>,
        Option<&Focusable>,
        Option<&crate::primitives::interaction::DisabledInteraction>,
    )>,
    parent_query: &Query<&ChildOf>,
    scope_query: &Query<&FocusScope>,
) -> Option<Entity> {
    let Ok((_, current_node, current_transform, _, _, _)) = focusables.get(current) else {
        return None;
    };
    let current_center = node_center(current_node, current_transform);

    let mut best: Option<(f32, f32, f32, Entity)> = None;
    for (candidate, node, transform, visibility, focusable, disabled) in focusables.iter() {
        if candidate == current {
            continue;
        }
        if disabled.is_some() {
            continue;
        }
        if visibility.map(|value| !value.get()).unwrap_or(false) {
            continue;
        }

        let Some(focusable) = focusable else {
            continue;
        };
        if !focusable.enabled {
            continue;
        }

        let candidate_scope = nearest_scope(candidate, parent_query, scope_query);
        if candidate_scope != scope {
            continue;
        }

        let center = node_center(node, transform);
        let delta = center - current_center;

        let Some((primary, cross, distance)) = directional_score(direction, delta) else {
            continue;
        };

        let score = (primary, cross, distance, candidate);
        if best
            .as_ref()
            .map(|current_best| score_tuple_less(score, *current_best))
            .unwrap_or(true)
        {
            best = Some(score);
        }
    }

    best.map(|(_, _, _, entity)| entity)
}

fn node_center(node: &ComputedNode, transform: Option<&UiGlobalTransform>) -> Vec2 {
    let size = node.size() * node.inverse_scale_factor();
    let center = transform.map(|value| value.translation).unwrap_or(Vec2::ZERO);
    center + (node.content_box().min + size * 0.5)
}

fn directional_score(direction: FocusDirection, delta: Vec2) -> Option<(f32, f32, f32)> {
    let epsilon = 0.5;
    let (primary, cross) = match direction {
        FocusDirection::Right => (delta.x, delta.y.abs()),
        FocusDirection::Left => (-delta.x, delta.y.abs()),
        FocusDirection::Up => (delta.y, delta.x.abs()),
        FocusDirection::Down => (-delta.y, delta.x.abs()),
    };

    if primary <= epsilon {
        return None;
    }

    Some((primary, cross, delta.length()))
}

fn score_tuple_less(a: (f32, f32, f32, Entity), b: (f32, f32, f32, Entity)) -> bool {
    if (a.0 - b.0).abs() > f32::EPSILON {
        return a.0 < b.0;
    }
    if (a.1 - b.1).abs() > f32::EPSILON {
        return a.1 < b.1;
    }
    if (a.2 - b.2).abs() > f32::EPSILON {
        return a.2 < b.2;
    }
    a.3.to_bits() < b.3.to_bits()
}

fn key_to_focus_direction(key_code: KeyCode) -> Option<FocusDirection> {
    match key_code {
        KeyCode::ArrowLeft => Some(FocusDirection::Left),
        KeyCode::ArrowRight => Some(FocusDirection::Right),
        KeyCode::ArrowUp => Some(FocusDirection::Up),
        KeyCode::ArrowDown => Some(FocusDirection::Down),
        _ => None,
    }
}

fn scope_matches(
    entry: &ShortcutRegistration,
    focused: Option<Entity>,
    active_scope: Option<Entity>,
    parent_query: &Query<&ChildOf>,
) -> bool {
    match entry.scope {
        ShortcutScope::FocusedComponent => match focused {
            Some(focused) => match entry.scope_entity {
                Some(required) => required == focused,
                None => true,
            },
            None => false,
        },
        ShortcutScope::ActiveFocusScope => match active_scope {
            Some(active_scope) => match entry.scope_entity {
                Some(required_scope) => required_scope == active_scope,
                None => true,
            },
            None => false,
        },
        ShortcutScope::Window | ShortcutScope::Application => {
            if let Some(required_scope) = entry.scope_entity {
                focused
                    .map(|entity| is_descendant_or_self(entity, required_scope, parent_query))
                    .unwrap_or(false)
            } else {
                true
            }
        }
    }
}

fn shortcut_scope_rank(scope: ShortcutScope) -> i32 {
    match scope {
        ShortcutScope::FocusedComponent => 0,
        ShortcutScope::ActiveFocusScope => 1,
        ShortcutScope::Window => 2,
        ShortcutScope::Application => 3,
    }
}

fn nearest_scope(
    entity: Entity,
    parent_query: &Query<&ChildOf>,
    scope_query: &Query<&FocusScope>,
) -> Option<Entity> {
    if scope_query.contains(entity) {
        return Some(entity);
    }

    let mut cursor = Some(entity);
    while let Some(current) = cursor {
        let parent = parent_query.get(current).ok().map(ChildOf::parent);
        let Some(parent) = parent else {
            break;
        };

        if scope_query.contains(parent) {
            return Some(parent);
        }

        cursor = Some(parent);
    }

    None
}

fn is_descendant_or_self(
    candidate: Entity,
    ancestor: Entity,
    parent_query: &Query<&ChildOf>,
) -> bool {
    if candidate == ancestor {
        return true;
    }

    let mut cursor = Some(candidate);
    while let Some(current) = cursor {
        let parent = parent_query.get(current).ok().map(ChildOf::parent);
        let Some(parent) = parent else {
            return false;
        };

        if parent == ancestor {
            return true;
        }

        cursor = Some(parent);
    }

    false
}

fn push_trace(queue: &mut VecDeque<String>, capacity: usize, value: String) {
    queue.push_back(value);
    while queue.len() > capacity.max(1) {
        queue.pop_front();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyboard_modifiers_default_to_unset() {
        let modifiers = KeyboardModifiers::default();
        assert!(!modifiers.shift && !modifiers.ctrl && !modifiers.alt && !modifiers.command);
    }

    #[test]
    fn shortcut_scope_precedence_orders_most_specific_first() {
        assert!(shortcut_scope_rank(ShortcutScope::FocusedComponent)
            < shortcut_scope_rank(ShortcutScope::Application));
        assert!(shortcut_scope_rank(ShortcutScope::ActiveFocusScope)
            < shortcut_scope_rank(ShortcutScope::Window));
    }

    #[test]
    fn directional_score_filters_wrong_direction() {
        assert!(directional_score(FocusDirection::Right, Vec2::new(-10.0, 0.0)).is_none());
        assert!(directional_score(FocusDirection::Left, Vec2::new(10.0, 0.0)).is_none());
    }
}
