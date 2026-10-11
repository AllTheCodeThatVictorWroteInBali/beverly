use bevy::{
    color::Mix,
    input::{ButtonState, keyboard::KeyboardInput},
    prelude::*,
    ui::BorderColor,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::components::button::ButtonMotionDisabled;
use crate::icons::{Icon, IconNode};
use crate::primitives::a11y::{self, FocusGained, FocusLost};
use crate::primitives::semantic::{SemanticNode, SemanticRole, SemanticValue};
use crate::rendering::{Border, Paint, Surface};
use crate::theme::{ThemeMode, ThemeResource};

/// Same neutral palette as the default alert, button and avatar.
struct InputPalette {
    paper: Color,
    ink: Color,
    muted: Color,
    disabled_ink: Color,
    disabled_fill: Color,
    border: Color,
    error: Color,
}

fn input_palette(mode: ThemeMode) -> InputPalette {
    let gray = |v: u8| Color::srgb_u8(v, v, v);
    match mode {
        ThemeMode::Light => InputPalette {
            paper: gray(255),
            ink: gray(0),
            muted: gray(115),
            disabled_ink: gray(163),
            disabled_fill: gray(245),
            border: gray(212),
            error: Color::srgb_u8(220, 38, 38),
        },
        ThemeMode::Dark => InputPalette {
            paper: gray(23),
            ink: gray(255),
            muted: gray(163),
            disabled_ink: gray(82),
            disabled_fill: gray(38),
            border: gray(38),
            error: Color::srgb_u8(248, 113, 113),
        },
    }
}
#[derive(Component)]
pub struct TextInput {
    pub value: String,
    pub placeholder: String,
    pub floating_label: Option<String>,
    pub accessible_label: Option<String>,
    pub kind: TextInputKind,
    pub max_length: Option<usize>,
    pub cursor: usize,
    pub disabled: bool,
    pub read_only: bool,
    pub required: bool,
    pub invalid: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextInputKind {
    Text,
    Password,
    Search,
    Number,
    Email,
}

impl Default for TextInputKind {
    fn default() -> Self {
        Self::Text
    }
}

#[derive(Component)]
pub struct TextInputSurface;

#[derive(Component, Default)]
struct TextInputSubmitWobble {
    elapsed: f32,
    active: bool,
}

#[derive(Component)]
pub struct TextInputText;

#[derive(Component)]
pub struct TextInputPlaceholder;

#[derive(Component)]
pub struct TextInputCursor;

#[derive(Component)]
pub struct TextInputSearchIcon;

#[derive(Component)]
pub struct TextInputSubmitButton;

#[derive(Component)]
pub struct TextInputFloatingLabel;

#[derive(Component)]
pub struct TextInputFocused;

#[derive(Message, Debug, Clone)]
pub enum TextInputEvent {
    Changed { entity: Entity, value: String },
    Submitted { entity: Entity, value: String },
    Focused { entity: Entity },
    Unfocused { entity: Entity },
}

#[derive(Clone)]
pub struct TextInputConfig {
    pub placeholder: String,
    pub floating_label: Option<String>,
    pub accessible_label: Option<String>,
    pub kind: TextInputKind,
    pub max_length: Option<usize>,
    pub initial_value: String,
    pub disabled: bool,
    pub read_only: bool,
    pub required: bool,
    pub invalid: bool,
    pub border_radius: Option<f32>,
    pub width: Option<Val>,
    pub flex_grow: Option<f32>,
    pub show_submit_button: bool,
    pub char_filter: Option<fn(char) -> bool>,
    pub error_message: Option<String>,
}

impl TextInputConfig {
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            floating_label: None,
            accessible_label: None,
            kind: TextInputKind::Text,
            max_length: None,
            initial_value: String::new(),
            disabled: false,
            read_only: false,
            required: false,
            invalid: false,
            border_radius: None,
            width: None,
            flex_grow: None,
            show_submit_button: true,
            char_filter: None,
            error_message: Some("This character is not allowed.".to_string()),
        }
    }

    /// Overrides the default text shown under the input when a character is rejected.
    pub fn error_message(mut self, message: impl Into<String>) -> Self {
        self.error_message = Some(message.into());
        self
    }

    /// Rejects every character the filter returns `false` for. A rejected key press
    /// flashes the border red and shakes the input.
    pub fn char_filter(mut self, filter: fn(char) -> bool) -> Self {
        self.char_filter = Some(filter);
        self
    }

    pub fn kind(mut self, kind: TextInputKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn floating_label(mut self, floating_label: impl Into<String>) -> Self {
        self.floating_label = Some(floating_label.into());
        self
    }

    /// Sets the field's persistent accessible name (e.g. "Email address").
    ///
    /// This is the accessible name assistive technology announces. It is
    /// independent from `floating_label` (a visible label element) and from
    /// `placeholder` (a supplementary hint that disappears once populated
    /// and must never substitute for a real name).
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.accessible_label = Some(label.into());
        self
    }

    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    pub fn initial_value(mut self, initial_value: impl Into<String>) -> Self {
        self.initial_value = initial_value.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    pub fn border_radius(mut self, border_radius: f32) -> Self {
        self.border_radius = Some(border_radius);
        self
    }

    pub fn width(mut self, width: Val) -> Self {
        self.width = Some(width);
        self
    }

    pub fn flex_grow(mut self, flex_grow: f32) -> Self {
        self.flex_grow = Some(flex_grow);
        self
    }

    pub fn show_submit_button(mut self, show_submit_button: bool) -> Self {
        self.show_submit_button = show_submit_button;
        self
    }
}

pub fn spawn_text_input(parent: &mut ChildSpawnerCommands, config: TextInputConfig) -> Entity {
    let initial_value = config.initial_value;
    let initial_cursor = initial_value.chars().count();
    let is_search = config.kind == TextInputKind::Search;
    let has_floating_label = config.floating_label.is_some();
    let border_radius = if is_search {
        config.border_radius.unwrap_or(17.0)
    } else {
        config.border_radius.unwrap_or(8.0)
    };
    let semantic_role = match config.kind {
        TextInputKind::Search => SemanticRole::SearchBox,
        _ => SemanticRole::TextInput,
    };
    let text_input_hint = match config.kind {
        TextInputKind::Password => Some(crate::primitives::semantic::TextInputHint::Password),
        TextInputKind::Number => Some(crate::primitives::semantic::TextInputHint::Number),
        TextInputKind::Email => Some(crate::primitives::semantic::TextInputHint::Email),
        TextInputKind::Text | TextInputKind::Search => None,
    };
    let accessible_label = config
        .accessible_label
        .clone()
        .or_else(|| config.floating_label.clone());
    if accessible_label.is_none() {
        bevy::log::warn!(
            "TextInput (placeholder {:?}) has no accessible name: call `.label(...)` so screen \
             readers announce a persistent name instead of the placeholder hint.",
            config.placeholder,
        );
    }
    let mut semantic = SemanticNode::new(semantic_role);
    if let Some(label) = accessible_label.clone() {
        semantic = semantic.label(label);
    }
    semantic.text_input_hint = text_input_hint;
    semantic.state.disabled = config.disabled;
    semantic.semantic_value = SemanticValue::Text(initial_value.clone());

    parent
        .spawn((
            Button,
            ButtonMotionDisabled,
            a11y::TabIndex(if config.disabled { -1 } else { 0 }),
            semantic,
            TextInputReject::default(),
            TextInputSubmitWobble::default(),
            TextInputCharFilter(config.char_filter),
            TextInput {
                value: initial_value,
                placeholder: config.placeholder.clone(),
                floating_label: config.floating_label.clone(),
                accessible_label: config.accessible_label.clone(),
                kind: config.kind,
                max_length: config.max_length,
                cursor: initial_cursor,
                disabled: config.disabled,
                read_only: config.read_only,
                required: config.required,
                invalid: config.invalid,
            },
            Node {
                width: config.width.unwrap_or(percent(100)),
                height: if is_search {
                    px(68.0)
                } else if has_floating_label {
                    px(66.0)
                } else {
                    px(58.0)
                },
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                flex_grow: config.flex_grow.unwrap_or(0.0),
                padding: UiRect {
                    left: if is_search { px(12.0) } else { px(16.0) },
                    right: if is_search { px(0.0) } else { px(16.0) },
                    top: if has_floating_label {
                        px(14.0)
                    } else {
                        px(0.0)
                    },
                    bottom: if has_floating_label {
                        px(10.0)
                    } else {
                        px(0.0)
                    },
                },
                border: UiRect::all(px(1.0)),
                border_radius: BorderRadius::all(px(border_radius)),
                column_gap: if is_search { px(8.0) } else { px(0.0) },
                position_type: PositionType::Relative,
                overflow: Overflow::visible(),
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderColor::all(Color::NONE),
            TextInputSurface,
            Surface::rounded_rect_fill(border_radius, Paint::solid(Color::WHITE))
                .uniform_border(1.0, Paint::solid(Color::srgb_u8(212, 212, 212))),
        ))
        .with_children(|input| {
            if let Some(message) = config.error_message {
                input.spawn((
                    TextInputErrorIcon,
                    IconNode::new(Icon::feather("alert-circle"))
                        .size(14.0)
                        .color(Color::NONE),
                    Visibility::Hidden,
                    Node {
                        position_type: PositionType::Absolute,
                        top: percent(50),
                        right: px(12),
                        margin: UiRect::top(px(-7)),
                        width: px(14),
                        height: px(14),
                        ..default()
                    },
                ));
                input.spawn((
                    TextInputErrorMessage,
                    Text::new(message),
                    TextFont {
                        font_size: FontSize::Px(13.0),
                        ..default()
                    },
                    TextColor(Color::NONE),
                    Visibility::Hidden,
                    // Hangs below the field, so it shakes along with it.
                    Node {
                        position_type: PositionType::Absolute,
                        top: percent(100),
                        left: px(0),
                        margin: UiRect::top(px(6)),
                        ..default()
                    },
                ));
            }

            if let Some(label) = config.floating_label {
                input.spawn((
                    TextInputFloatingLabel,
                    Text::new(label),
                    TextFont {
                        font_size: FontSize::Px(16.0),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        position_type: PositionType::Absolute,
                        left: if is_search { px(40.0) } else { px(16.0) },
                        top: px(20.0),
                        ..default()
                    },
                ));
            }

            if is_search {
                input.spawn((
                    TextInputSearchIcon,
                    IconNode::new(Icon::feather("search"))
                        .size(14.0)
                        .color(Color::WHITE),
                    Node {
                        width: px(14.0),
                        height: px(14.0),
                        ..default()
                    },
                ));
            }

            input.spawn((
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(16.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                TextInputText,
                TextInputCursor,
                Node { ..default() },
            ));

            input.spawn((
                Text::new(config.placeholder),
                TextFont {
                    font_size: FontSize::Px(16.0),
                    ..default()
                },
                TextColor(Color::NONE),
                TextInputPlaceholder,
                Node {
                    position_type: PositionType::Absolute,
                    left: if is_search { px(40.0) } else { px(16.0) },
                    ..default()
                },
            ));

            if is_search && config.show_submit_button {
                input.spawn((
                    Node {
                        flex_grow: 1.0,
                        min_width: Val::Px(0.0),
                        ..default()
                    },
                    Visibility::Hidden,
                ));

                if config.show_submit_button {
                    input
                        .spawn((
                            Button,
                            TextInputSubmitButton,
                            Node {
                                width: px(52.0),
                                height: px(52.0),
                                border_radius: BorderRadius::all(px(26.0)),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                margin: UiRect::right(px(7.0)),
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                            BorderColor::all(Color::NONE),
                            Surface::rounded_rect_fill(
                                26.0,
                                Paint::solid(Color::srgb(0.14, 0.18, 0.24)),
                            )
                            .uniform_border(1.0, Paint::solid(Color::srgb(0.24, 0.30, 0.39))),
                        ))
                        .with_children(|submit| {
                            submit.spawn((
                                IconNode::new(Icon::feather("arrow-right"))
                                    .size(14.0)
                                    .color(Color::WHITE),
                                Node {
                                    width: px(14.0),
                                    height: px(14.0),
                                    ..default()
                                },
                            ));
                        });
                }
            }
        })
        .id()
}

fn text_input_focus(
    mut commands: Commands,
    interaction_query: Query<
        (Entity, &Interaction),
        (Changed<Interaction>, With<TextInputSurface>),
    >,
    focused_query: Query<Entity, With<TextInputFocused>>,
    input_query: Query<&TextInput>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut events: MessageWriter<TextInputEvent>,
) {
    let pressed_input = interaction_query
        .iter()
        .find_map(|(entity, interaction)| (*interaction == Interaction::Pressed).then_some(entity));

    if let Some(entity) = pressed_input {
        if input_query
            .get(entity)
            .map(|input| input.disabled)
            .unwrap_or(false)
        {
            return;
        }

        for focused in &focused_query {
            if focused == entity {
                continue;
            }

            commands.entity(focused).remove::<TextInputFocused>();
            events.write(TextInputEvent::Unfocused { entity: focused });
        }

        if !focused_query.contains(entity) {
            commands.entity(entity).insert(TextInputFocused);
            events.write(TextInputEvent::Focused { entity });
        }

        return;
    }

    if mouse.just_pressed(MouseButton::Left) {
        for focused in &focused_query {
            commands.entity(focused).remove::<TextInputFocused>();
            events.write(TextInputEvent::Unfocused { entity: focused });
        }
    }
}

fn text_input_submit_button(
    mut input_query: Query<(Entity, &TextInput, &mut TextInputSubmitWobble)>,
    submit_query: Query<
        (&ChildOf, &Interaction),
        (Changed<Interaction>, With<TextInputSubmitButton>),
    >,
    mut events: MessageWriter<TextInputEvent>,
) {
    for (child_of, interaction) in &submit_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let parent = child_of.parent();
        if let Ok((entity, input, mut wobble)) = input_query.get_mut(parent) {
            if input.disabled {
                continue;
            }

            wobble.elapsed = 0.0;
            wobble.active = true;
            events.write(TextInputEvent::Submitted {
                entity,
                value: input.value.clone(),
            });
        }
    }
}

const SUBMIT_WOBBLE_SECONDS: f32 = 0.32;
const SUBMIT_WOBBLE_DEGREES: f32 = 1.8;
const SUBMIT_WOBBLE_HERTZ: f32 = 5.0;
const SUBMIT_WOBBLE_DECAY: f32 = 9.0;

fn submit_wobble_angle(elapsed: f32) -> f32 {
    if elapsed >= SUBMIT_WOBBLE_SECONDS {
        return 0.0;
    }

    SUBMIT_WOBBLE_DEGREES.to_radians()
        * (std::f32::consts::TAU * SUBMIT_WOBBLE_HERTZ * elapsed).sin()
        * (-SUBMIT_WOBBLE_DECAY * elapsed).exp()
}

fn text_input_submit_wobble_system(
    time: Res<Time>,
    mut inputs: Query<(&mut UiTransform, &mut TextInputSubmitWobble), With<TextInputSurface>>,
) {
    for (mut transform, mut wobble) in &mut inputs {
        if !wobble.active {
            continue;
        }

        wobble.elapsed += time.delta_secs();
        if wobble.elapsed >= SUBMIT_WOBBLE_SECONDS {
            wobble.active = false;
            transform.rotation = Rot2::IDENTITY;
        } else {
            transform.rotation = Rot2::radians(submit_wobble_angle(wobble.elapsed));
        }
    }
}

/// Held-key state for the editing keys that repeat (the OS repeat events are ignored
/// for these so the rate is the same everywhere).
#[derive(Default)]
struct KeyRepeat {
    key: Option<KeyCode>,
    held: f32,
    since_last: f32,
}

const REPEAT_DELAY_SECS: f32 = 0.4;
const REPEAT_INTERVAL_SECS: f32 = 0.035;

fn is_repeatable_key(key: KeyCode) -> bool {
    matches!(
        key,
        KeyCode::Backspace | KeyCode::Delete | KeyCode::ArrowLeft | KeyCode::ArrowRight
    )
}

/// Applies one step of a repeatable key; returns whether the text changed.
fn apply_repeatable_key(input: &mut TextInput, key: KeyCode) -> bool {
    match key {
        KeyCode::ArrowLeft => {
            input.cursor = input.cursor.saturating_sub(1);
            false
        }
        KeyCode::ArrowRight => {
            input.cursor = (input.cursor + 1).min(grapheme_count(&input.value));
            false
        }
        KeyCode::Backspace if !input.read_only => {
            let mut cursor = input.cursor;
            let removed = remove_char_before_cursor(&mut input.value, &mut cursor);
            input.cursor = cursor;
            removed
        }
        KeyCode::Delete if !input.read_only => {
            let cursor = input.cursor;
            remove_char_at_cursor(&mut input.value, cursor)
        }
        _ => false,
    }
}

/// Extra per-input character filter (`None` accepts everything the kind allows).
#[derive(Component, Clone, Copy)]
struct TextInputCharFilter(Option<fn(char) -> bool>);

/// Rejection feedback: the border flashes red and the input shakes briefly.
#[derive(Component, Default)]
struct TextInputReject {
    elapsed: f32,
    active: bool,
    /// Seconds the error message stays on screen.
    message_left: f32,
}

#[derive(Component)]
struct TextInputErrorMessage;

#[derive(Component)]
struct TextInputErrorIcon;

const REJECT_MESSAGE_SECS: f32 = 2.0;
const REJECT_MESSAGE_FADE_SECS: f32 = 0.4;
const REJECT_FLASH_SECS: f32 = 0.6;
const REJECT_SHAKE_SECS: f32 = 0.4;
const REJECT_SHAKE_PX: f32 = 5.0;
const REJECT_SHAKE_HERTZ: f32 = 10.0;
const REJECT_SHAKE_DECAY: f32 = 8.0;

impl TextInputReject {
    fn start(&mut self) {
        self.elapsed = 0.0;
        self.active = true;
        self.message_left = REJECT_MESSAGE_SECS;
    }
}

fn reject_error_alpha(message_left: f32) -> f32 {
    (message_left / REJECT_MESSAGE_FADE_SECS).clamp(0.0, 1.0)
}

fn text_input_reject_system(
    time: Res<Time>,
    mut inputs: Query<(&mut TextInputReject, &mut UiTransform)>,
) {
    for (mut reject, mut transform) in &mut inputs {
        if reject.message_left > 0.0 {
            reject.message_left = (reject.message_left - time.delta_secs()).max(0.0);
        }

        if !reject.active {
            continue;
        }

        reject.elapsed += time.delta_secs();
        let t = reject.elapsed;
        if t >= REJECT_FLASH_SECS {
            reject.active = false;
        }

        let offset = if t < REJECT_SHAKE_SECS {
            REJECT_SHAKE_PX
                * (std::f32::consts::TAU * REJECT_SHAKE_HERTZ * t).sin()
                * (-REJECT_SHAKE_DECAY * t).exp()
        } else {
            0.0
        };
        transform.translation = Val2::px(offset, 0.0);
    }
}

/// Shows the error message in red, fading out at the end.
fn text_input_error_message_system(
    theme: Res<ThemeResource>,
    inputs: Query<(&TextInputReject, &Children)>,
    mut messages: Query<
        (&mut TextColor, &mut Visibility),
        (With<TextInputErrorMessage>, Without<TextInputErrorIcon>),
    >,
    mut icons: Query<
        (&mut IconNode, &mut Visibility),
        (With<TextInputErrorIcon>, Without<TextInputErrorMessage>),
    >,
) {
    let palette = input_palette(theme.current.mode);

    for (reject, children) in &inputs {
        let alpha = reject_error_alpha(reject.message_left);
        let visibility = if reject.message_left > 0.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };

        for child in children.iter() {
            if let Ok((mut icon, mut icon_visibility)) = icons.get_mut(child) {
                icon.color = palette.error.with_alpha(alpha);
                if *icon_visibility != visibility {
                    *icon_visibility = visibility;
                }
                continue;
            }
            let Ok((mut color, mut message_visibility)) = messages.get_mut(child) else {
                continue;
            };
            color.0 = palette.error.with_alpha(alpha);
            if *message_visibility != visibility {
                *message_visibility = visibility;
            }
        }
    }
}

fn text_input_keyboard(
    mut commands: Commands,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut repeat: Local<KeyRepeat>,
    mut keyboard: MessageReader<KeyboardInput>,
    mut focused_query: Query<
        (
            Entity,
            &mut TextInput,
            Option<&TextInputCharFilter>,
            Option<&mut TextInputReject>,
        ),
        With<TextInputFocused>,
    >,
    mut events: MessageWriter<TextInputEvent>,
) {
    let Some((entity, mut input, filter, mut reject)) = focused_query.iter_mut().next() else {
        repeat.key = None;
        return;
    };

    if input.disabled {
        repeat.key = None;
        return;
    }

    for event in keyboard.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        if is_repeatable_key(event.key_code) {
            if !event.repeat {
                if apply_repeatable_key(&mut input, event.key_code) {
                    events.write(TextInputEvent::Changed {
                        entity,
                        value: input.value.clone(),
                    });
                }
                *repeat = KeyRepeat {
                    key: Some(event.key_code),
                    ..default()
                };
            }
            continue;
        }

        if event.key_code == KeyCode::Escape {
            commands.entity(entity).remove::<TextInputFocused>();
            events.write(TextInputEvent::Unfocused { entity });
            continue;
        }

        if event.key_code == KeyCode::Enter {
            events.write(TextInputEvent::Submitted {
                entity,
                value: input.value.clone(),
            });
            continue;
        }

        if event.key_code == KeyCode::Home {
            input.cursor = 0;
            continue;
        }

        if event.key_code == KeyCode::End {
            input.cursor = grapheme_count(&input.value);
            continue;
        }

        let Some(text) = &event.text else {
            continue;
        };

        if input.read_only || text.is_empty() {
            continue;
        }

        let mut accepted = String::new();
        for ch in text.chars() {
            if ch.is_control() {
                continue;
            }

            if !allows_char(input.kind, ch)
                || filter
                    .and_then(|filter| filter.0)
                    .is_some_and(|allowed| !allowed(ch))
            {
                if let Some(reject) = reject.as_mut() {
                    reject.start();
                }
                continue;
            }

            if let Some(max) = input.max_length {
                let next_len = grapheme_count(&input.value) + grapheme_count(&accepted);
                if next_len >= max {
                    break;
                }
            }

            accepted.push(ch);
        }

        if accepted.is_empty() {
            continue;
        }

        let insert_at = byte_index_from_grapheme_index(&input.value, input.cursor);
        input.value.insert_str(insert_at, &accepted);
        input.cursor += grapheme_count(&accepted);

        events.write(TextInputEvent::Changed {
            entity,
            value: input.value.clone(),
        });
    }

    // Holding a repeatable key keeps applying it after a short delay.
    if let Some(key) = repeat.key {
        if !keys.pressed(key) {
            repeat.key = None;
            return;
        }

        let dt = time.delta_secs();
        repeat.held += dt;
        if repeat.held < REPEAT_DELAY_SECS {
            return;
        }

        repeat.since_last += dt;
        let mut changed = false;
        while repeat.since_last >= REPEAT_INTERVAL_SECS {
            repeat.since_last -= REPEAT_INTERVAL_SECS;
            changed |= apply_repeatable_key(&mut input, key);
        }
        if changed {
            events.write(TextInputEvent::Changed {
                entity,
                value: input.value.clone(),
            });
        }
    }
}

fn update_text_input_visuals(
    time: Res<Time>,
    theme: Res<ThemeResource>,
    mut inputs: Query<
        (
            Entity,
            &TextInput,
            Option<&TextInputFocused>,
            Option<&Children>,
            Option<&TextInputReject>,
            &mut Surface,
        ),
        With<TextInputSurface>,
    >,
    mut text_query: Query<
        (&ChildOf, &mut Text, &mut TextColor),
        (
            With<TextInputText>,
            Without<TextInputPlaceholder>,
            Without<TextInputFloatingLabel>,
        ),
    >,
    mut placeholder_query: Query<
        (&ChildOf, &mut Text, &mut TextColor, &mut Visibility),
        (
            With<TextInputPlaceholder>,
            Without<TextInputText>,
            Without<TextInputCursor>,
            Without<TextInputFloatingLabel>,
        ),
    >,
    mut label_query: Query<
        (
            &ChildOf,
            &mut Text,
            &mut TextColor,
            &mut TextFont,
            &mut Node,
        ),
        (
            With<TextInputFloatingLabel>,
            Without<TextInputText>,
            Without<TextInputPlaceholder>,
            Without<TextInputCursor>,
        ),
    >,
    mut search_icon_query: Query<(&ChildOf, &mut IconNode), With<TextInputSearchIcon>>,
) {
    let blink_on = (time.elapsed_secs() * 2.0).fract() < 0.5;
    let palette = input_palette(theme.current.mode);

    for (entity, input, focused, children, reject, mut surface) in &mut inputs {
        let is_focused = focused.is_some();
        let display_value = if input.kind == TextInputKind::Password {
            "*".repeat(grapheme_count(&input.value))
        } else {
            input.value.clone()
        };
        let display_value = if is_focused && !input.disabled && blink_on {
            insert_text_cursor(&display_value, input.cursor)
        } else {
            display_value
        };

        for (parent, mut text, mut text_color) in &mut text_query {
            if parent.parent() == entity {
                *text = Text::new(display_value.clone());
                text_color.0 = if input.disabled {
                    palette.disabled_ink
                } else {
                    palette.ink
                };
            }
        }

        let placeholder_visibility = if should_show_placeholder(&input.value, is_focused) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };

        for (parent, mut placeholder_text, mut placeholder_color, mut visibility) in
            &mut placeholder_query
        {
            if parent.parent() == entity {
                *placeholder_text = Text::new(input.placeholder.clone());
                placeholder_color.0 = palette.muted;
                *visibility = if input.floating_label.is_some() {
                    Visibility::Hidden
                } else {
                    placeholder_visibility
                };
            }
        }

        if let Some(label) = input.floating_label.as_ref() {
            let label_active = is_focused || !input.value.is_empty();

            for (parent, mut text, mut text_color, mut font, mut node) in &mut label_query {
                if parent.parent() == entity {
                    *text = Text::new(label.clone());
                    text_color.0 = if input.disabled {
                        palette.disabled_ink
                    } else if label_active {
                        palette.ink
                    } else {
                        palette.muted
                    };
                    font.font_size = FontSize::Px(if label_active { 12.0 } else { 16.0 });
                    node.top = if label_active { px(6.0) } else { px(20.0) };
                    node.left = if input.kind == TextInputKind::Search {
                        px(40.0)
                    } else {
                        px(16.0)
                    };
                }
            }
        }

        // Persistent invalid state stays red; rejection feedback fades with its message.
        let resting_outline = if is_focused && !input.disabled {
            palette.ink
        } else {
            palette.border
        };
        let outline = if input.invalid {
            palette.error
        } else {
            let error_alpha = reject.map_or(0.0, |reject| reject_error_alpha(reject.message_left));
            resting_outline.mix(&palette.error, error_alpha)
        };
        surface.fill = Paint::solid(if input.disabled {
            palette.disabled_fill
        } else {
            palette.paper
        });
        surface.border = Some(Border::new(1.0, Paint::solid(outline)));

        for (parent, mut icon_node) in &mut search_icon_query {
            if parent.parent() == entity {
                icon_node.color = if input.disabled {
                    palette.disabled_ink
                } else {
                    palette.muted
                };
            }
        }

        let _ = children;
    }
}

fn update_text_input_submit_button_visuals(
    theme: Res<ThemeResource>,
    mut submit_buttons: Query<(&mut Surface, &Children), With<TextInputSubmitButton>>,
    mut icons: Query<&mut IconNode>,
) {
    let palette = input_palette(theme.current.mode);

    // Inverted pill: ink-colored circle with a paper-colored arrow.
    for (mut button_surface, children) in &mut submit_buttons {
        button_surface.fill = Paint::solid(palette.ink);
        button_surface.border = Some(Border::new(1.0, Paint::solid(palette.ink)));

        for child in children.iter() {
            if let Ok(mut icon) = icons.get_mut(child) {
                if icon.color != palette.paper {
                    icon.color = palette.paper;
                }
            }
        }
    }
}

fn insert_text_cursor(value: &str, cursor: usize) -> String {
    let byte_index = byte_index_from_grapheme_index(value, cursor);
    let mut with_cursor = String::with_capacity(value.len() + 1);
    with_cursor.push_str(&value[..byte_index]);
    with_cursor.push('|');
    with_cursor.push_str(&value[byte_index..]);
    with_cursor
}

fn should_show_placeholder(value: &str, is_focused: bool) -> bool {
    value.is_empty() && !is_focused
}

fn allows_char(kind: TextInputKind, ch: char) -> bool {
    match kind {
        TextInputKind::Number => ch.is_ascii_digit(),
        TextInputKind::Email => ch.is_ascii_alphanumeric() || ".-_+@".contains(ch),
        TextInputKind::Text | TextInputKind::Password | TextInputKind::Search => true,
    }
}

fn byte_index_from_grapheme_index(value: &str, grapheme_index: usize) -> usize {
    value
        .grapheme_indices(true)
        .nth(grapheme_index)
        .map(|(idx, _)| idx)
        .unwrap_or(value.len())
}

fn grapheme_count(value: &str) -> usize {
    value.graphemes(true).count()
}

fn remove_char_before_cursor(value: &mut String, cursor: &mut usize) -> bool {
    if *cursor == 0 {
        return false;
    }

    let end = byte_index_from_grapheme_index(value, *cursor);
    let start = byte_index_from_grapheme_index(value, *cursor - 1);
    value.replace_range(start..end, "");
    *cursor -= 1;
    true
}

fn remove_char_at_cursor(value: &mut String, cursor: usize) -> bool {
    let len = grapheme_count(value);
    if cursor >= len {
        return false;
    }

    let start = byte_index_from_grapheme_index(value, cursor);
    let end = byte_index_from_grapheme_index(value, cursor + 1);
    value.replace_range(start..end, "");
    true
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::{
        TextInputConfig, TextInputEvent, TextInputKind, TextInputSubmitButton,
        TextInputSubmitWobble, allows_char, byte_index_from_grapheme_index, grapheme_count,
        insert_text_cursor, reject_error_alpha, remove_char_at_cursor, remove_char_before_cursor,
        should_show_placeholder, spawn_text_input, text_input_submit_button,
        text_input_submit_wobble_system,
    };
    use crate::components::button::ButtonMotionDisabled;

    #[test]
    fn clicking_search_submit_wobbles_the_whole_input() {
        let mut app = App::new();
        app.init_resource::<Time>()
            .add_message::<TextInputEvent>()
            .add_systems(
                Update,
                (text_input_submit_button, text_input_submit_wobble_system).chain(),
            );
        let parent = app.world_mut().spawn(Node::default()).id();
        let mut input_entity = None;
        {
            let mut commands = app.world_mut().commands();
            commands.entity(parent).with_children(|children| {
                input_entity = Some(spawn_text_input(
                    children,
                    TextInputConfig::new("Search")
                        .label("Search")
                        .kind(TextInputKind::Search),
                ));
            });
        }
        app.world_mut().flush();
        let input = input_entity.unwrap();
        let submit = app
            .world()
            .get::<Children>(input)
            .unwrap()
            .iter()
            .find(|entity| app.world().get::<TextInputSubmitButton>(*entity).is_some())
            .unwrap();

        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs_f32(1.0 / 60.0));
        *app.world_mut().get_mut::<Interaction>(submit).unwrap() = Interaction::Pressed;
        app.update();

        assert!(
            app.world()
                .get::<TextInputSubmitWobble>(input)
                .unwrap()
                .active
        );
        assert_ne!(
            app.world().get::<UiTransform>(input).unwrap().rotation,
            Rot2::IDENTITY
        );
    }

    #[test]
    fn search_field_stays_fixed_while_submit_button_keeps_shared_motion() {
        let mut app = App::new();
        let parent = app.world_mut().spawn(Node::default()).id();
        let mut input_entity = None;
        {
            let mut commands = app.world_mut().commands();
            commands.entity(parent).with_children(|children| {
                input_entity = Some(spawn_text_input(
                    children,
                    TextInputConfig::new("Search")
                        .label("Search")
                        .kind(TextInputKind::Search),
                ));
            });
        }
        app.world_mut().flush();

        let input = input_entity.unwrap();
        assert!(app.world().get::<ButtonMotionDisabled>(input).is_some());
        let submit = app
            .world()
            .get::<Children>(input)
            .unwrap()
            .iter()
            .find(|entity| app.world().get::<TextInputSubmitButton>(*entity).is_some())
            .unwrap();
        assert!(app.world().get::<ButtonMotionDisabled>(submit).is_none());
    }

    #[test]
    fn rejection_message_is_enabled_by_default_and_can_be_overridden() {
        assert_eq!(
            TextInputConfig::new("Name").error_message.as_deref(),
            Some("This character is not allowed.")
        );
        assert_eq!(
            TextInputConfig::new("Name")
                .error_message("Letters only")
                .error_message
                .as_deref(),
            Some("Letters only")
        );
    }

    #[test]
    fn rejection_feedback_uses_a_shared_fade_ramp() {
        assert_eq!(reject_error_alpha(0.4), 1.0);
        assert_eq!(reject_error_alpha(0.2), 0.5);
        assert_eq!(reject_error_alpha(0.0), 0.0);
    }

    #[test]
    fn text_inputs_accept_spaces() {
        assert!(allows_char(TextInputKind::Text, ' '));
        assert!(allows_char(TextInputKind::Password, ' '));
        assert!(allows_char(TextInputKind::Search, ' '));
        assert!(!allows_char(TextInputKind::Email, ' '));
    }

    #[test]
    fn shows_placeholder_when_empty_and_unfocused() {
        assert!(should_show_placeholder("", false));
    }

    #[test]
    fn hides_placeholder_when_focused_even_if_empty() {
        assert!(!should_show_placeholder("", true));
    }

    #[test]
    fn hides_placeholder_when_value_is_present() {
        assert!(!should_show_placeholder("hello", false));
    }

    #[test]
    fn text_cursor_is_inserted_at_the_requested_grapheme() {
        assert_eq!(insert_text_cursor("snack", 0), "|snack");
        assert_eq!(insert_text_cursor("snack", 3), "sna|ck");
        assert_eq!(insert_text_cursor("A👨‍👩‍👧‍👦B", 2), "A👨‍👩‍👧‍👦|B");
    }

    #[test]
    fn grapheme_cursor_treats_emoji_sequence_as_one_character() {
        let value = "A👨‍👩‍👧‍👦B";

        assert_eq!(grapheme_count(value), 3);
        assert_eq!(byte_index_from_grapheme_index(value, 1), 1);
        assert_eq!(byte_index_from_grapheme_index(value, 2), value.len() - 1);
    }

    #[test]
    fn deleting_combining_sequence_removes_the_whole_grapheme() {
        let mut value = "e\u{301}x".to_string();
        let mut cursor = 1;

        assert!(remove_char_before_cursor(&mut value, &mut cursor));
        assert_eq!(value, "x");
        assert_eq!(cursor, 0);
        assert!(remove_char_at_cursor(&mut value, cursor));
        assert_eq!(value, "");
    }
}

pub struct TextInputPlugin;

impl Plugin for TextInputPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<crate::primitives::semantic::SemanticAccessibilityPlugin>() {
            app.add_plugins(crate::primitives::semantic::SemanticAccessibilityPlugin);
        }
        app.add_message::<TextInputEvent>()
            .add_observer(on_text_input_focus_gained)
            .add_observer(on_text_input_focus_lost)
            .add_systems(
                Update,
                (
                    text_input_focus,
                    text_input_submit_button,
                    text_input_submit_wobble_system,
                    text_input_keyboard,
                    text_input_reject_system,
                    text_input_error_message_system,
                    update_text_input_visuals,
                    update_text_input_submit_button_visuals,
                )
                    .chain(),
            );
    }
}

/// Bridges keyboard (Tab) focus into this widget's own `TextInputFocused`
/// marker, so tabbing to a text field lets you type immediately — the same
/// as clicking into it.
fn on_text_input_focus_gained(
    trigger: On<FocusGained>,
    mut commands: Commands,
    surfaces: Query<(), With<TextInputSurface>>,
    input_query: Query<&TextInput>,
    focused_query: Query<Entity, With<TextInputFocused>>,
    mut events: MessageWriter<TextInputEvent>,
) {
    let entity = trigger.entity;
    if !surfaces.contains(entity) {
        return;
    }
    if input_query
        .get(entity)
        .map(|input| input.disabled)
        .unwrap_or(true)
    {
        return;
    }

    for focused in &focused_query {
        if focused != entity {
            commands.entity(focused).remove::<TextInputFocused>();
            events.write(TextInputEvent::Unfocused { entity: focused });
        }
    }

    if !focused_query.contains(entity) {
        commands.entity(entity).insert(TextInputFocused);
        events.write(TextInputEvent::Focused { entity });
    }
}

fn on_text_input_focus_lost(
    trigger: On<FocusLost>,
    mut commands: Commands,
    surfaces: Query<(), With<TextInputSurface>>,
    mut events: MessageWriter<TextInputEvent>,
) {
    let entity = trigger.entity;
    if !surfaces.contains(entity) {
        return;
    }
    commands.entity(entity).remove::<TextInputFocused>();
    events.write(TextInputEvent::Unfocused { entity });
}
