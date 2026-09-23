use bevy::prelude::*;

use crate::rendering::{Paint, Surface};

// ============================================================
// Button Group
// ============================================================

#[derive(Component)]
pub struct ButtonGroup {
    pub orientation: ButtonGroupOrientation,
    pub selection: ButtonGroupSelection,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonGroupOrientation {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonGroupSelection {
    None,
    Single,
    Multiple,
}

// ============================================================
// Button Group Item
// ============================================================

#[derive(Component)]
pub struct ButtonGroupItem {
    pub group: Entity,
    pub id: String,
    pub selected: bool,
    pub disabled: bool,
}

// ============================================================
// Events
// ============================================================

#[derive(Message, Clone)]
pub struct ButtonGroupEvent {
    pub group: Entity,
    pub button: Entity,
    pub id: String,
    pub selected: bool,
}

// ============================================================
// Configuration
// ============================================================

pub struct ButtonGroupConfig {
    pub orientation: ButtonGroupOrientation,
    pub selection: ButtonGroupSelection,
    pub spacing: f32,
}

impl Default for ButtonGroupConfig {
    fn default() -> Self {
        Self {
            orientation: ButtonGroupOrientation::Horizontal,
            selection: ButtonGroupSelection::Single,
            spacing: 4.0,
        }
    }
}

// ============================================================
// Button Definition
// ============================================================

pub struct ButtonGroupButton {
    pub id: String,
    pub label: String,
    pub selected: bool,
    pub disabled: bool,
}

impl ButtonGroupButton {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            selected: false,
            disabled: false,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

// ============================================================
// Plugin
// ============================================================

pub struct ButtonGroupPlugin;

impl Plugin for ButtonGroupPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ButtonGroupEvent>()
            .add_systems(Update, button_group_interaction);
    }
}

// ============================================================
// Builder
// ============================================================

pub fn spawn_button_group(
    commands: &mut Commands,
    config: ButtonGroupConfig,
    buttons: Vec<ButtonGroupButton>,
) -> Entity {
    let direction = match config.orientation {
        ButtonGroupOrientation::Horizontal => FlexDirection::Row,
        ButtonGroupOrientation::Vertical => FlexDirection::Column,
    };

    let group = commands
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: direction,
                column_gap: if matches!(
                    config.orientation,
                    ButtonGroupOrientation::Vertical
                ) {
                    Val::Px(config.spacing)
                } else {
                    Val::Px(0.0)
                },
                row_gap: if matches!(
                    config.orientation,
                    ButtonGroupOrientation::Horizontal
                ) {
                    Val::Px(config.spacing)
                } else {
                    Val::Px(0.0)
                },
                ..default()
            },
            ButtonGroup {
                orientation: config.orientation,
                selection: config.selection,
            },
        ))
        .id();

    for button in buttons {
        let child = commands
            .spawn((
                Button,
                Node {
                    padding: UiRect::axes(
                        Val::Px(12.0),
                        Val::Px(8.0),
                    ),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                Surface::rounded_rect_fill(
                    8.0,
                    Paint::solid(if button.selected {
                        Color::srgb(0.25, 0.25, 0.25)
                    } else {
                        Color::srgb(0.12, 0.12, 0.12)
                    }),
                ),
                ButtonGroupItem {
                    group,
                    id: button.id,
                    selected: button.selected,
                    disabled: button.disabled,
                },
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new(button.label),
                    TextColor(Color::WHITE),
                ));
            })
            .id();

        commands.entity(group).add_child(child);
    }

    group
}

// ============================================================
// Interaction
// ============================================================

fn button_group_interaction(
    interactions: Query<(Entity, &Interaction), (Changed<Interaction>, With<Button>)>,

    groups: Query<&ButtonGroup>,

    mut buttons: Query<
        (
            Entity,
            &mut ButtonGroupItem,
            &mut Surface,
        ),
        With<Button>,
    >,

    mut events: MessageWriter<ButtonGroupEvent>,
) {
    for (button_entity, interaction) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Ok((_, item, _)) = buttons.get(button_entity) else {
            continue;
        };
        if item.disabled {
            continue;
        }
        let item_group = item.group;
        let item_id = item.id.clone();

        let Ok(group) = groups.get(item_group) else {
            continue;
        };

        match group.selection {
            ButtonGroupSelection::None => {
                events.write(ButtonGroupEvent {
                    group: item_group,
                    button: button_entity,
                    id: item_id,
                    selected: false,
                });
            }

            ButtonGroupSelection::Single => {
                for (other_entity, mut other, mut surface) in &mut buttons {
                    if other.group != item_group {
                        continue;
                    }

                    other.selected = other_entity == button_entity;

                    surface.fill = Paint::solid(if other.selected {
                        Color::srgb(0.25, 0.25, 0.25)
                    } else {
                        Color::srgb(0.12, 0.12, 0.12)
                    });
                }

                if let Ok((_, mut item, _)) = buttons.get_mut(button_entity) {
                    item.selected = true;
                }

                events.write(ButtonGroupEvent {
                    group: item_group,
                    button: button_entity,
                    id: item_id,
                    selected: true,
                });
            }

            ButtonGroupSelection::Multiple => {
                let Ok((_, mut item, _)) = buttons.get_mut(button_entity) else {
                    continue;
                };
                item.selected = !item.selected;
                let selected = item.selected;

                events.write(ButtonGroupEvent {
                    group: item_group,
                    button: button_entity,
                    id: item_id,
                    selected,
                });
            }
        }
    }
}