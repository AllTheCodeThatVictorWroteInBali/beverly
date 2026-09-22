use std::collections::{HashMap, VecDeque};

use bevy::input_focus::tab_navigation::TabIndex;
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::*;

use crate::primitives::interaction::DisabledInteraction;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusOrigin {
    Keyboard,
    Pointer,
    Programmatic,
    Accessibility,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusNavigationPolicy {
    Sequential,
    Directional,
    Explicit,
    Composite,
}

impl Default for FocusNavigationPolicy {
    fn default() -> Self {
        Self::Sequential
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusDirection {
    Left,
    Right,
    Up,
    Down,
}

impl Default for FocusOrigin {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusRejectReason {
    Removed,
    NotFocusable,
    Disabled,
    Hidden,
    OutsideActiveScope,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Focusable {
    pub enabled: bool,
    pub tab_stop: bool,
    pub explicit_order: Option<i32>,
}

impl Default for Focusable {
    fn default() -> Self {
        Self {
            enabled: true,
            tab_stop: true,
            explicit_order: None,
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct FocusScope {
    pub trap: bool,
    pub active: bool,
    pub navigation_policy: FocusNavigationPolicy,
}

impl FocusScope {
    pub fn regular() -> Self {
        Self {
            trap: false,
            active: true,
            navigation_policy: FocusNavigationPolicy::Sequential,
        }
    }

    pub fn modal() -> Self {
        Self {
            trap: true,
            active: true,
            navigation_policy: FocusNavigationPolicy::Sequential,
        }
    }

    pub fn with_navigation_policy(mut self, policy: FocusNavigationPolicy) -> Self {
        self.navigation_policy = policy;
        self
    }
}

#[derive(Message, Clone, Copy, Debug)]
pub struct FocusRequest {
    pub target: Entity,
    pub origin: FocusOrigin,
}

#[derive(Message, Clone, Copy, Debug)]
pub struct FocusRejected {
    pub target: Entity,
    pub origin: FocusOrigin,
    pub reason: FocusRejectReason,
}

#[derive(Message, Clone, Copy, Debug)]
pub struct FocusChanged {
    pub previous: Option<Entity>,
    pub current: Option<Entity>,
    pub origin: FocusOrigin,
    pub active_scope: Option<Entity>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FocusOwnerState {
    pub current: Option<Entity>,
    pub previous: Option<Entity>,
    pub origin: FocusOrigin,
    pub active_scope: Option<Entity>,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum FocusRootId {
    Global,
}

#[derive(Resource, Debug)]
pub struct FocusManagerState {
    pub owners: HashMap<FocusRootId, FocusOwnerState>,
    pub active_root: FocusRootId,
}

impl Default for FocusManagerState {
    fn default() -> Self {
        let mut owners = HashMap::default();
        owners.insert(FocusRootId::Global, FocusOwnerState::default());
        Self {
            owners,
            active_root: FocusRootId::Global,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct FocusDebugSnapshot {
    pub current: Option<Entity>,
    pub previous: Option<Entity>,
    pub origin: FocusOrigin,
    pub active_scope: Option<Entity>,
    pub next: Option<Entity>,
    pub previous_in_order: Option<Entity>,
}

#[derive(Resource, Debug, Clone)]
pub struct FocusDebugTrace {
    pub capacity: usize,
    pub changes: VecDeque<String>,
    pub rejections: VecDeque<String>,
}

impl Default for FocusDebugTrace {
    fn default() -> Self {
        Self {
            capacity: 20,
            changes: VecDeque::new(),
            rejections: VecDeque::new(),
        }
    }
}

impl Default for FocusDebugSnapshot {
    fn default() -> Self {
        Self {
            current: None,
            previous: None,
            origin: FocusOrigin::Unknown,
            active_scope: None,
            next: None,
            previous_in_order: None,
        }
    }
}

#[derive(Resource, Default)]
struct FocusOrderCache {
    tab_sequence: HashMap<Option<Entity>, Vec<Entity>>,
    all_focusable: HashMap<Option<Entity>, Vec<Entity>>,
}

#[derive(Resource, Default)]
struct PendingFocusOrigin(Option<FocusOrigin>);

#[derive(Resource, Default)]
struct LastInputOrigin(FocusOrigin);

pub struct FocusPlugin;

/// PostUpdate focus requests include modal transitions produced in Update.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum FocusSystems {
    ApplyRequests,
    Sync,
}

impl Plugin for FocusPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FocusManagerState>()
            .init_resource::<FocusOrderCache>()
            .init_resource::<FocusDebugSnapshot>()
            .init_resource::<FocusDebugTrace>()
            .init_resource::<PendingFocusOrigin>()
            .init_resource::<LastInputOrigin>()
            .add_message::<FocusRequest>()
            .add_message::<FocusRejected>()
            .add_message::<FocusChanged>()
            .add_systems(
                PreUpdate,
                track_last_input_origin.after(bevy::input::InputSystems),
            )
            .add_systems(
                PostUpdate,
                (
                    ensure_focusable_defaults,
                    apply_focus_requests
                        .in_set(FocusSystems::ApplyRequests)
                        .after(bevy::camera::visibility::VisibilitySystems::VisibilityPropagate),
                    rebuild_focus_order_cache,
                    sync_focus_state.in_set(FocusSystems::Sync),
                    collect_focus_debug_trace,
                    refresh_focus_debug_snapshot,
                )
                    .chain()
                    .before(bevy::input_focus::InputFocusSystems::FocusChangeEvents),
            );
    }
}

fn track_last_input_origin(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut last: ResMut<LastInputOrigin>,
) {
    if keys.any_just_pressed([
        KeyCode::Tab,
        KeyCode::Enter,
        KeyCode::NumpadEnter,
        KeyCode::Space,
        KeyCode::ArrowLeft,
        KeyCode::ArrowRight,
        KeyCode::ArrowUp,
        KeyCode::ArrowDown,
    ]) {
        last.0 = FocusOrigin::Keyboard;
    } else if mouse.any_just_pressed([MouseButton::Left, MouseButton::Right, MouseButton::Middle]) {
        last.0 = FocusOrigin::Pointer;
    }
}

fn apply_focus_requests(
    mut requests: MessageReader<FocusRequest>,
    mut rejected: MessageWriter<FocusRejected>,
    mut pending_origin: ResMut<PendingFocusOrigin>,
    mut focus: ResMut<InputFocus>,
    scope_query: Query<(Entity, &FocusScope)>,
    parent_query: Query<&ChildOf>,
    focusable_query: Query<&Focusable>,
    tab_index_query: Query<&TabIndex>,
    disabled_query: Query<(), With<DisabledInteraction>>,
    visibility_query: Query<&InheritedVisibility>,
) {
    let active_trap_scope = scope_query
        .iter()
        .find_map(|(entity, scope)| (scope.trap && scope.active).then_some(entity));

    for request in requests.read() {
        let reason = validate_focus_target(
            request.target,
            active_trap_scope,
            &focusable_query,
            &tab_index_query,
            &disabled_query,
            &visibility_query,
            &parent_query,
        );

        if let Some(reason) = reason {
            rejected.write(FocusRejected {
                target: request.target,
                origin: request.origin,
                reason,
            });
            continue;
        }

        pending_origin.0 = Some(request.origin);
        focus.set(request.target, FocusCause::Navigated);
    }
}

fn ensure_focusable_defaults(
    mut commands: Commands,
    new_tab_nodes: Query<(Entity, &TabIndex), Added<TabIndex>>,
) {
    for (entity, tab_index) in &new_tab_nodes {
        commands.entity(entity).insert(Focusable {
            enabled: true,
            tab_stop: tab_index.0 >= 0,
            explicit_order: None,
        });
    }
}

fn rebuild_focus_order_cache(
    mut cache: ResMut<FocusOrderCache>,
    parent_query: Query<&ChildOf>,
    scope_query: Query<(), With<FocusScope>>,
    query: Query<(
        Entity,
        Option<&Focusable>,
        Option<&TabIndex>,
        Option<&InheritedVisibility>,
        Option<&DisabledInteraction>,
    )>,
) {
    let mut tab_sequence: HashMap<Option<Entity>, Vec<(i32, i32, u64, Entity)>> = HashMap::default();
    let mut all_focusable: HashMap<Option<Entity>, Vec<(i32, i32, u64, Entity)>> = HashMap::default();

    for (entity, focusable, tab_index, visibility, disabled) in &query {
        if disabled.is_some() {
            continue;
        }

        if visibility.map(|value| !value.get()).unwrap_or(false) {
            continue;
        }

        let (can_focus, tab_stop, explicit_order, order_index) = focusability_traits(focusable, tab_index);
        if !can_focus {
            continue;
        }

        let scope = nearest_scope(entity, &parent_query, &scope_query);
        let sort_key = (
            explicit_order.unwrap_or(i32::MAX),
            order_index,
            entity.to_bits(),
            entity,
        );

        all_focusable.entry(scope).or_default().push(sort_key);
        if tab_stop {
            tab_sequence.entry(scope).or_default().push(sort_key);
        }
    }

    cache.all_focusable.clear();
    cache.tab_sequence.clear();

    for (scope, mut values) in all_focusable {
        values.sort_by_key(|(explicit_order, tab_index, entity_index, _)| {
            (*explicit_order, *tab_index, *entity_index)
        });
        cache
            .all_focusable
            .insert(scope, values.into_iter().map(|(_, _, _, entity)| entity).collect());
    }

    for (scope, mut values) in tab_sequence {
        values.sort_by_key(|(explicit_order, tab_index, entity_index, _)| {
            (*explicit_order, *tab_index, *entity_index)
        });
        cache
            .tab_sequence
            .insert(scope, values.into_iter().map(|(_, _, _, entity)| entity).collect());
    }
}

fn sync_focus_state(
    focus: Res<InputFocus>,
    mut manager: ResMut<FocusManagerState>,
    parent_query: Query<&ChildOf>,
    scope_query: Query<(), With<FocusScope>>,
    last_input_origin: Res<LastInputOrigin>,
    mut pending_origin: ResMut<PendingFocusOrigin>,
    mut changed: MessageWriter<FocusChanged>,
) {
    if !focus.is_changed() {
        return;
    }

    let active_root = manager.active_root;
    let owner = manager
        .owners
        .entry(active_root)
        .or_insert(FocusOwnerState::default());

    let current = focus.get();
    let previous = owner.current;
    let origin = pending_origin.0.take().unwrap_or(last_input_origin.0);
    let active_scope = current.and_then(|entity| nearest_scope(entity, &parent_query, &scope_query));

    owner.previous = previous;
    owner.current = current;
    owner.origin = origin;
    owner.active_scope = active_scope;

    changed.write(FocusChanged {
        previous,
        current,
        origin,
        active_scope,
    });
}

fn refresh_focus_debug_snapshot(
    manager: Res<FocusManagerState>,
    cache: Res<FocusOrderCache>,
    mut snapshot: ResMut<FocusDebugSnapshot>,
) {
    let Some(owner) = manager.owners.get(&manager.active_root) else {
        return;
    };

    let scope = owner.active_scope;
    let sequence = cache.tab_sequence.get(&scope).cloned().unwrap_or_default();
    let (next, previous_in_order) = neighbor_nodes(owner.current, &sequence);

    *snapshot = FocusDebugSnapshot {
        current: owner.current,
        previous: owner.previous,
        origin: owner.origin,
        active_scope: owner.active_scope,
        next,
        previous_in_order,
    };
}

fn collect_focus_debug_trace(
    mut changes: MessageReader<FocusChanged>,
    mut rejections: MessageReader<FocusRejected>,
    mut trace: ResMut<FocusDebugTrace>,
) {
    for change in changes.read() {
        trace.changes.push_back(format!(
            "focus {:?} -> {:?} origin={:?} scope={:?}",
            change.previous,
            change.current,
            change.origin,
            change.active_scope
        ));
    }

    for rejection in rejections.read() {
        trace.rejections.push_back(format!(
            "reject target={:?} origin={:?} reason={:?}",
            rejection.target,
            rejection.origin,
            rejection.reason
        ));
    }

    while trace.changes.len() > trace.capacity {
        trace.changes.pop_front();
    }
    while trace.rejections.len() > trace.capacity {
        trace.rejections.pop_front();
    }
}

fn focusability_traits(
    focusable: Option<&Focusable>,
    tab_index: Option<&TabIndex>,
) -> (bool, bool, Option<i32>, i32) {
    match (focusable, tab_index) {
        (Some(focusable), Some(tab_index)) => (
            focusable.enabled,
            focusable.tab_stop,
            focusable.explicit_order,
            tab_index.0,
        ),
        (Some(focusable), None) => (
            focusable.enabled,
            focusable.tab_stop,
            focusable.explicit_order,
            0,
        ),
        (None, Some(tab_index)) => (true, tab_index.0 >= 0, None, tab_index.0),
        (None, None) => (false, false, None, 0),
    }
}

fn validate_focus_target(
    target: Entity,
    active_trap_scope: Option<Entity>,
    focusable_query: &Query<&Focusable>,
    tab_index_query: &Query<&TabIndex>,
    disabled_query: &Query<(), With<DisabledInteraction>>,
    visibility_query: &Query<&InheritedVisibility>,
    parent_query: &Query<&ChildOf>,
) -> Option<FocusRejectReason> {
    if disabled_query.get(target).is_ok() {
        return Some(FocusRejectReason::Disabled);
    }

    if visibility_query.get(target).map(|value| !value.get()).unwrap_or(false) {
        return Some(FocusRejectReason::Hidden);
    }

    let can_focus = focusable_query
        .get(target)
        .map(|focusable| focusable.enabled)
        .unwrap_or(false)
        || tab_index_query.get(target).is_ok();

    let has_any_focus_metadata = focusable_query.get(target).is_ok()
        || tab_index_query.get(target).is_ok()
        || visibility_query.get(target).is_ok()
        || disabled_query.get(target).is_ok();

    if !has_any_focus_metadata {
        return Some(FocusRejectReason::Removed);
    }

    if !can_focus {
        return Some(FocusRejectReason::NotFocusable);
    }

    if let Some(trap_scope) = active_trap_scope {
        if !is_descendant_or_self(target, trap_scope, parent_query) {
            return Some(FocusRejectReason::OutsideActiveScope);
        }
    }

    None
}

fn nearest_scope(
    entity: Entity,
    parent_query: &Query<&ChildOf>,
    scope_query: &Query<(), With<FocusScope>>,
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

fn neighbor_nodes(current: Option<Entity>, sequence: &[Entity]) -> (Option<Entity>, Option<Entity>) {
    let Some(current) = current else {
        return (sequence.first().copied(), sequence.last().copied());
    };

    if sequence.is_empty() {
        return (None, None);
    }

    let Some(index) = sequence.iter().position(|entity| *entity == current) else {
        return (sequence.first().copied(), sequence.last().copied());
    };

    let next = sequence.get((index + 1) % sequence.len()).copied();
    let previous = sequence
        .get((index + sequence.len() - 1) % sequence.len())
        .copied();

    (next, previous)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_index_negative_is_not_tab_stop_but_is_focusable() {
        let (can_focus, tab_stop, explicit_order, order_index) =
            focusability_traits(None, Some(&TabIndex(-1)));
        assert!(can_focus);
        assert!(!tab_stop);
        assert_eq!(explicit_order, None);
        assert_eq!(order_index, -1);
    }

    #[test]
    fn neighbor_nodes_wraps() {
        let a = Entity::from_bits(1);
        let b = Entity::from_bits(2);
        let c = Entity::from_bits(3);

        let (next, previous) = neighbor_nodes(Some(c), &[a, b, c]);
        assert_eq!(next, Some(a));
        assert_eq!(previous, Some(b));
    }
}
