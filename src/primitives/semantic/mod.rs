use std::collections::HashMap;

use bevy::a11y::AccessibilityNode;
use accesskit::{Action as AccessKitAction, Invalid, Live, Node as AccessKitNode, Role as AccessKitRole, Toggled};
use bevy::prelude::*;
use bevy::ui::UiSystems;

use crate::components::checkbox::{CheckboxEvent, CheckboxState};
use crate::components::button::BeverlyButton;
use crate::components::input::{TextInput, TextInputKind};
use crate::components::textarea::Textarea;
use crate::primitives::focus::{FocusChanged, FocusDebugSnapshot, FocusDebugTrace, FocusSystems};
use crate::primitives::interaction::{
    InteractionAction,
    InteractionActionEvent,
    InteractionActionSource,
    UiActionSystems,
};
use crate::primitives::keyboard::KeyboardDebugTrace;
use crate::components::slider::{Slider, SliderChanged};
use crate::components::toggle::{Toggle, ToggleEvent};
use crate::primitives::root::AppRootSurface;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnnouncementPriority {
    Polite,
    Assertive,
}

// Canonical, backend-agnostic role vocabulary. AccessKit distinguishes far
// more roles than this (e.g. password/number/email/multiline text inputs),
// but those distinctions are carried separately via `TextInputHint` on
// `SemanticNode` and resolved to a concrete `accesskit::Role` only inside
// `accesskit_role` - mapping to a specific accessibility backend's vocabulary
// stays a backend concern, not part of this crate's semantic model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemanticRole {
    Button,
    Link,

    Text,
    Heading,

    Image,

    TextInput,
    SearchBox,

    Checkbox,
    Switch,

    Radio,
    RadioGroup,

    Slider,

    ComboBox,
    ListBox,
    ListBoxOption,

    Tab,
    TabList,
    TabPanel,

    Menu,
    MenuItem,

    List,
    ListItem,

    Table,
    Row,
    Cell,
    ColumnHeader,
    RowHeader,

    ProgressBar,
    Status,
    Alert,

    Dialog,
    Tooltip,

    Separator,

    Generic,
}

/// Refines a [`SemanticRole::TextInput`] node with the backend-facing subtype
/// (masked/numeric/email keyboard hints, or a multiline editor) without
/// growing the canonical role vocabulary itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextInputHint {
    Password,
    Number,
    Email,
    Multiline,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SemanticState {
    pub disabled: bool,
    pub focused: bool,
    pub selected: bool,
    pub checked: Option<bool>,
    pub indeterminate: bool,
    pub expanded: Option<bool>,
    pub pressed: bool,
    pub busy: bool,
    pub read_only: bool,
    pub required: bool,
    pub invalid: bool,
    pub hidden: bool,
    pub modal: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum SemanticValue {
    #[default]
    None,
    Text(String),
    Numeric {
        value: f64,
        min: Option<f64>,
        max: Option<f64>,
    },
    Range {
        value: f64,
        min: f64,
        max: f64,
        step: Option<f64>,
    },
    Selection {
        selected: bool,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SemanticRelationships {
    pub labelled_by: Option<Entity>,
    pub described_by: Vec<Entity>,
    pub error_message: Option<Entity>,
    pub controls: Vec<Entity>,
    pub controlled_by: Option<Entity>,
    pub owns: Vec<Entity>,
    pub active_descendant: Option<Entity>,
}

#[derive(Component, Clone, Debug)]
pub struct SemanticNode {
    pub role: SemanticRole,
    pub text_input_hint: Option<TextInputHint>,
    pub label: Option<String>,
    pub description: Option<String>,
    pub value: Option<String>,
    pub semantic_value: SemanticValue,
    pub state: SemanticState,
    pub actions: Vec<InteractionAction>,
    pub relationships: SemanticRelationships,
    pub live: Option<AnnouncementPriority>,
    pub semantic_id: Option<String>,
    pub decorative: bool,
    pub accessibility_hidden: bool,
}

impl SemanticNode {
    pub fn new(role: SemanticRole) -> Self {
        Self {
            role,
            text_input_hint: None,
            label: None,
            description: None,
            value: None,
            semantic_value: SemanticValue::None,
            state: SemanticState::default(),
            actions: vec![InteractionAction::Activate],
            relationships: SemanticRelationships::default(),
            live: None,
            semantic_id: None,
            decorative: false,
            accessibility_hidden: false,
        }
    }

    pub fn text_input_hint(mut self, hint: TextInputHint) -> Self {
        self.text_input_hint = Some(hint);
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn semantic_value(mut self, value: SemanticValue) -> Self {
        self.semantic_value = value;
        self
    }

    pub fn semantic_id(mut self, semantic_id: impl Into<String>) -> Self {
        self.semantic_id = Some(semantic_id.into());
        self
    }

    pub fn decorative(mut self, decorative: bool) -> Self {
        self.decorative = decorative;
        self
    }

    pub fn accessibility_hidden(mut self, hidden: bool) -> Self {
        self.accessibility_hidden = hidden;
        self
    }
}

#[derive(Clone, Debug)]
pub struct SemanticSnapshotNode {
    pub role: SemanticRole,
    pub label: Option<String>,
    pub description: Option<String>,
    pub value: Option<String>,
    pub semantic_value: SemanticValue,
    pub state: SemanticState,
    pub actions: Vec<InteractionAction>,
    pub relationships: SemanticRelationships,
    pub live: Option<AnnouncementPriority>,
    pub semantic_id: Option<String>,
    pub parent: Option<Entity>,
    pub focused: bool,
    pub decorative: bool,
    pub accessibility_hidden: bool,
}

#[derive(Resource, Default)]
pub struct SemanticTreeSnapshot {
    pub generation: u64,
    pub nodes: HashMap<Entity, SemanticSnapshotNode>,
}

#[derive(Resource, Default)]
struct SemanticTreeDirty(bool);

#[derive(Resource, Clone, Debug)]
pub struct SemanticDebugSettings {
    pub enabled: bool,
    pub show_overlay: bool,
    pub max_nodes: usize,
    pub max_traces: usize,
}

impl Default for SemanticDebugSettings {
    fn default() -> Self {
        let enabled = matches!(
            std::env::var("UI_SEMANTIC_DEBUG").ok().as_deref(),
            Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("on")
        );
        Self {
            enabled,
            show_overlay: enabled,
            max_nodes: 18,
            max_traces: 6,
        }
    }
}

#[derive(Component)]
struct SemanticDebugOverlayRoot;

#[derive(Component)]
struct SemanticDebugOverlayText;

#[derive(Message, Clone, Debug)]
pub struct AccessibilityActionRequest {
    pub target: Entity,
    pub action: InteractionAction,
}

#[derive(Message, Clone, Debug)]
pub struct AutomationActionRequest {
    pub target: Entity,
    pub action: InteractionAction,
}

#[derive(Message, Clone, Debug)]
pub struct AccessibilityAnnouncement {
    pub message: String,
    pub priority: AnnouncementPriority,
}

pub struct SemanticPlugin;

pub(crate) struct SemanticAccessibilityPlugin;

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct SemanticAccessibilitySystems;

impl Plugin for SemanticAccessibilityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
                PostUpdate,
            sync_accesskit_from_semantics.in_set(SemanticAccessibilitySystems),
            )
            .configure_sets(
                PostUpdate,
                SemanticAccessibilitySystems
                    .after(crate::animation::skeleton::SkeletonSystems)
                    .before(UiSystems::Prepare),
            );
    }
}

impl Plugin for SemanticPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<SemanticAccessibilityPlugin>() {
            app.add_plugins(SemanticAccessibilityPlugin);
        }
        app.init_resource::<SemanticTreeSnapshot>()
            .init_resource::<SemanticTreeDirty>()
            .init_resource::<SemanticDebugSettings>()
            .add_message::<AccessibilityActionRequest>()
            .add_message::<AutomationActionRequest>()
            .add_message::<AccessibilityAnnouncement>()
            .add_systems(
                PreUpdate,
                (
                    bootstrap_semantic_nodes,
                    bridge_accessibility_actions,
                    bridge_automation_actions,
                    apply_semantic_actions,
                )
                    .chain()
                    .in_set(UiActionSystems::Apply)
                    .after(UiActionSystems::Pointer)
                    .after(UiActionSystems::Keyboard),
            )
            .add_systems(
                PostUpdate,
                // FocusManager -> SemanticTree -> AccessKit: focus state must
                // land on SemanticNode before the same-frame AccessKit
                // projection, so there is no second, lagging focus state.
                sync_focus_state_into_semantics
                    .after(FocusSystems::Sync)
                    .before(SemanticAccessibilitySystems),
            )
            .add_systems(
                PostUpdate,
                (mark_semantic_tree_dirty, rebuild_semantic_tree)
                    .chain()
                    .after(SemanticAccessibilitySystems),
            )
            .add_systems(
                Update,
                (
                    ensure_semantic_debug_overlay,
                    update_semantic_debug_overlay,
                )
                    .chain(),
            );
    }
}

fn sync_accesskit_from_semantics(
    mut commands: Commands,
    mut nodes: Query<
        (
            Entity,
            &SemanticNode,
            Option<&CheckboxState>,
            Option<&Toggle>,
            Option<&Slider>,
            Option<&BeverlyButton>,
            Option<&TextInput>,
            Option<&Textarea>,
            Option<&mut AccessibilityNode>,
        ),
        Or<(
            Added<SemanticNode>,
            Changed<SemanticNode>,
            Changed<CheckboxState>,
            Changed<Toggle>,
            Changed<Slider>,
            Changed<BeverlyButton>,
            Changed<TextInput>,
            Changed<Textarea>,
        )>,
    >,
) {
    for (entity, semantic, checkbox, toggle, slider, button, input, textarea, accessibility) in
        &mut nodes
    {
        let derived = derive_component_semantics(
            semantic, checkbox, toggle, slider, button, input, textarea,
        );
        let projected = project_semantic_node(&derived);
        if let Some(mut accessibility) = accessibility {
            accessibility.0 = projected;
        } else {
            commands.entity(entity).insert(AccessibilityNode(projected));
        }
    }
}

fn derive_component_semantics(
    semantic: &SemanticNode,
    checkbox: Option<&CheckboxState>,
    toggle: Option<&Toggle>,
    slider: Option<&Slider>,
    button: Option<&BeverlyButton>,
    input: Option<&TextInput>,
    textarea: Option<&Textarea>,
) -> SemanticNode {
    let mut derived = semantic.clone();

    if let Some(state) = checkbox {
        derived.role = SemanticRole::Checkbox;
        derived.state.checked = Some(state.checked);
        derived.state.indeterminate = state.indeterminate;
        derived.state.disabled = state.disabled;
    }
    if let Some(toggle) = toggle {
        derived.role = SemanticRole::Switch;
        derived.state.checked = Some(toggle.checked);
        derived.state.indeterminate = false;
        derived.state.disabled = toggle.disabled;
    }
    if let Some(slider) = slider {
        derived.role = SemanticRole::Slider;
        derived.state.disabled = slider.disabled;
        derived.semantic_value = SemanticValue::Range {
            value: slider.value as f64,
            min: slider.min as f64,
            max: slider.max as f64,
            step: slider.step.map(f64::from),
        };
        derived.actions = vec![InteractionAction::Increment, InteractionAction::Decrement];
    }
    if let Some(button) = button {
        derived.role = SemanticRole::Button;
        derived.label = Some(button.label.clone());
        derived.state.disabled = button.disabled;
    }
    if let Some(input) = input {
        derived.role = match input.kind {
            TextInputKind::Search => SemanticRole::SearchBox,
            _ => SemanticRole::TextInput,
        };
        derived.text_input_hint = match input.kind {
            TextInputKind::Password => Some(TextInputHint::Password),
            TextInputKind::Number => Some(TextInputHint::Number),
            TextInputKind::Email => Some(TextInputHint::Email),
            TextInputKind::Text | TextInputKind::Search => None,
        };
        derived.label = Some(
            input
                .floating_label
                .clone()
                .unwrap_or_else(|| input.placeholder.clone()),
        );
        derived.state.disabled = input.disabled;
        derived.state.read_only = input.read_only;
        derived.state.required = input.required;
        derived.state.invalid = input.invalid;
        derived.semantic_value = SemanticValue::Text(input.value.clone());
    }
    if let Some(textarea) = textarea {
        derived.role = SemanticRole::TextInput;
        derived.text_input_hint = Some(TextInputHint::Multiline);
        derived.semantic_value = SemanticValue::Text(textarea.value.clone());
    }

    derived
}

fn project_semantic_node(semantic: &SemanticNode) -> AccessKitNode {
    let mut node = AccessKitNode::new(accesskit_role(semantic));
    if let Some(label) = &semantic.label {
        node.set_label(label.clone());
    }
    if let Some(description) = &semantic.description {
        node.set_description(description.clone());
    }
    if let Some(value) = &semantic.value {
        node.set_value(value.clone());
    }

    match &semantic.semantic_value {
        SemanticValue::None => {}
        SemanticValue::Text(value) => node.set_value(value.clone()),
        SemanticValue::Numeric { value, min, max } => {
            node.set_numeric_value(*value);
            if let Some(min) = min {
                node.set_min_numeric_value(*min);
            }
            if let Some(max) = max {
                node.set_max_numeric_value(*max);
            }
        }
        SemanticValue::Range { value, min, max, step } => {
            node.set_numeric_value(*value);
            node.set_min_numeric_value(*min);
            node.set_max_numeric_value(*max);
            if let Some(step) = step {
                node.set_numeric_value_step(*step);
            }
        }
        SemanticValue::Selection { selected } => node.set_selected(*selected),
    }

    if semantic.state.disabled {
        node.set_disabled();
    }
    if semantic.state.required {
        node.set_required();
    }
    if semantic.state.read_only {
        node.set_read_only();
    }
    if semantic.state.busy {
        node.set_busy();
    }
    if semantic.state.hidden || semantic.decorative || semantic.accessibility_hidden {
        node.set_hidden();
    }
    if semantic.state.modal {
        node.set_modal();
    }
    if semantic.state.invalid {
        node.set_invalid(Invalid::True);
    }
    if let Some(live) = semantic.live {
        node.set_live(match live {
            AnnouncementPriority::Polite => Live::Polite,
            AnnouncementPriority::Assertive => Live::Assertive,
        });
    }
    if let Some(expanded) = semantic.state.expanded {
        node.set_expanded(expanded);
    }
    if semantic.state.selected {
        node.set_selected(true);
    }
    if let Some(checked) = semantic.state.checked {
        node.set_toggled(if semantic.state.indeterminate {
            Toggled::Mixed
        } else {
            Toggled::from(checked)
        });
    }

    for action in &semantic.actions {
        let action = match action {
            InteractionAction::Activate | InteractionAction::Select | InteractionAction::Toggle => {
                Some(AccessKitAction::Click)
            }
            InteractionAction::Dismiss | InteractionAction::Close => Some(AccessKitAction::Collapse),
            InteractionAction::Open => Some(AccessKitAction::Expand),
            InteractionAction::Increment => Some(AccessKitAction::Increment),
            InteractionAction::Decrement => Some(AccessKitAction::Decrement),
            InteractionAction::SetMinimum | InteractionAction::SetMaximum => None,
        };
        if let Some(action) = action {
            node.add_action(action);
        }
    }

    node
}

fn accesskit_role(semantic: &SemanticNode) -> AccessKitRole {
    match semantic.role {
        SemanticRole::Button => AccessKitRole::Button,
        SemanticRole::Link => AccessKitRole::Link,
        SemanticRole::Text => AccessKitRole::TextRun,
        SemanticRole::Heading => AccessKitRole::Heading,
        SemanticRole::Image => AccessKitRole::Image,
        SemanticRole::TextInput => match semantic.text_input_hint {
            Some(TextInputHint::Password) => AccessKitRole::PasswordInput,
            Some(TextInputHint::Number) => AccessKitRole::NumberInput,
            Some(TextInputHint::Email) => AccessKitRole::EmailInput,
            Some(TextInputHint::Multiline) => AccessKitRole::MultilineTextInput,
            None => AccessKitRole::TextInput,
        },
        SemanticRole::SearchBox => AccessKitRole::SearchInput,
        SemanticRole::Checkbox => AccessKitRole::CheckBox,
        SemanticRole::Switch => AccessKitRole::Switch,
        SemanticRole::Slider => AccessKitRole::Slider,
        SemanticRole::Radio => AccessKitRole::RadioButton,
        SemanticRole::RadioGroup => AccessKitRole::RadioGroup,
        SemanticRole::List => AccessKitRole::List,
        SemanticRole::ListItem => AccessKitRole::ListItem,
        SemanticRole::Menu => AccessKitRole::Menu,
        SemanticRole::MenuItem => AccessKitRole::MenuItem,
        SemanticRole::Tab => AccessKitRole::Tab,
        SemanticRole::TabList => AccessKitRole::TabList,
        SemanticRole::TabPanel => AccessKitRole::TabPanel,
        SemanticRole::ComboBox => AccessKitRole::ComboBox,
        SemanticRole::ListBox => AccessKitRole::ListBox,
        SemanticRole::ListBoxOption => AccessKitRole::ListBoxOption,
        SemanticRole::Table => AccessKitRole::Table,
        SemanticRole::Row => AccessKitRole::Row,
        SemanticRole::Cell => AccessKitRole::Cell,
        SemanticRole::ColumnHeader => AccessKitRole::ColumnHeader,
        SemanticRole::RowHeader => AccessKitRole::RowHeader,
        SemanticRole::ProgressBar => AccessKitRole::ProgressIndicator,
        SemanticRole::Status => AccessKitRole::Status,
        SemanticRole::Separator => AccessKitRole::Splitter,
        SemanticRole::Dialog => AccessKitRole::Dialog,
        SemanticRole::Alert => AccessKitRole::Alert,
        SemanticRole::Tooltip => AccessKitRole::Tooltip,
        SemanticRole::Generic => AccessKitRole::GenericContainer,
    }
}

fn bridge_accessibility_actions(
    mut requests: MessageReader<AccessibilityActionRequest>,
    mut actions: MessageWriter<InteractionActionEvent>,
) {
    for request in requests.read() {
        actions.write(InteractionActionEvent {
            action: request.action,
            target: request.target,
            pointer_id: None,
            source: InteractionActionSource::Accessibility,
            consumed: false,
        });
    }
}

fn bootstrap_semantic_nodes(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &AccessibilityNode,
            Option<&Button>,
            Option<&crate::components::link::Link>,
            Option<&crate::components::checkbox::Checkbox>,
            Option<&crate::components::toggle::Toggle>,
            Option<&crate::components::slider::Slider>,
        ),
        (Added<AccessibilityNode>, Without<SemanticNode>),
    >,
) {
    for (entity, accessibility, button, link, checkbox, switch, slider) in &query {
        let (role, text_input_hint) = if link.is_some() {
            (SemanticRole::Link, None)
        } else if checkbox.is_some() {
            (SemanticRole::Checkbox, None)
        } else if switch.is_some() {
            (SemanticRole::Switch, None)
        } else if slider.is_some() {
            (SemanticRole::Slider, None)
        } else if button.is_some() {
            (SemanticRole::Button, None)
        } else {
            semantic_role(accessibility.0.role())
        };

        let mut semantic = SemanticNode::new(role);
        semantic.text_input_hint = text_input_hint;
        semantic.label = accessibility.0.label().map(str::to_owned);
        semantic.description = accessibility.0.description().map(str::to_owned);
        semantic.value = accessibility.0.value().map(str::to_owned);
        semantic.state.disabled = accessibility.0.is_disabled();
        semantic.state.selected = accessibility.0.is_selected().unwrap_or(false);
        semantic.state.checked = accessibility.0.toggled().map(|toggled| toggled == Toggled::True);
        semantic.state.indeterminate = accessibility.0.toggled() == Some(Toggled::Mixed);
        semantic.state.expanded = accessibility.0.is_expanded();
        semantic.state.busy = accessibility.0.is_busy();
        semantic.state.read_only = accessibility.0.is_read_only();
        semantic.state.required = accessibility.0.is_required();
        semantic.state.invalid = accessibility.0.invalid().is_some();
        semantic.state.hidden = accessibility.0.is_hidden();
        semantic.state.modal = accessibility.0.is_modal();
        semantic.semantic_value = match (
            accessibility.0.numeric_value(),
            accessibility.0.min_numeric_value(),
            accessibility.0.max_numeric_value(),
        ) {
            (Some(value), Some(min), Some(max)) => SemanticValue::Range {
                value,
                min,
                max,
                step: accessibility.0.numeric_value_step(),
            },
            (Some(value), min, max) => SemanticValue::Numeric { value, min, max },
            _ => SemanticValue::None,
        };
        semantic.actions = [
            (AccessKitAction::Click, InteractionAction::Activate),
            (AccessKitAction::Expand, InteractionAction::Open),
            (AccessKitAction::Collapse, InteractionAction::Close),
            (AccessKitAction::Increment, InteractionAction::Increment),
            (AccessKitAction::Decrement, InteractionAction::Decrement),
        ]
        .into_iter()
        .filter_map(|(accesskit_action, semantic_action)| {
            accessibility
                .0
                .supports_action(accesskit_action)
                .then_some(semantic_action)
        })
        .collect();
        commands.entity(entity).insert(semantic);
    }
}

fn semantic_role(role: AccessKitRole) -> (SemanticRole, Option<TextInputHint>) {
    match role {
        AccessKitRole::Button | AccessKitRole::DefaultButton => (SemanticRole::Button, None),
        AccessKitRole::Link => (SemanticRole::Link, None),
        AccessKitRole::TextRun | AccessKitRole::Label | AccessKitRole::Paragraph => (SemanticRole::Text, None),
        AccessKitRole::Heading => (SemanticRole::Heading, None),
        AccessKitRole::Image => (SemanticRole::Image, None),
        AccessKitRole::MultilineTextInput => (SemanticRole::TextInput, Some(TextInputHint::Multiline)),
        AccessKitRole::PasswordInput => (SemanticRole::TextInput, Some(TextInputHint::Password)),
        AccessKitRole::NumberInput => (SemanticRole::TextInput, Some(TextInputHint::Number)),
        AccessKitRole::EmailInput => (SemanticRole::TextInput, Some(TextInputHint::Email)),
        AccessKitRole::TextInput => (SemanticRole::TextInput, None),
        AccessKitRole::SearchInput | AccessKitRole::Search => (SemanticRole::SearchBox, None),
        AccessKitRole::CheckBox => (SemanticRole::Checkbox, None),
        AccessKitRole::Switch => (SemanticRole::Switch, None),
        AccessKitRole::Slider => (SemanticRole::Slider, None),
        AccessKitRole::RadioButton => (SemanticRole::Radio, None),
        AccessKitRole::RadioGroup => (SemanticRole::RadioGroup, None),
        AccessKitRole::List => (SemanticRole::List, None),
        AccessKitRole::ListItem => (SemanticRole::ListItem, None),
        AccessKitRole::Menu => (SemanticRole::Menu, None),
        AccessKitRole::MenuItem => (SemanticRole::MenuItem, None),
        AccessKitRole::Tab => (SemanticRole::Tab, None),
        AccessKitRole::TabList => (SemanticRole::TabList, None),
        AccessKitRole::TabPanel => (SemanticRole::TabPanel, None),
        AccessKitRole::ComboBox => (SemanticRole::ComboBox, None),
        AccessKitRole::ListBox => (SemanticRole::ListBox, None),
        AccessKitRole::ListBoxOption => (SemanticRole::ListBoxOption, None),
        AccessKitRole::Table => (SemanticRole::Table, None),
        AccessKitRole::Row => (SemanticRole::Row, None),
        AccessKitRole::Cell | AccessKitRole::GridCell => (SemanticRole::Cell, None),
        AccessKitRole::ColumnHeader => (SemanticRole::ColumnHeader, None),
        AccessKitRole::RowHeader => (SemanticRole::RowHeader, None),
        AccessKitRole::ProgressIndicator => (SemanticRole::ProgressBar, None),
        AccessKitRole::Status => (SemanticRole::Status, None),
        AccessKitRole::Dialog | AccessKitRole::AlertDialog => (SemanticRole::Dialog, None),
        AccessKitRole::Alert => (SemanticRole::Alert, None),
        AccessKitRole::Tooltip => (SemanticRole::Tooltip, None),
        AccessKitRole::Splitter => (SemanticRole::Separator, None),
        AccessKitRole::Group | AccessKitRole::GenericContainer => (SemanticRole::Generic, None),
        _ => (SemanticRole::Generic, None),
    }
}

fn bridge_automation_actions(
    mut requests: MessageReader<AutomationActionRequest>,
    mut actions: MessageWriter<InteractionActionEvent>,
) {
    for request in requests.read() {
        actions.write(InteractionActionEvent {
            action: request.action,
            target: request.target,
            pointer_id: None,
            source: InteractionActionSource::Automation,
            consumed: false,
        });
    }
}

fn apply_semantic_actions(
    mut actions: MessageReader<InteractionActionEvent>,
    mut interactions: Query<&mut Interaction>,
    mut checkboxes: Query<&mut CheckboxState>,
    mut toggles: Query<&mut Toggle>,
    mut sliders: Query<&mut Slider>,
    mut checkbox_events: MessageWriter<CheckboxEvent>,
    mut toggle_events: MessageWriter<ToggleEvent>,
    mut slider_events: MessageWriter<SliderChanged>,
) {
    for action_event in actions.read() {
        let target = action_event.target;

        // These legacy widgets already act on the pointer-down Pressed edge.
        // Keep the tap action available to other readers, but do not replay it
        // as another Pressed edge on release. Non-pointer actions still bridge
        // into the legacy widget handlers until those handlers are migrated.
        if action_event.source == InteractionActionSource::Pointer
            && matches!(action_event.action, InteractionAction::Activate | InteractionAction::Toggle)
            && interactions.contains(target)
        {
            continue;
        }

        match action_event.action {
            InteractionAction::Activate => {
                if let Ok(mut interaction) = interactions.get_mut(target) {
                    interaction.set_if_neq(Interaction::Pressed);
                }
            }
            InteractionAction::Toggle => {
                if let Ok(mut interaction) = interactions.get_mut(target) {
                    interaction.set_if_neq(Interaction::Pressed);
                    continue;
                }

                if let Ok(mut state) = checkboxes.get_mut(target) {
                    if state.disabled {
                        continue;
                    }

                    if state.indeterminate {
                        state.indeterminate = false;
                        state.checked = true;
                    } else {
                        state.checked = !state.checked;
                    }

                    checkbox_events.write(CheckboxEvent::Changed {
                        entity: target,
                        checked: state.checked,
                        indeterminate: state.indeterminate,
                    });
                    continue;
                }

                if let Ok(mut toggle) = toggles.get_mut(target) {
                    if toggle.disabled {
                        continue;
                    }

                    toggle.checked = !toggle.checked;
                    toggle_events.write(ToggleEvent::Changed {
                        entity: target,
                        checked: toggle.checked,
                    });
                }
            }
            InteractionAction::Increment
            | InteractionAction::Decrement
            | InteractionAction::SetMinimum
            | InteractionAction::SetMaximum => {
                let Ok(mut slider) = sliders.get_mut(target) else {
                    continue;
                };
                if slider.disabled {
                    continue;
                }

                let step = slider
                    .step
                    .unwrap_or((slider.max - slider.min) * 0.01)
                    .max(f32::EPSILON);

                let next = match action_event.action {
                    InteractionAction::Increment => slider.value + step,
                    InteractionAction::Decrement => slider.value - step,
                    InteractionAction::SetMinimum => slider.min,
                    InteractionAction::SetMaximum => slider.max,
                    _ => slider.value,
                }
                .clamp(slider.min, slider.max);

                if (slider.value - next).abs() > f32::EPSILON {
                    slider.value = next;
                    slider_events.write(SliderChanged {
                        entity: target,
                        value: next,
                        dragging: false,
                    });
                }
            }
            _ => {}
        }
    }
}

fn sync_focus_state_into_semantics(
    mut focus_changes: MessageReader<FocusChanged>,
    mut nodes: Query<&mut SemanticNode>,
    mut dirty: ResMut<SemanticTreeDirty>,
) {
    for change in focus_changes.read() {
        if let Some(previous) = change.previous {
            if let Ok(mut node) = nodes.get_mut(previous) {
                node.state.focused = false;
                dirty.0 = true;
            }
        }

        if let Some(current) = change.current {
            if let Ok(mut node) = nodes.get_mut(current) {
                node.state.focused = true;
                dirty.0 = true;
            }
        }
    }
}

fn mark_semantic_tree_dirty(
    mut dirty: ResMut<SemanticTreeDirty>,
    changed_semantic: Query<Entity, (With<SemanticNode>, Or<(Added<SemanticNode>, Changed<SemanticNode>)>)>,
    changed_hierarchy: Query<Entity, (With<SemanticNode>, Changed<ChildOf>)>,
    changed_component_state: Query<
        Entity,
        Or<(
            Changed<CheckboxState>,
            Changed<Toggle>,
            Changed<Slider>,
            Changed<BeverlyButton>,
            Changed<TextInput>,
            Changed<Textarea>,
        )>,
    >,
    semantic_nodes: Query<(), With<SemanticNode>>,
    mut removed_semantic: RemovedComponents<SemanticNode>,
    mut removed_hierarchy: RemovedComponents<ChildOf>,
) {
    dirty.0 |= !changed_semantic.is_empty()
        || !changed_hierarchy.is_empty()
        || !changed_component_state.is_empty();

    // Always drain both readers, even when another change already marked the
    // tree dirty, so removals cannot cause redundant rebuilds on idle frames.
    for _ in removed_semantic.read() {
        dirty.0 = true;
    }

    for entity in removed_hierarchy.read() {
        // Only direct parents are stored in the snapshot. Despawned nodes are
        // covered by removed_semantic; unrelated hierarchy changes do not matter.
        dirty.0 |= semantic_nodes.contains(entity);
    }
}

fn rebuild_semantic_tree(
    mut tree: ResMut<SemanticTreeSnapshot>,
    mut dirty: ResMut<SemanticTreeDirty>,
    semantic_nodes: Query<(
        Entity,
        &SemanticNode,
        Option<&ChildOf>,
        Option<&CheckboxState>,
        Option<&Toggle>,
        Option<&Slider>,
        Option<&BeverlyButton>,
        Option<&TextInput>,
        Option<&Textarea>,
    )>,
) {
    if !dirty.0 {
        return;
    }

    tree.generation = tree.generation.saturating_add(1);
    tree.nodes.clear();

    for (entity, node, parent, checkbox, toggle, slider, button, input, textarea) in
        &semantic_nodes
    {
        let node = derive_component_semantics(
            node, checkbox, toggle, slider, button, input, textarea,
        );
        if node.decorative || node.accessibility_hidden {
            continue;
        }

        tree.nodes.insert(
            entity,
            SemanticSnapshotNode {
                role: node.role,
                label: node.label.clone(),
                    description: node.description.clone(),
                value: node.value.clone(),
                    semantic_value: node.semantic_value.clone(),
                state: node.state.clone(),
                actions: node.actions.clone(),
                    relationships: node.relationships.clone(),
                live: node.live,
                semantic_id: node.semantic_id.clone(),
                parent: parent.map(ChildOf::parent),
                focused: node.state.focused,
                decorative: node.decorative,
                accessibility_hidden: node.accessibility_hidden,
            },
        );
    }

    dirty.0 = false;
}

fn ensure_semantic_debug_overlay(
    mut commands: Commands,
    settings: Res<SemanticDebugSettings>,
    root_query: Query<Entity, With<AppRootSurface>>,
    overlay_query: Query<Entity, With<SemanticDebugOverlayRoot>>,
) {
    if !settings.enabled || !settings.show_overlay {
        return;
    }

    if !overlay_query.is_empty() {
        return;
    }

    let Some(root) = root_query.iter().next() else {
        return;
    };

    commands.entity(root).with_children(|parent| {
        parent
            .spawn((
                Name::new("semantic-focus-debug-overlay"),
                SemanticDebugOverlayRoot,
                Pickable::IGNORE,
                ZIndex(4500),
                Node {
                    position_type: PositionType::Absolute,
                    top: px(16.0),
                    right: px(16.0),
                    width: px(460.0),
                    min_height: px(180.0),
                    max_height: percent(90.0),
                    padding: UiRect::all(px(10.0)),
                    border: UiRect::all(px(1.0)),
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::clip(),
                    ..default()
                },
                BorderColor::all(Color::srgba(0.18, 0.55, 0.98, 0.8)),
                BackgroundColor(Color::srgba(0.02, 0.04, 0.08, 0.90)),
            ))
            .with_children(|overlay| {
                overlay.spawn((
                    SemanticDebugOverlayText,
                    Text::new("semantic debug initializing..."),
                    TextFont {
                        font_size: FontSize::Px(12.0),
                        ..default()
                    },
                    TextColor(Color::srgba(0.86, 0.93, 1.0, 0.98)),
                ));
            });
    });
}

fn update_semantic_debug_overlay(
    settings: Res<SemanticDebugSettings>,
    tree: Res<SemanticTreeSnapshot>,
    focus_debug: Res<FocusDebugSnapshot>,
    focus_trace: Res<FocusDebugTrace>,
    keyboard_debug: Res<KeyboardDebugTrace>,
    mut text_query: Query<&mut Text, With<SemanticDebugOverlayText>>,
) {
    if !settings.enabled || !settings.show_overlay {
        return;
    }

    let Ok(mut text) = text_query.single_mut() else {
        return;
    };

    let mut lines = Vec::new();
    lines.push("Semantic + Focus Inspector".to_string());
    lines.push(format!(
        "focus current={:?} previous={:?} origin={:?} scope={:?}",
        focus_debug.current,
        focus_debug.previous,
        focus_debug.origin,
        focus_debug.active_scope
    ));
    lines.push(format!(
        "focus next={:?} previous_in_order={:?}",
        focus_debug.next,
        focus_debug.previous_in_order
    ));

    lines.push("recent focus changes:".to_string());
    for entry in focus_trace
        .changes
        .iter()
        .rev()
        .take(settings.max_traces)
        .rev()
    {
        lines.push(format!("  {}", entry));
    }

    lines.push("recent focus rejections:".to_string());
    for entry in focus_trace
        .rejections
        .iter()
        .rev()
        .take(settings.max_traces)
        .rev()
    {
        lines.push(format!("  {}", entry));
    }
    lines.push(format!(
        "semantic generation={} nodes={}",
        tree.generation,
        tree.nodes.len()
    ));

    lines.push("recent keyboard routing:".to_string());
    for entry in keyboard_debug
        .routing
        .iter()
        .rev()
        .take(settings.max_traces)
        .rev()
    {
        lines.push(format!("  {}", entry));
    }

    lines.push("recent shortcuts:".to_string());
    for entry in keyboard_debug
        .shortcuts
        .iter()
        .rev()
        .take(settings.max_traces)
        .rev()
    {
        lines.push(format!("  {}", entry));
    }

    lines.push("recent keyboard actions:".to_string());
    for entry in keyboard_debug
        .actions
        .iter()
        .rev()
        .take(settings.max_traces)
        .rev()
    {
        lines.push(format!("  {}", entry));
    }

    let mut semantic_rows: Vec<_> = tree.nodes.iter().collect();
    semantic_rows.sort_by_key(|(entity, _)| entity.to_bits());
    lines.push("semantic tree sample:".to_string());
    for (entity, node) in semantic_rows.into_iter().take(settings.max_nodes) {
        lines.push(format!(
            "  {:?} role={:?} label={:?} value={:?} focused={} selected={} checked={:?} actions={:?}",
            entity,
            node.role,
            node.label,
            node.value,
            node.state.focused,
            node.state.selected,
            node.state.checked,
            node.actions
        ));
    }

    text.0 = lines.join("\n");
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::input::ButtonState;
    use bevy::input::keyboard::{Key, KeyboardInput};
    use bevy::input_focus::InputFocus;
    use crate::components::checkbox::{CheckboxConfig, CheckboxPlugin, spawn_checkbox};
    use crate::primitives::focus::FocusPlugin;
    use crate::primitives::interaction::*;
    use crate::primitives::keyboard::KeyboardPlugin;
    use crate::components::slider::component::SliderPlugin;
    use crate::theme::ThemeResource;
    use accesskit::{Action as AccessKitAction, Invalid, Role as AccessKitRole, Toggled};

    fn snapshot_app() -> App {
        let mut app = App::new();
        app.init_resource::<SemanticTreeSnapshot>()
            .init_resource::<SemanticTreeDirty>()
            .add_systems(
                PostUpdate,
                (mark_semantic_tree_dirty, rebuild_semantic_tree).chain(),
            );
        app
    }

    fn assert_idle_generation(app: &mut App, expected: u64) {
        for _ in 0..3 {
            app.update();
            assert_eq!(
                app.world().resource::<SemanticTreeSnapshot>().generation,
                expected,
                "idle frames must not rebuild the snapshot",
            );
        }
    }

    #[test]
    fn snapshot_derives_checkbox_state_and_rebuilds_on_component_change() {
        let mut app = snapshot_app();
        let entity = app
            .world_mut()
            .spawn((
                SemanticNode::new(SemanticRole::Text),
                CheckboxState {
                    checked: false,
                    disabled: false,
                    indeterminate: false,
                },
            ))
            .id();

        app.update();
        let tree = app.world().resource::<SemanticTreeSnapshot>();
        let node = &tree.nodes[&entity];
        assert_eq!(node.role, SemanticRole::Checkbox);
        assert_eq!(node.state.checked, Some(false));
        let generation = tree.generation;

        app.world_mut().get_mut::<CheckboxState>(entity).unwrap().checked = true;
        app.update();

        let tree = app.world().resource::<SemanticTreeSnapshot>();
        assert_eq!(tree.generation, generation + 1);
        assert_eq!(tree.nodes[&entity].state.checked, Some(true));
        assert_eq!(
            app.world().get::<SemanticNode>(entity).unwrap().role,
            SemanticRole::Text,
        );
    }

    #[test]
    fn snapshot_detach_clears_parent_and_drains_all_hierarchy_removals() {
        let mut app = snapshot_app();
        let parent = app.world_mut().spawn_empty().id();
        let children: Vec<_> = (0..3)
            .map(|_| app.world_mut().spawn((SemanticNode::new(SemanticRole::Button), ChildOf(parent))).id())
            .collect();
        app.update();
        let generation = app.world().resource::<SemanticTreeSnapshot>().generation;
        for &child in &children {
            assert_eq!(app.world().resource::<SemanticTreeSnapshot>().nodes[&child].parent, Some(parent));
            app.world_mut().entity_mut(child).remove::<ChildOf>();
        }

        app.update();
        let tree = app.world().resource::<SemanticTreeSnapshot>();
        assert_eq!(tree.generation, generation + 1);
        for child in children {
            assert_eq!(tree.nodes[&child].parent, None);
        }
        assert_idle_generation(&mut app, generation + 1);
    }

    #[test]
    fn snapshot_reparent_preserves_direct_nonsemantic_parent() {
        let mut app = snapshot_app();
        let ancestor = app.world_mut().spawn(SemanticNode::new(SemanticRole::List)).id();
        let first = app.world_mut().spawn(ChildOf(ancestor)).id();
        let second = app.world_mut().spawn(ChildOf(ancestor)).id();
        let child = app.world_mut().spawn((SemanticNode::new(SemanticRole::ListItem), ChildOf(first))).id();
        app.update();
        let tree = app.world().resource::<SemanticTreeSnapshot>();
        let generation = tree.generation;
        assert_eq!(tree.nodes[&child].parent, Some(first));
        assert!(!tree.nodes.contains_key(&first));

        app.world_mut().entity_mut(child).insert(ChildOf(second));
        app.update();
        let tree = app.world().resource::<SemanticTreeSnapshot>();
        assert_eq!(tree.generation, generation + 1);
        assert_eq!(tree.nodes[&child].parent, Some(second));
        assert!(!tree.nodes.contains_key(&second));
        assert_idle_generation(&mut app, generation + 1);
    }

    #[test]
    fn snapshot_despawn_removes_subtree_and_drains_all_semantic_removals() {
        let mut app = snapshot_app();
        let parent = app.world_mut().spawn(SemanticNode::new(SemanticRole::List)).id();
        let child = app.world_mut().spawn((SemanticNode::new(SemanticRole::ListItem), ChildOf(parent))).id();
        let grandchild = app.world_mut().spawn((SemanticNode::new(SemanticRole::Text), ChildOf(child))).id();
        let survivor = app.world_mut().spawn(SemanticNode::new(SemanticRole::Button)).id();
        app.update();
        let generation = app.world().resource::<SemanticTreeSnapshot>().generation;

        app.world_mut().entity_mut(parent).despawn();
        app.update();
        let tree = app.world().resource::<SemanticTreeSnapshot>();
        assert_eq!(tree.generation, generation + 1);
        for entity in [parent, child, grandchild] {
            assert!(!tree.nodes.contains_key(&entity));
        }
        assert_eq!(tree.nodes.len(), 1);
        assert!(tree.nodes.contains_key(&survivor));
        assert_idle_generation(&mut app, generation + 1);
    }

    #[test]
    fn snapshot_removal_readers_are_drained_even_when_other_changes_mark_dirty() {
        for dirty_source in ["semantic", "hierarchy", "already_dirty"] {
            let mut app = snapshot_app();
            let first = app.world_mut().spawn_empty().id();
            let second = app.world_mut().spawn_empty().id();
            let changed = app.world_mut().spawn((SemanticNode::new(SemanticRole::Button), ChildOf(first))).id();
            let detached = app.world_mut().spawn((SemanticNode::new(SemanticRole::Text), ChildOf(first))).id();
            let removed: Vec<_> = (0..3)
                .map(|_| app.world_mut().spawn(SemanticNode::new(SemanticRole::Text)).id())
                .collect();
            app.update();
            let generation = app.world().resource::<SemanticTreeSnapshot>().generation;

            match dirty_source {
                "semantic" => {
                    app.world_mut().get_mut::<SemanticNode>(changed).unwrap().label = Some("Updated".into());
                }
                "hierarchy" => {
                    app.world_mut().entity_mut(changed).insert(ChildOf(second));
                }
                "already_dirty" => app.world_mut().resource_mut::<SemanticTreeDirty>().0 = true,
                _ => unreachable!(),
            }
            app.world_mut().entity_mut(detached).remove::<ChildOf>();
            for &entity in &removed {
                app.world_mut().entity_mut(entity).remove::<SemanticNode>();
            }

            app.update();
            let tree = app.world().resource::<SemanticTreeSnapshot>();
            assert_eq!(tree.generation, generation + 1, "dirty source: {dirty_source}");
            assert_eq!(tree.nodes[&detached].parent, None);
            for entity in removed {
                assert!(!tree.nodes.contains_key(&entity));
            }
            assert_idle_generation(&mut app, generation + 1);
        }
    }

    #[test]
    fn snapshot_idle_and_nonsemantic_hierarchy_changes_keep_generation_stable() {
        let mut app = snapshot_app();
        assert_idle_generation(&mut app, 0);
        let ancestor = app.world_mut().spawn(SemanticNode::new(SemanticRole::List)).id();
        let wrapper = app.world_mut().spawn(ChildOf(ancestor)).id();
        let child = app.world_mut().spawn((SemanticNode::new(SemanticRole::ListItem), ChildOf(wrapper))).id();
        app.update();
        let generation = app.world().resource::<SemanticTreeSnapshot>().generation;
        assert_eq!(generation, 1);
        assert_idle_generation(&mut app, generation);

        // The snapshot stores the direct parent, not the nearest semantic ancestor.
        app.world_mut().entity_mut(wrapper).remove::<ChildOf>();
        assert_idle_generation(&mut app, generation);
        assert_eq!(app.world().resource::<SemanticTreeSnapshot>().nodes[&child].parent, Some(wrapper));
    }

    #[test]
    fn component_state_is_projected_without_mutating_semantic_metadata() {
        let semantic = SemanticNode::new(SemanticRole::Slider)
            .label("Volume")
            .description("Output volume")
            .semantic_value(SemanticValue::Range {
                value: 0.75,
                min: 0.0,
                max: 1.0,
                step: Some(0.05),
            });
        let mut semantic = semantic;
        semantic.state.disabled = true;
        semantic.state.required = true;
        semantic.state.invalid = true;
        semantic.actions = vec![InteractionAction::Increment, InteractionAction::Decrement];

        let projected = project_semantic_node(&semantic);
        assert_eq!(projected.role(), AccessKitRole::Slider);
        assert_eq!(projected.label(), Some("Volume"));
        assert_eq!(projected.description(), Some("Output volume"));
        assert_eq!(projected.numeric_value(), Some(0.75));
        assert_eq!(projected.min_numeric_value(), Some(0.0));
        assert_eq!(projected.max_numeric_value(), Some(1.0));
        assert_eq!(projected.numeric_value_step(), Some(0.05));
        assert!(projected.is_disabled());
        assert!(projected.is_required());
        assert_eq!(projected.invalid(), Some(Invalid::True));
        assert!(projected.supports_action(AccessKitAction::Increment));
        assert!(projected.supports_action(AccessKitAction::Decrement));

        semantic.role = SemanticRole::Checkbox;
        semantic.semantic_value = SemanticValue::None;
        semantic.state.checked = Some(true);
        semantic.state.indeterminate = true;
        let projected = project_semantic_node(&semantic);
        assert_eq!(projected.role(), AccessKitRole::CheckBox);
        assert_eq!(projected.toggled(), Some(Toggled::Mixed));

        let mut app = App::new();
        app.add_systems(Update, sync_accesskit_from_semantics);
        let entity = app.world_mut().spawn((
            semantic,
            AccessibilityNode(AccessKitNode::new(AccessKitRole::Unknown)),
            crate::components::checkbox::Checkbox,
            CheckboxState {
                checked: true,
                disabled: false,
                indeterminate: true,
            },
        )).id();
        app.update();
        let projected = &app.world().get::<AccessibilityNode>(entity).unwrap().0;
        assert_eq!(projected.role(), AccessKitRole::CheckBox);
        assert_eq!(projected.label(), Some("Volume"));
        assert_eq!(projected.toggled(), Some(Toggled::Mixed));
        assert!(!projected.is_disabled());
        let authored = app.world().get::<SemanticNode>(entity).unwrap();
        assert_eq!(authored.state.checked, Some(true));
        assert!(authored.state.indeterminate);
        assert_eq!(authored.state.disabled, true);
    }

    #[test]
    fn text_input_component_state_projects_value_and_constraints() {
        let mut app = App::new();
        app.add_systems(Update, sync_accesskit_from_semantics);
        let entity = app.world_mut().spawn((
            SemanticNode::new(SemanticRole::Text),
            TextInput {
                value: "secret".into(),
                placeholder: "Email address".into(),
                floating_label: None,
                kind: TextInputKind::Email,
                max_length: None,
                cursor: 6,
                disabled: true,
                read_only: true,
                required: true,
                invalid: true,
            },
        )).id();

        app.update();

        let projected = &app.world().get::<AccessibilityNode>(entity).unwrap().0;
        assert_eq!(projected.role(), AccessKitRole::EmailInput);
        assert_eq!(projected.label(), Some("Email address"));
        assert_eq!(projected.value(), Some("secret"));
        assert!(projected.is_disabled());
        assert!(projected.is_read_only());
        assert!(projected.is_required());
        assert_eq!(projected.invalid(), Some(Invalid::True));
    }

    fn action_app() -> App {
        let mut app = App::new();
        app.init_resource::<Time>()
            .init_resource::<InputFocus>()
            .init_resource::<ThemeResource>()
            .add_message::<ToggleEvent>()
            // Intentionally register consumers before producers. The shared
            // stages, not plugin registration order, must determine execution.
            .add_plugins((
                SemanticPlugin,
                SliderPlugin,
                CheckboxPlugin,
                FocusPlugin,
                KeyboardPlugin,
                crate::primitives::interaction::InteractionPlugin,
                bevy::input::InputPlugin,
            ));
        app.world_mut().resource_mut::<SemanticDebugSettings>().enabled = false;
        app
    }

    fn checkbox(app: &mut App) -> Entity {
        let mut entity = None;
        app.world_mut().commands().spawn(Node::default()).with_children(|parent| {
            entity = Some(spawn_checkbox(parent, CheckboxConfig::default()));
        });
        app.world_mut().flush();
        entity.unwrap()
    }

    fn pointer(app: &mut App, target: Entity, event_type: InteractionEventType, time: f64) {
        // Inject at the routed-event boundary: hit testing/OS input are not
        // part of these headless gesture -> action -> legacy consumer tests.
        app.world_mut().write_message(UiPointerEvent {
            event_type,
            pointer: PointerEvent {
                timestamp_secs: time,
                ..default()
            },
            context: InteractionEventContext {
                target,
                current_target: target,
                phase: InteractionEventPhase::Target,
                local_position: Vec2::ZERO,
            },
            propagation_stopped: false,
            default_prevented: false,
        });
    }

    fn key(app: &mut App, target: Entity, key_code: KeyCode, state: ButtonState, repeat: bool) {
        app.world_mut().write_message(KeyboardInput {
            key_code,
            logical_key: Key::Space,
            state,
            text: None,
            repeat,
            window: target,
        });
    }

    fn checkbox_changes(app: &mut App) -> usize {
        app.world_mut().resource_mut::<Messages<CheckboxEvent>>().drain().count()
    }

    #[test]
    fn held_pointer_activates_legacy_checkbox_once_without_release_replay() {
        for action in [InteractionAction::Activate, InteractionAction::Toggle] {
            let mut app = action_app();
            let target = checkbox(&mut app);
            app.update();
            app.world_mut().entity_mut(target).insert(ActionBinding { action });

            pointer(&mut app, target, InteractionEventType::PointerDown, 1.0);
            app.update();
            assert!(app.world().get::<CheckboxState>(target).unwrap().checked);
            assert_eq!(checkbox_changes(&mut app), 1);

            for _ in 0..3 {
                app.update();
                assert_eq!(*app.world().get::<Interaction>(target).unwrap(), Interaction::Pressed);
                assert!(app.world().get::<CheckboxState>(target).unwrap().checked);
                assert_eq!(checkbox_changes(&mut app), 0, "a held press must not retrigger");
            }

            pointer(&mut app, target, InteractionEventType::PointerUp, 1.1);
            app.update();
            assert_ne!(*app.world().get::<Interaction>(target).unwrap(), Interaction::Pressed);
            assert!(app.world().get::<CheckboxState>(target).unwrap().checked);
            assert_eq!(checkbox_changes(&mut app), 0, "tap release must not replay pointer-down");
            let actions: Vec<_> = app.world_mut()
                .resource_mut::<Messages<InteractionActionEvent>>().drain().collect();
            assert_eq!(actions.len(), 1, "tap remains available to independent action readers");
            assert_eq!(actions[0].action, action);
            assert_eq!(actions[0].source, InteractionActionSource::Pointer);
        }
    }

    #[test]
    fn keyboard_toggle_reaches_legacy_consumer_once_in_same_frame() {
        let mut app = action_app();
        let target = checkbox(&mut app);
        app.world_mut().insert_resource(InputFocus::from_entity(target));
        app.update();

        key(&mut app, target, KeyCode::Space, ButtonState::Pressed, false);
        app.update();
        assert!(app.world().get::<CheckboxState>(target).unwrap().checked);
        assert_eq!(checkbox_changes(&mut app), 1);
        for _ in 0..3 {
            app.update();
            assert_eq!(checkbox_changes(&mut app), 0);
        }
        key(&mut app, target, KeyCode::Space, ButtonState::Pressed, true);
        app.update();
        assert_eq!(checkbox_changes(&mut app), 0);
        key(&mut app, target, KeyCode::Space, ButtonState::Released, false);
        app.update();
        assert_eq!(checkbox_changes(&mut app), 0);
        key(&mut app, target, KeyCode::Space, ButtonState::Pressed, false);
        app.update();
        assert!(!app.world().get::<CheckboxState>(target).unwrap().checked);
        assert_eq!(checkbox_changes(&mut app), 1);
    }

    #[test]
    fn keyboard_slider_changes_once_and_preserves_home_end() {
        let mut app = action_app();
        let target = app.world_mut().spawn((Button, Slider::new(0.0, 100.0).value(10.0).step(2.0))).id();
        app.world_mut().insert_resource(InputFocus::from_entity(target));
        app.update();

        for (code, expected) in [
            (KeyCode::ArrowRight, 12.0),
            (KeyCode::ArrowUp, 14.0),
            (KeyCode::ArrowLeft, 12.0),
            (KeyCode::ArrowDown, 10.0),
            (KeyCode::PageUp, 12.0),
            (KeyCode::PageDown, 10.0),
            (KeyCode::End, 100.0),
            (KeyCode::ArrowRight, 100.0),
            (KeyCode::Home, 0.0),
            (KeyCode::ArrowLeft, 0.0),
        ] {
            let previous = app.world().get::<Slider>(target).unwrap().value;
            key(&mut app, target, code, ButtonState::Pressed, false);
            app.update();
            assert_eq!(app.world().get::<Slider>(target).unwrap().value, expected);
            let changes: Vec<_> = app.world_mut().resource_mut::<Messages<SliderChanged>>().drain().collect();
            assert_eq!(changes.len(), usize::from(previous != expected));
            for change in changes {
                assert_eq!(change.entity, target);
                assert_eq!(change.value, expected);
                assert!(!change.dragging);
            }
            key(&mut app, target, code, ButtonState::Pressed, true);
            app.update();
            assert_eq!(app.world().get::<Slider>(target).unwrap().value, expected);
            assert_eq!(app.world_mut().resource_mut::<Messages<SliderChanged>>().drain().count(), 0);
            key(&mut app, target, code, ButtonState::Released, false);
            app.update();
        }
    }

    #[test]
    fn focus_change_feeds_semantic_tree_in_the_same_frame() {
        let mut app = action_app();
        let first = checkbox(&mut app);
        let second = checkbox(&mut app);
        app.update();

        app.world_mut().insert_resource(InputFocus::from_entity(first));
        app.update();
        assert!(app.world().get::<SemanticNode>(first).unwrap().state.focused);
        assert!(app.world().resource::<SemanticTreeSnapshot>().nodes[&first].focused);

        app.world_mut().insert_resource(InputFocus::from_entity(second));
        app.update();
        assert!(!app.world().get::<SemanticNode>(first).unwrap().state.focused);
        assert!(app.world().get::<SemanticNode>(second).unwrap().state.focused);
        let tree = app.world().resource::<SemanticTreeSnapshot>();
        assert!(!tree.nodes[&first].focused);
        assert!(tree.nodes[&second].focused);
    }

    #[test]
    fn semantic_node_builder_sets_fields() {
        let node = SemanticNode::new(SemanticRole::Button)
            .label("Play")
            .value("Ready")
            .semantic_id("play_button");

        assert_eq!(node.role, SemanticRole::Button);
        assert_eq!(node.label.as_deref(), Some("Play"));
        assert_eq!(node.value.as_deref(), Some("Ready"));
        assert_eq!(node.semantic_id.as_deref(), Some("play_button"));
    }
}
