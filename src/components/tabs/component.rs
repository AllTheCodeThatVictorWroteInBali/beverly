use bevy::a11y::AccessibilityNode;
use bevy::input_focus::InputFocus;
use bevy::prelude::*;

use crate::rendering::{Paint, Surface};
use crate::animation::animation::{
    NodeLeftPercentTransitionTarget,
    SurfaceTransitionTarget,
    TextColorTransitionTarget,
    should_animate_target,
    themed_transition,
};
use crate::primitives::a11y::{self, FocusCause};
use crate::theme::{ThemeColors, ThemeResource, dark_theme};

/// A single tab definition.
#[derive(Clone, Debug)]
pub struct Tab {
    pub id: String,
    pub label: String,
}

impl Tab {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

/// Configuration for a reusable tabs component.
#[derive(Clone, Debug)]
pub struct TabsConfig {
    pub tabs: Vec<Tab>,
    pub active: usize,
}

impl TabsConfig {
    pub fn new(tabs: Vec<Tab>) -> Self {
        Self { tabs, active: 0 }
    }

    pub fn with_active(mut self, active: usize) -> Self {
        self.active = active;
        self
    }

    pub fn active_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active)
    }
}

#[derive(Clone, Debug)]
struct TabTransition {
    from: usize,
    to: usize,
}

impl TabTransition {
    fn new(from: usize, to: usize) -> Self {
        Self { from, to }
    }
}

/// Root component for a tabs instance.
#[derive(Component, Debug)]
pub struct Tabs {
    pub config: TabsConfig,
    transition: Option<TabTransition>,
}

impl Tabs {
    pub fn new(tabs: Vec<Tab>) -> Self {
        Self {
            config: TabsConfig::new(tabs),
            transition: None,
        }
    }

    pub fn active(&self) -> usize {
        self.config.active
    }

    pub fn set_active(&mut self, index: usize) {
        if index >= self.config.tabs.len() {
            return;
        }

        let previous = self.config.active;
        if previous == index {
            self.transition = None;
            return;
        }

        self.config.active = index;
        self.transition = Some(TabTransition::new(previous, index));
    }
}

/// Identifies an individual tab button.
#[derive(Component, Debug, Clone)]
pub struct TabButton {
    pub tabs_entity: Entity,
    pub index: usize,
}

/// Identifies the content zone belonging to a tab.
#[derive(Component, Debug, Clone)]
pub struct TabZone {
    pub tabs_entity: Entity,
    pub index: usize,
}

#[derive(Component)]
struct TabBarShell;

#[derive(Component)]
struct TabContentShell;

/// Event emitted whenever the active tab changes.
#[derive(Message, Debug, Clone)]
pub struct TabChanged {
    pub tabs: Entity,
    pub index: usize,
    pub id: String,
}

/// Plugin for the tabs system.
pub struct TabsPlugin;

impl Plugin for TabsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TabChanged>().add_systems(
            Update,
            (
                tabs_roving_nav_system,
                tab_button_interaction,
                tabs_a11y_sync_system,
                update_tab_visuals,
                animate_tab_content,
            )
                .chain(),
        );
    }
}

/// Spawn a tabs component using a normal Bevy command queue.
pub fn spawn_tabs<F>(commands: &mut Commands, config: TabsConfig, mut build_zone: F) -> Entity
where
    F: FnMut(&mut ChildSpawnerCommands, usize, &Tab),
{
    let colors = dark_theme().colors;

    let mut tabs = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(14.0),
            ..default()
        },
        Tabs {
            config: config.clone(),
            transition: None,
        },
    ));
    let tabs_entity = tabs.id();

    tabs.with_children(|root| {
        root.spawn((
            TabBarShell,
            a11y::tab_list_node("Tabs"),
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(px(1.0)),
                border_radius: BorderRadius::all(px(14.0)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderColor::all(Color::NONE),
            Surface::rounded_rect_fill(
                14.0,
                Paint::solid(colors.surface_elevated.with_alpha(0.76)),
            )
            .uniform_border(1.0, Paint::solid(colors.border.with_alpha(0.45))),
        ))
        .with_children(|bar| {
            for (index, tab) in config.tabs.iter().enumerate() {
                let active = index == config.active;
                bar.spawn((
                    Button,
                    a11y::TabIndex(if active { 0 } else { -1 }),
                    a11y::tab_node(tab.label.clone(), active),
                    Node {
                        padding: UiRect::axes(Val::Px(18.0), Val::Px(10.0)),
                        border: UiRect::all(px(1.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border_radius: BorderRadius::all(Val::Px(14.0)),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    BorderColor::all(Color::NONE),
                    Surface::rounded_rect_fill(
                        14.0,
                        Paint::solid(tab_bg_color(colors, active, false)),
                    )
                    .uniform_border(1.0, Paint::solid(tab_border_color(colors, active, false))),
                    TabButton {
                        tabs_entity: tabs_entity,
                        index,
                    },
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new(tab.label.clone()),
                        TextFont {
                            font_size: FontSize::Px(15.0),
                            ..default()
                        },
                        TextColor(tab_text_color(colors, active, false)),
                    ));
                });
            }
        });

        root.spawn((
            TabContentShell,
            ScrollPosition::default(),
            Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                position_type: PositionType::Relative,
                overflow: Overflow {
                    x: OverflowAxis::Clip,
                    y: OverflowAxis::Scroll,
                },
                padding: UiRect::all(Val::Px(14.0)),
                border: UiRect::all(px(1.0)),
                border_radius: BorderRadius::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderColor::all(Color::NONE),
            Surface::rounded_rect_fill(
                20.0,
                Paint::solid(colors.surface.with_alpha(0.62)),
            )
            .uniform_border(1.0, Paint::solid(colors.border.with_alpha(0.40))),
        ))
        .with_children(|content| {
            for (index, tab) in config.tabs.iter().enumerate() {
                content
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            position_type: PositionType::Absolute,
                            left: if index == config.active {
                                Val::Percent(0.0)
                            } else {
                                Val::Percent(100.0)
                            },
                            top: Val::Px(0.0),
                            display: Display::Flex,
                            ..default()
                        },
                        TabZone {
                            tabs_entity: tabs_entity,
                            index,
                        },
                    ))
                    .with_children(|zone| {
                        build_zone(zone, index, tab);
                    });
            }
        });
    });

    tabs_entity
}

/// Spawn a tabs component with extra content at the right side of the tab bar.
pub fn spawn_tabs_in_with_bar<F, G>(
    parent: &mut ChildSpawnerCommands,
    config: TabsConfig,
    build_bar_right: G,
    mut build_zone: F,
) -> Entity
where
    F: FnMut(&mut ChildSpawnerCommands, usize, &Tab),
    G: FnOnce(&mut ChildSpawnerCommands),
{
    let colors = dark_theme().colors;

    let mut tabs = parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(14.0),
            ..default()
        },
        Tabs {
            config: config.clone(),
            transition: None,
        },
    ));
    let tabs_entity = tabs.id();

    tabs.with_children(|root| {
        root.spawn((
            TabBarShell,
            a11y::tab_list_node("Tabs"),
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(px(1.0)),
                border_radius: BorderRadius::all(px(14.0)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderColor::all(Color::NONE),
            Surface::rounded_rect_fill(
                14.0,
                Paint::solid(colors.surface_elevated.with_alpha(0.76)),
            )
            .uniform_border(1.0, Paint::solid(colors.border.with_alpha(0.45))),
        ))
        .with_children(|bar| {
            for (index, tab) in config.tabs.iter().enumerate() {
                let active = index == config.active;
                bar.spawn((
                    Button,
                    a11y::TabIndex(if active { 0 } else { -1 }),
                    a11y::tab_node(tab.label.clone(), active),
                    Node {
                        padding: UiRect::axes(Val::Px(18.0), Val::Px(10.0)),
                        border: UiRect::all(px(1.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border_radius: BorderRadius::all(Val::Px(14.0)),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    BorderColor::all(Color::NONE),
                    Surface::rounded_rect_fill(
                        14.0,
                        Paint::solid(tab_bg_color(colors, active, false)),
                    )
                    .uniform_border(1.0, Paint::solid(tab_border_color(colors, active, false))),
                    TabButton {
                        tabs_entity: tabs_entity,
                        index,
                    },
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new(tab.label.clone()),
                        TextFont {
                            font_size: FontSize::Px(15.0),
                            ..default()
                        },
                        TextColor(tab_text_color(colors, active, false)),
                    ));
                });
            }

            bar.spawn(Node {
                flex_grow: 1.0,
                min_width: Val::Px(0.0),
                ..default()
            });

            build_bar_right(bar);
        });

        root.spawn((
            TabContentShell,
            ScrollPosition::default(),
            Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                position_type: PositionType::Relative,
                overflow: Overflow {
                    x: OverflowAxis::Clip,
                    y: OverflowAxis::Scroll,
                },
                padding: UiRect::all(Val::Px(14.0)),
                border: UiRect::all(px(1.0)),
                border_radius: BorderRadius::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderColor::all(Color::NONE),
            Surface::rounded_rect_fill(
                20.0,
                Paint::solid(colors.surface.with_alpha(0.62)),
            )
            .uniform_border(1.0, Paint::solid(colors.border.with_alpha(0.40))),
        ))
        .with_children(|content| {
            for (index, tab) in config.tabs.iter().enumerate() {
                content
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            position_type: PositionType::Absolute,
                            left: if index == config.active {
                                Val::Percent(0.0)
                            } else {
                                Val::Percent(100.0)
                            },
                            top: Val::Px(0.0),
                            display: Display::Flex,
                            ..default()
                        },
                        TabZone {
                            tabs_entity: tabs_entity,
                            index,
                        },
                    ))
                    .with_children(|zone| {
                        build_zone(zone, index, tab);
                    });
            }
        });
    });

    tabs_entity
}

/// Spawn a tabs component anchored to a child-spawner context.
pub fn spawn_tabs_in<F>(
    parent: &mut ChildSpawnerCommands,
    config: TabsConfig,
    mut build_zone: F,
) -> Entity
where
    F: FnMut(&mut ChildSpawnerCommands, usize, &Tab),
{
    let colors = dark_theme().colors;

    let mut tabs = parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(14.0),
            ..default()
        },
        Tabs {
            config: config.clone(),
            transition: None,
        },
    ));
    let tabs_entity = tabs.id();

    tabs.with_children(|root| {
        root.spawn((
            TabBarShell,
            a11y::tab_list_node("Tabs"),
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(px(1.0)),
                border_radius: BorderRadius::all(px(14.0)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(colors.surface_elevated.with_alpha(0.76)),
            BorderColor::all(colors.border.with_alpha(0.45)),
        ))
        .with_children(|bar| {
            for (index, tab) in config.tabs.iter().enumerate() {
                let active = index == config.active;
                bar.spawn((
                    Button,
                    a11y::TabIndex(if active { 0 } else { -1 }),
                    a11y::tab_node(tab.label.clone(), active),
                    Node {
                        padding: UiRect::axes(Val::Px(18.0), Val::Px(10.0)),
                        border: UiRect::all(px(1.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border_radius: BorderRadius::all(Val::Px(14.0)),
                        ..default()
                    },
                    BackgroundColor(tab_bg_color(colors, active, false)),
                    BorderColor::all(tab_border_color(colors, active, false)),
                    TabButton {
                        tabs_entity: tabs_entity,
                        index,
                    },
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new(tab.label.clone()),
                        TextFont {
                            font_size: FontSize::Px(15.0),
                            ..default()
                        },
                        TextColor(tab_text_color(colors, active, false)),
                    ));
                });
            }
        });

        root.spawn((
            TabContentShell,
            ScrollPosition::default(),
            Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                position_type: PositionType::Relative,
                overflow: Overflow {
                    x: OverflowAxis::Clip,
                    y: OverflowAxis::Scroll,
                },
                padding: UiRect::all(Val::Px(14.0)),
                border: UiRect::all(px(1.0)),
                border_radius: BorderRadius::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(colors.surface.with_alpha(0.62)),
            BorderColor::all(colors.border.with_alpha(0.40)),
        ))
        .with_children(|content| {
            for (index, tab) in config.tabs.iter().enumerate() {
                content
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            position_type: PositionType::Absolute,
                            left: if index == config.active {
                                Val::Percent(0.0)
                            } else {
                                Val::Percent(100.0)
                            },
                            top: Val::Px(0.0),
                            display: Display::Flex,
                            ..default()
                        },
                        TabZone {
                            tabs_entity: tabs_entity,
                            index,
                        },
                    ))
                    .with_children(|zone| {
                        build_zone(zone, index, tab);
                    });
            }
        });
    });

    tabs_entity
}

/// Arrow-key roving focus for the tab list: Left/Up and Right/Down move
/// both keyboard focus and the active tab together, matching the ARIA
/// `tablist` keyboard pattern (Tab enters/leaves the whole list once).
fn tabs_roving_nav_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut focus: ResMut<InputFocus>,
    tab_buttons: Query<(Entity, &TabButton)>,
    mut tabs_query: Query<&mut Tabs>,
    mut changed: MessageWriter<TabChanged>,
) {
    let Some(focused) = focus.get() else {
        return;
    };
    let Ok((_, focused_button)) = tab_buttons.get(focused) else {
        return;
    };

    let next_key = keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::ArrowDown);
    let prev_key = keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::ArrowUp);
    if !next_key && !prev_key {
        return;
    }

    let tabs_entity = focused_button.tabs_entity;
    let mut siblings: Vec<(Entity, usize)> = tab_buttons
        .iter()
        .filter(|(_, button)| button.tabs_entity == tabs_entity)
        .map(|(entity, button)| (entity, button.index))
        .collect();
    siblings.sort_by_key(|(_, index)| *index);

    let len = siblings.len();
    if len < 2 {
        return;
    }
    let Some(position) = siblings.iter().position(|(entity, _)| *entity == focused) else {
        return;
    };

    let next_position = if next_key {
        (position + 1) % len
    } else {
        (position + len - 1) % len
    };
    let (next_entity, next_index) = siblings[next_position];

    if let Ok(mut tabs) = tabs_query.get_mut(tabs_entity)
        && tabs.config.active != next_index
    {
        tabs.set_active(next_index);
        if let Some(tab) = tabs.config.active_tab() {
            changed.write(TabChanged {
                tabs: tabs_entity,
                index: next_index,
                id: tab.id.clone(),
            });
        }
    }

    focus.set(next_entity, FocusCause::Navigated);
}

/// Keeps each tab button's tab stop and AccessKit selected state in sync
/// with the active tab (only the active tab is a tab stop).
fn tabs_a11y_sync_system(
    tabs_query: Query<(Entity, &Tabs), Changed<Tabs>>,
    mut buttons: Query<(&TabButton, &mut a11y::TabIndex, &mut AccessibilityNode)>,
) {
    for (tabs_entity, tabs) in &tabs_query {
        for (button, mut tab_index, mut node) in &mut buttons {
            if button.tabs_entity != tabs_entity {
                continue;
            }
            let active = button.index == tabs.config.active;
            tab_index.0 = if active { 0 } else { -1 };
            node.0.set_selected(active);
        }
    }
}

/// Handle tab clicks.
fn tab_button_interaction(
    mut interaction_query: Query<(&Interaction, &TabButton), (Changed<Interaction>, With<Button>)>,
    mut tabs_query: Query<&mut Tabs>,
    mut changed: MessageWriter<TabChanged>,
) {
    for (interaction, button) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Ok(mut tabs) = tabs_query.get_mut(button.tabs_entity) else {
            continue;
        };

        if tabs.config.active == button.index {
            continue;
        }

        tabs.set_active(button.index);

        if let Some(tab) = tabs.config.active_tab() {
            changed.write(TabChanged {
                tabs: button.tabs_entity,
                index: button.index,
                id: tab.id.clone(),
            });
        }
    }
}

/// Update tab button appearance.
fn update_tab_visuals(
    mut commands: Commands,
    theme: Res<ThemeResource>,
    tabs_query: Query<(Entity, &Tabs)>,
    mut surface_queries: ParamSet<(
        Query<(
            Entity,
            &TabButton,
            &Interaction,
            &mut Surface,
            &Children,
            Option<&SurfaceTransitionTarget>,
        )>,
        Query<
            (Entity, &mut Surface, Option<&SurfaceTransitionTarget>),
            (
                With<TabBarShell>,
                Without<TabContentShell>,
                Without<TabButton>,
            ),
        >,
        Query<
            (Entity, &mut Surface, Option<&SurfaceTransitionTarget>),
            (
                With<TabContentShell>,
                Without<TabBarShell>,
                Without<TabButton>,
            ),
        >,
    )>,
    mut texts: Query<(Entity, &mut TextColor, Option<&TextColorTransitionTarget>)>,
) {
    let colors = theme.current.colors;
    let transition = themed_transition(&theme, |tokens| tokens.interaction);

    for (entity, surface, current_target) in &mut surface_queries.p1() {
        let mut target = (*surface).clone();
        target.fill = Paint::solid(colors.surface_elevated.with_alpha(0.76));
        if let Some(border) = target.border.as_mut() {
            border.paint = Paint::solid(colors.border.with_alpha(0.45));
        }

        if should_animate_target(&current_target.map(|value| value.target.clone()), &target) {
            commands
                .entity(entity)
                .insert(SurfaceTransitionTarget::new(target, transition));
        }
    }

    for (entity, surface, current_target) in &mut surface_queries.p2() {
        let mut target = (*surface).clone();
        target.fill = Paint::solid(colors.surface.with_alpha(0.62));
        if let Some(border) = target.border.as_mut() {
            border.paint = Paint::solid(colors.border.with_alpha(0.40));
        }

        if should_animate_target(&current_target.map(|value| value.target.clone()), &target) {
            commands
                .entity(entity)
                .insert(SurfaceTransitionTarget::new(target, transition));
        }
    }

    for (tabs_entity, tabs) in &tabs_query {
        for (entity, button, interaction, surface, children, current_target) in &mut surface_queries.p0() {
            if button.tabs_entity != tabs_entity {
                continue;
            }

            let active = button.index == tabs.config.active;
            let hovered =
                *interaction == Interaction::Hovered || *interaction == Interaction::Pressed;

            let bg = tab_bg_color(colors, active, hovered);
            let border = tab_border_color(colors, active, hovered);
            let text = tab_text_color(colors, active, hovered);

            let mut target = (*surface).clone();
            target.fill = Paint::solid(bg);
            if let Some(surface_border) = target.border.as_mut() {
                surface_border.paint = Paint::solid(border);
            }

            if should_animate_target(&current_target.map(|value| value.target.clone()), &target) {
                commands
                    .entity(entity)
                    .insert(SurfaceTransitionTarget::new(target, transition));
            }

            for child in children.iter() {
                if let Ok((text_entity, _, current_text_target)) = texts.get_mut(child) {
                    if should_animate_target(&current_text_target.map(|value| value.target), &text) {
                        commands
                            .entity(text_entity)
                            .insert(TextColorTransitionTarget::new(text, transition));
                    }
                }
            }
        }
    }
}

fn tab_bg_color(colors: ThemeColors, active: bool, hovered: bool) -> Color {
    if active {
        colors.primary.with_alpha(0.98)
    } else if hovered {
        colors.secondary.with_alpha(0.78)
    } else {
        colors.surface.with_alpha(0.62)
    }
}

fn tab_border_color(colors: ThemeColors, active: bool, hovered: bool) -> Color {
    if active {
        colors.border_strong.with_alpha(0.76)
    } else if hovered {
        colors.border.with_alpha(0.58)
    } else {
        colors.border.with_alpha(0.38)
    }
}

fn tab_text_color(colors: ThemeColors, active: bool, hovered: bool) -> Color {
    if active {
        Color::WHITE
    } else if hovered {
        colors.text
    } else {
        colors.text_muted
    }
}

/// Animate content panels as the active tab changes.
fn animate_tab_content(
    mut commands: Commands,
    theme: Res<ThemeResource>,
    mut tabs_query: Query<(Entity, &mut Tabs)>,
    zones: Query<(Entity, &TabZone, Option<&NodeLeftPercentTransitionTarget>)>,
) {
    let transition_spec = themed_transition(&theme, |tokens| tokens.interaction);

    for (tabs_entity, mut tabs) in &mut tabs_query {
        let transition = tabs.transition.take();
        let active = tabs.config.active;

        for (entity, zone, current_target) in &zones {
            if zone.tabs_entity != tabs_entity {
                continue;
            }

            let target_percent = if let Some(transition) = transition.as_ref() {
                let direction = if transition.to > transition.from {
                    1.0
                } else {
                    -1.0
                };

                if zone.index == transition.to {
                    0.0
                } else if zone.index == transition.from {
                    -100.0 * direction
                } else {
                    100.0
                }
            } else if zone.index == active {
                0.0
            } else {
                100.0
            };

            if should_animate_target(&current_target.map(|value| value.target_percent), &target_percent) {
                commands.entity(entity).insert(
                    NodeLeftPercentTransitionTarget::new(target_percent, transition_spec),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_active_index_is_preserved() {
        let config = TabsConfig::new(vec![Tab::new("one", "One")]).with_active(1);
        assert_eq!(config.active, 1);
    }
}
