use bevy::prelude::*;
use bevy::{
    input::{ButtonState, keyboard::KeyboardInput},
    ui::BorderColor,
};

use crate::primitives::a11y::{self, FocusGained, FocusLost};
use crate::icons::{Icon, IconNode};
use crate::components::text::{TextRole, ThemedText};
use crate::theme::ThemeResource;
use crate::rendering::{Border, GradientStop, LinearGradient as UiLinearGradient, Paint, Surface};
#[derive(Component)]
pub struct TextInput {
    pub value: String,
    pub placeholder: String,
    pub floating_label: Option<String>,
    pub kind: TextInputKind,
    pub max_length: Option<usize>,
    pub cursor: usize,
    pub disabled: bool,
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
    pub kind: TextInputKind,
    pub max_length: Option<usize>,
    pub initial_value: String,
    pub disabled: bool,
    pub border_radius: Option<f32>,
    pub width: Option<Val>,
    pub flex_grow: Option<f32>,
    pub show_submit_button: bool,
}

impl TextInputConfig {
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            floating_label: None,
            kind: TextInputKind::Text,
            max_length: None,
            initial_value: String::new(),
            disabled: false,
            border_radius: None,
            width: None,
            flex_grow: None,
            show_submit_button: true,
        }
    }

    pub fn kind(mut self, kind: TextInputKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn floating_label(mut self, floating_label: impl Into<String>) -> Self {
        self.floating_label = Some(floating_label.into());
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
        config.border_radius.unwrap_or(0.0)
    };

    parent
        .spawn((
            Button,
            a11y::TabIndex(if config.disabled { -1 } else { 0 }),
            text_input_a11y_node(
                config.kind,
                config
                    .floating_label
                    .clone()
                    .unwrap_or_else(|| config.placeholder.clone()),
            ),
            TextInput {
                value: initial_value,
                placeholder: config.placeholder.clone(),
                floating_label: config.floating_label.clone(),
                kind: config.kind,
                max_length: config.max_length,
                cursor: initial_cursor,
                disabled: config.disabled,
            },
            Node {
                width: config.width.unwrap_or(percent(100)),
                height: if is_search {
                    px(34.0)
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
                    right: if is_search { px(12.0) } else { px(16.0) },
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
            Surface::rounded_rect_fill(
                border_radius,
                Paint::linear(UiLinearGradient::vertical(vec![
                    GradientStop::new(0.0, Color::srgb(0.10, 0.10, 0.13)),
                    GradientStop::new(1.0, Color::srgb(0.13, 0.13, 0.17)),
                ])),
            )
            .uniform_border(1.0, Paint::solid(Color::srgb(0.24, 0.24, 0.30))),
        ))
        .with_children(|input| {
            if let Some(label) = config.floating_label {
                input.spawn((
                    TextInputFloatingLabel,
                    ThemedText::new(TextRole::Label),
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
                ThemedText::new(TextRole::Body),
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                TextInputText,
                Node { ..default() },
            ));

            input.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new(config.placeholder),
                TextFont {
                    font_size: FontSize::Px(20.0),
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
                                width: px(28.0),
                                height: px(28.0),
                                border_radius: BorderRadius::all(px(14.0)),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                margin: UiRect::right(px(12.0)),
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                            BorderColor::all(Color::NONE),
                            Surface::rounded_rect_fill(
                                14.0,
                                Paint::solid(Color::srgb(0.14, 0.18, 0.24)),
                            )
                            .uniform_border(1.0, Paint::solid(Color::srgb(0.24, 0.30, 0.39))),
                        ))
                        .with_children(|submit| {
                            submit.spawn((
                                IconNode::new(Icon::feather("arrow-right")).size(14.0).color(Color::WHITE),
                                Node {
                                    width: px(14.0),
                                    height: px(14.0),
                                    ..default()
                                },
                            ));
                        });
                }
            }

            input.spawn((
                ThemedText::new(TextRole::Accent),
                Text::new("|"),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                TextInputCursor,
                Visibility::Hidden,
                Node {
                    margin: UiRect::left(px(-2.0)),
                    ..default()
                },
            ));
        })
        .id()
}

fn text_input_a11y_node(
    kind: TextInputKind,
    label: impl Into<String>,
) -> bevy::a11y::AccessibilityNode {
    match kind {
        TextInputKind::Text => a11y::text_input_node(label),
        TextInputKind::Password => a11y::password_input_node(label),
        TextInputKind::Search => a11y::search_input_node(label),
        TextInputKind::Number => a11y::number_input_node(label),
        TextInputKind::Email => a11y::email_input_node(label),
    }
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
    input_query: Query<(Entity, &TextInput)>,
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
        if let Ok((entity, input)) = input_query.get(parent) {
            if input.disabled {
                continue;
            }

            events.write(TextInputEvent::Submitted {
                entity,
                value: input.value.clone(),
            });
        }
    }
}

fn text_input_keyboard(
    mut commands: Commands,
    mut keyboard: MessageReader<KeyboardInput>,
    mut focused_query: Query<(Entity, &mut TextInput), With<TextInputFocused>>,
    mut events: MessageWriter<TextInputEvent>,
) {
    let Some((entity, mut input)) = focused_query.iter_mut().next() else {
        return;
    };

    if input.disabled {
        return;
    }

    for event in keyboard.read() {
        if event.state != ButtonState::Pressed {
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

        if event.key_code == KeyCode::ArrowLeft {
            input.cursor = input.cursor.saturating_sub(1);
            continue;
        }

        if event.key_code == KeyCode::ArrowRight {
            input.cursor = (input.cursor + 1).min(input.value.chars().count());
            continue;
        }

        if event.key_code == KeyCode::Home {
            input.cursor = 0;
            continue;
        }

        if event.key_code == KeyCode::End {
            input.cursor = input.value.chars().count();
            continue;
        }

        if event.key_code == KeyCode::Backspace {
            let mut cursor = input.cursor;
            if remove_char_before_cursor(&mut input.value, &mut cursor) {
                input.cursor = cursor;
                events.write(TextInputEvent::Changed {
                    entity,
                    value: input.value.clone(),
                });
            }
            continue;
        }

        if event.key_code == KeyCode::Delete {
            let cursor = input.cursor;
            if remove_char_at_cursor(&mut input.value, cursor) {
                events.write(TextInputEvent::Changed {
                    entity,
                    value: input.value.clone(),
                });
            }
            continue;
        }

        let Some(text) = &event.text else {
            continue;
        };

        if text.is_empty() {
            continue;
        }

        let mut accepted = String::new();
        for ch in text.chars() {
            if ch.is_control() {
                continue;
            }

            if !allows_char(input.kind, ch) {
                continue;
            }

            if let Some(max) = input.max_length {
                let next_len = input.value.chars().count() + accepted.chars().count();
                if next_len >= max {
                    break;
                }
            }

            accepted.push(ch);
        }

        if accepted.is_empty() {
            continue;
        }

        let insert_at = byte_index_from_char_index(&input.value, input.cursor);
        input.value.insert_str(insert_at, &accepted);
        input.cursor += accepted.chars().count();

        events.write(TextInputEvent::Changed {
            entity,
            value: input.value.clone(),
        });
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
            &mut Surface,
        ),
        With<TextInputSurface>,
    >,
    mut text_query: Query<
        (&ChildOf, &mut Text, &mut TextColor),
        (
            With<TextInputText>,
            Without<TextInputPlaceholder>,
            Without<TextInputCursor>,
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
    mut cursor_query: Query<
        (&ChildOf, &mut TextColor, &mut Visibility),
        (
            With<TextInputCursor>,
            Without<TextInputText>,
            Without<TextInputPlaceholder>,
            Without<TextInputFloatingLabel>,
        ),
    >,
    mut search_icon_query: Query<(&ChildOf, &mut IconNode), With<TextInputSearchIcon>>,
) {
    let blink_on = (time.elapsed_secs() * 2.0).fract() < 0.5;
    let colors = theme.current.colors;

    for (entity, input, focused, children, mut surface) in &mut inputs {
        let is_focused = focused.is_some();
        let display_value = if input.kind == TextInputKind::Password {
            "*".repeat(input.value.chars().count())
        } else {
            input.value.clone()
        };

        for (parent, mut text, mut text_color) in &mut text_query {
            if parent.parent() == entity {
                *text = Text::new(display_value.clone());
                text_color.0 = if input.disabled {
                    colors.text_disabled
                } else {
                    colors.text
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
                placeholder_color.0 = colors.text_muted;
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
                        colors.text_disabled
                    } else if label_active {
                        colors.focus
                    } else {
                        colors.text_muted
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

        let cursor_visibility = if is_focused && !input.disabled && blink_on {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };

        for (parent, mut cursor_color, mut visibility) in &mut cursor_query {
            if parent.parent() == entity {
                cursor_color.0 = colors.focus;
                *visibility = cursor_visibility;
            }
        }

        if input.kind == TextInputKind::Search {
            let is_dark = theme.current.mode == crate::theme::ThemeMode::Dark;
            surface.fill = Paint::solid(if is_dark {
                colors.surface.with_alpha(0.88)
            } else {
                colors.surface.with_alpha(0.96)
            });
            surface.border = Some(Border::new(
                1.0,
                Paint::solid(if is_focused {
                    colors.focus.with_alpha(0.95)
                } else if is_dark {
                    colors.border.with_alpha(0.92)
                } else {
                    colors.border.with_alpha(0.72)
                }),
            ));
        } else if input.disabled {
            surface.fill = Paint::solid(colors.surface_elevated);
            surface.border = Some(Border::new(1.0, Paint::solid(colors.border)));
        } else if is_focused {
            surface.fill = Paint::solid(colors.surface_elevated);
            surface.border = Some(Border::new(1.0, Paint::solid(colors.focus)));
        } else {
            surface.fill = Paint::solid(colors.surface);
            surface.border = Some(Border::new(1.0, Paint::solid(colors.border_strong)));
        }

        for (parent, mut icon_node) in &mut search_icon_query {
            if parent.parent() == entity {
                icon_node.color = if input.disabled {
                    colors.text_disabled
                } else {
                    colors.text_muted
                };
            }
        }

        let _ = children;
    }
}

fn update_text_input_submit_button_visuals(
    theme: Res<ThemeResource>,
    mut submit_buttons: Query<&mut Surface, With<TextInputSubmitButton>>,
) {
    let colors = theme.current.colors;

    for mut button_surface in &mut submit_buttons {
        button_surface.fill = Paint::solid(colors.secondary);
        button_surface.border = Some(Border::new(1.0, Paint::solid(colors.border)));
    }
}

fn should_show_placeholder(value: &str, is_focused: bool) -> bool {
    value.is_empty() && !is_focused
}

fn allows_char(kind: TextInputKind, ch: char) -> bool {
    match kind {
        TextInputKind::Number => ch.is_ascii_digit(),
        TextInputKind::Email => ch.is_ascii_alphanumeric() || ".-_@".contains(ch),
        TextInputKind::Text | TextInputKind::Password | TextInputKind::Search => true,
    }
}

fn byte_index_from_char_index(value: &str, char_index: usize) -> usize {
    value
        .char_indices()
        .nth(char_index)
        .map(|(idx, _)| idx)
        .unwrap_or(value.len())
}

fn remove_char_before_cursor(value: &mut String, cursor: &mut usize) -> bool {
    if *cursor == 0 {
        return false;
    }

    let end = byte_index_from_char_index(value, *cursor);
    let start = byte_index_from_char_index(value, *cursor - 1);
    value.replace_range(start..end, "");
    *cursor -= 1;
    true
}

fn remove_char_at_cursor(value: &mut String, cursor: usize) -> bool {
    let len = value.chars().count();
    if cursor >= len {
        return false;
    }

    let start = byte_index_from_char_index(value, cursor);
    let end = byte_index_from_char_index(value, cursor + 1);
    value.replace_range(start..end, "");
    true
}

#[cfg(test)]
mod tests {
    use super::should_show_placeholder;

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
}

pub struct TextInputPlugin;

impl Plugin for TextInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TextInputEvent>()
            .add_observer(on_text_input_focus_gained)
            .add_observer(on_text_input_focus_lost)
            .add_systems(
                Update,
                (
                    text_input_focus,
                    text_input_submit_button,
                    text_input_keyboard,
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
