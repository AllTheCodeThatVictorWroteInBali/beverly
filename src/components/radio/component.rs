use bevy::a11y::AccessibilityNode;
use bevy::input_focus::InputFocus;
use bevy::prelude::*;

use crate::rendering::{Paint, Surface};
use crate::primitives::a11y::{self, FocusCause};
use crate::components::text::{TextRole, ThemedText};
use crate::theme::{ThemeColors, ThemeResource, dark_theme};

/// Identifies a radio group.
#[derive(Component)]
pub struct RadioGroup {
    pub id: String,
    pub selected: String,
}

/// Identifies an individual radio button.
#[derive(Component)]
pub struct RadioButton {
    pub group_id: String,
    pub value: String,
}

/// Marker for the visual indicator inside a radio button.
#[derive(Component)]
pub struct RadioIndicator;

#[derive(Component)]
struct RadioIndicatorPart {
    owner: Entity,
}

#[derive(Component)]
struct RadioDotPart {
    owner: Entity,
}

/// Marker for the radio button's label.
#[derive(Component)]
pub struct RadioLabel;

#[derive(Component)]
struct RadioLabelPart {
    owner: Entity,
}

/// Event emitted whenever a radio button is selected.
#[derive(Message, Debug, Clone)]
pub struct RadioChanged {
    pub entity: Entity,
    pub group_id: String,
    pub value: String,
}

/// Configuration for creating a radio group.
pub struct RadioOption {
    pub value: String,
    pub label: String,
}

impl RadioOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

/// Reusable radio group builder.
pub struct RadioGroupBuilder {
    id: String,
    options: Vec<RadioOption>,
    selected: Option<String>,
    width: Val,
    spacing: Val,
}

impl RadioGroupBuilder {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            options: Vec::new(),
            selected: None,
            width: Val::Auto,
            spacing: Val::Px(8.0),
        }
    }

    pub fn option(mut self, value: impl Into<String>, label: impl Into<String>) -> Self {
        self.options.push(RadioOption::new(value, label));
        self
    }

    pub fn options(
        mut self,
        options: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        self.options = options
            .into_iter()
            .map(|(value, label)| RadioOption::new(value, label))
            .collect();

        self
    }

    pub fn selected(mut self, value: impl Into<String>) -> Self {
        self.selected = Some(value.into());
        self
    }

    pub fn width(mut self, width: Val) -> Self {
        self.width = width;
        self
    }

    pub fn spacing(mut self, spacing: Val) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn spawn(self, parent: &mut ChildSpawnerCommands) -> Entity {
        let colors = dark_theme().colors;
        let selected = self
            .selected
            .clone()
            .or_else(|| self.options.first().map(|x| x.value.clone()))
            .unwrap_or_default();

        let group_id = self.id.clone();

        let mut group_commands = parent.spawn((
            Node {
                width: self.width,
                flex_direction: FlexDirection::Column,
                row_gap: self.spacing,
                ..default()
            },
            a11y::radio_group_node(self.id.clone()),
            RadioGroup {
                id: self.id.clone(),
                selected: selected.clone(),
            },
        ));

        let group = group_commands.id();

        group_commands.with_children(|group_parent| {
            for option in self.options {
                let is_selected = option.value == selected;
                let label_text = option.label.clone();

                let mut button_commands = group_parent.spawn((
                    Button,
                    a11y::TabIndex(if is_selected { 0 } else { -1 }),
                    a11y::radio_node(label_text, is_selected),
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(36.0),
                        align_items: AlignItems::Center,
                        padding: UiRect::horizontal(Val::Px(10.0)),
                        column_gap: Val::Px(8.0),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    RadioButton {
                        group_id: group_id.clone(),
                        value: option.value.clone(),
                    },
                ));

                let button_entity = button_commands.id();

                button_commands.with_children(|button_parent| {
                    button_parent
                        .spawn((
                            Node {
                                width: Val::Px(18.0),
                                height: Val::Px(18.0),
                                border: UiRect::all(Val::Px(2.0)),
                                border_radius: BorderRadius::MAX,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                            BorderColor::all(Color::NONE),
                            Surface::rounded_rect_fill(
                                9.0,
                                Paint::solid(radio_border_color(
                                    colors,
                                    is_selected,
                                    Interaction::None,
                                )),
                            ),
                            RadioIndicator,
                            RadioIndicatorPart {
                                owner: button_entity,
                            },
                        ))
                        .with_children(|indicator| {
                            indicator.spawn((
                                Node {
                                    width: Val::Px(8.0),
                                    height: Val::Px(8.0),
                                    border_radius: BorderRadius::MAX,
                                    ..default()
                                },
                                BackgroundColor(Color::NONE),
                                Surface::rounded_rect_fill(
                                    4.0,
                                    Paint::solid(radio_dot_color(colors, is_selected)),
                                ),
                                RadioDotPart {
                                    owner: button_entity,
                                },
                            ));
                        });

                    button_parent.spawn((
                        ThemedText::new(TextRole::Label),
                        Text::new(option.label),
                        TextFont {
                            font_size: FontSize::Px(16.0),
                            ..default()
                        },
                        TextColor(radio_label_color(colors, is_selected)),
                        RadioLabel,
                        RadioLabelPart {
                            owner: button_entity,
                        },
                    ));
                });
            }
        });

        group
    }
}

pub fn spawn_radio_group(parent: &mut ChildSpawnerCommands, builder: RadioGroupBuilder) -> Entity {
    builder.spawn(parent)
}

fn radio_interaction_system(
    mut button_query: Query<
        (Entity, &Interaction, &RadioButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut groups: Query<&mut RadioGroup>,
    mut events: MessageWriter<RadioChanged>,
) {
    for (entity, interaction, button) in &mut button_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        for mut group in &mut groups {
            if group.id != button.group_id {
                continue;
            }

            if group.selected == button.value {
                break;
            }

            group.selected = button.value.clone();
            events.write(RadioChanged {
                entity,
                group_id: group.id.clone(),
                value: group.selected.clone(),
            });
            break;
        }
    }
}

fn radio_visual_system(
    theme: Res<ThemeResource>,
    button_query: Query<(&RadioButton, &Interaction)>,
    groups: Query<&RadioGroup>,
    mut surface_queries: ParamSet<(
        Query<(&RadioIndicatorPart, &mut Surface), (With<RadioIndicator>, Without<RadioLabel>)>,
        Query<(&RadioDotPart, &mut Surface)>,
    )>,
    mut label_query: Query<(&RadioLabelPart, &mut TextColor), With<RadioLabel>>,
) {
    let colors = theme.current.colors;

    for (part, mut surface) in &mut surface_queries.p0() {
        let Ok((button, interaction)) = button_query.get(part.owner) else {
            continue;
        };

        let is_selected = groups
            .iter()
            .any(|group| group.id == button.group_id && group.selected == button.value);

        surface.fill = Paint::solid(radio_border_color(colors, is_selected, *interaction));
    }

    for (part, mut surface) in &mut surface_queries.p1() {
        let Ok((button, _)) = button_query.get(part.owner) else {
            continue;
        };

        let is_selected = groups
            .iter()
            .any(|group| group.id == button.group_id && group.selected == button.value);

        surface.fill = Paint::solid(radio_dot_color(colors, is_selected));
    }

    for (part, mut label_color) in &mut label_query {
        let Ok((button, _)) = button_query.get(part.owner) else {
            continue;
        };

        let is_selected = groups
            .iter()
            .any(|group| group.id == button.group_id && group.selected == button.value);

        label_color.0 = radio_label_color(colors, is_selected);
    }
}

fn radio_border_color(colors: ThemeColors, selected: bool, interaction: Interaction) -> Color {
    match (selected, interaction) {
        (true, Interaction::None) => colors.primary,
        (true, Interaction::Hovered) => colors.primary_hover,
        (true, Interaction::Pressed) => colors.primary_active,
        (false, Interaction::None) => colors.border,
        (false, Interaction::Hovered) => colors.border_strong,
        (false, Interaction::Pressed) => colors.text_muted,
    }
}

fn radio_dot_color(colors: ThemeColors, selected: bool) -> Color {
    if selected {
        colors.primary
    } else {
        Color::NONE
    }
}

fn radio_label_color(colors: ThemeColors, selected: bool) -> Color {
    if selected {
        colors.text
    } else {
        colors.text_muted
    }
}

// ============================================================
// ACCESSIBILITY
// ============================================================

/// Arrow-key roving focus: moves both keyboard focus and the selected value
/// together, matching the native `radiogroup` keyboard pattern (Tab enters
/// or leaves the group; Up/Down/Left/Right change the selection).
fn radio_roving_nav_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut focus: ResMut<InputFocus>,
    buttons: Query<&RadioButton>,
    child_of_query: Query<&ChildOf>,
    groups: Query<&Children, With<RadioGroup>>,
    mut group_query: Query<&mut RadioGroup>,
    mut events: MessageWriter<RadioChanged>,
) {
    let Some(focused) = focus.get() else {
        return;
    };
    if buttons.get(focused).is_err() {
        return;
    }

    let next_key = keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::ArrowRight);
    let prev_key = keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::ArrowLeft);
    if !next_key && !prev_key {
        return;
    }

    let Ok(child_of) = child_of_query.get(focused) else {
        return;
    };
    let group_entity = child_of.parent();
    let Ok(children) = groups.get(group_entity) else {
        return;
    };

    let ordered: Vec<Entity> = children
        .iter()
        .filter(|entity| buttons.contains(*entity))
        .collect();
    let len = ordered.len();
    if len < 2 {
        return;
    }
    let Some(index) = ordered.iter().position(|entity| *entity == focused) else {
        return;
    };

    let next_index = if next_key {
        (index + 1) % len
    } else {
        (index + len - 1) % len
    };
    let next_entity = ordered[next_index];
    let Ok(next_button) = buttons.get(next_entity) else {
        return;
    };
    let group_id = next_button.group_id.clone();
    let value = next_button.value.clone();

    if let Ok(mut group) = group_query.get_mut(group_entity)
        && group.selected != value
    {
        group.selected = value.clone();
        events.write(RadioChanged {
            entity: next_entity,
            group_id,
            value,
        });
    }

    focus.set(next_entity, FocusCause::Navigated);
}

/// Keeps each button's tab stop and AccessKit toggled state in sync with
/// which value is currently selected (only the selected radio is a tab
/// stop, matching the native roving-tabindex pattern).
fn radio_a11y_sync_system(
    groups: Query<(&RadioGroup, &Children), Changed<RadioGroup>>,
    mut buttons: Query<(&RadioButton, &mut a11y::TabIndex, &mut AccessibilityNode)>,
) {
    for (group, children) in &groups {
        for child in children.iter() {
            let Ok((button, mut tab_index, mut node)) = buttons.get_mut(child) else {
                continue;
            };
            let selected = button.value == group.selected;
            tab_index.0 = if selected { 0 } else { -1 };
            node.0.set_toggled(if selected {
                accesskit::Toggled::True
            } else {
                accesskit::Toggled::False
            });
        }
    }
}

pub struct RadioPlugin;

impl Plugin for RadioPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<RadioChanged>().add_systems(
            Update,
            (
                radio_roving_nav_system,
                radio_interaction_system,
                radio_a11y_sync_system,
                radio_visual_system,
            )
                .chain(),
        );
    }
}
