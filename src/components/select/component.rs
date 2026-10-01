use bevy::prelude::*;
use std::fmt::Display as FmtDisplay;
use std::marker::PhantomData;

use crate::rendering::{Paint, Surface};
use crate::primitives::a11y;
use crate::primitives::semantic::{SemanticNode, SemanticRole};
use crate::icons::{Icon, IconNode};
use crate::components::text::{TextRole, ThemedText};
use crate::theme::{ThemeResource, dark_theme};

// ============================================================
// ACCESSIBLE LABEL ABSTRACTION
// ============================================================

/// Produces the screen-reader-facing label for one `Select<T>` option/value.
///
/// This keeps the accessibility projection generic over `T` instead of
/// hard-coding it to `Select<String>`: any `T: Display` gets this for free,
/// so callers writing `Select<Region>` or `Select<DeploymentEnvironment>`
/// remain fully accessible without extra boilerplate.
pub trait SelectSemanticValue {
    fn accessible_label(&self) -> String;
}

impl<T> SelectSemanticValue for T
where
    T: FmtDisplay,
{
    fn accessible_label(&self) -> String {
        self.to_string()
    }
}

// ============================================================
// SELECT COMPONENT
// ============================================================

#[derive(Component, Clone)]
pub struct Select<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub options: Vec<T>,
    pub selected: Option<usize>,
    pub placeholder: String,
    pub disabled: bool,
    pub is_open: bool,
    pub width: Val,
    _marker: PhantomData<T>,
}

impl<T> Select<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn new(options: Vec<T>) -> Self {
        Self {
            options,
            selected: None,
            placeholder: "Select an option...".to_string(),
            disabled: false,
            is_open: false,
            width: Val::Px(240.0),
            _marker: PhantomData,
        }
    }

    pub fn placeholder(mut self, value: impl Into<String>) -> Self {
        self.placeholder = value.into();
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        if index < self.options.len() {
            self.selected = Some(index);
        }
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn width(mut self, width: Val) -> Self {
        self.width = width;
        self
    }

    pub fn selected_value(&self) -> Option<&T> {
        self.selected.and_then(|index| self.options.get(index))
    }

    pub fn select(&mut self, index: usize) {
        if index < self.options.len() {
            self.selected = Some(index);
        }
    }

    pub fn clear(&mut self) {
        self.selected = None;
    }
}

// ============================================================
// EVENT
// ============================================================

#[derive(Message, Debug, Clone)]
pub struct SelectChanged<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub entity: Entity,
    pub index: usize,
    pub value: T,
}

impl<T> SelectChanged<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn new(entity: Entity, index: usize, value: T) -> Self {
        Self {
            entity,
            index,
            value,
        }
    }
}

// ============================================================
// INTERNAL COMPONENTS
// ============================================================

#[derive(Component)]
pub struct SelectRoot;

#[derive(Component, Clone, Copy)]
struct SelectButton {
    owner: Entity,
}

#[derive(Component, Clone, Copy)]
struct SelectLabel {
    owner: Entity,
}

#[derive(Component, Clone, Copy)]
struct SelectArrow {
    owner: Entity,
}

#[derive(Component, Clone, Copy)]
struct SelectDropdown {
    owner: Entity,
}

#[derive(Component)]
struct SelectOption {
    pub parent: Entity,
    pub index: usize,
}

// ============================================================
// PLUGIN
// ============================================================

/// Adds `Select<T>` input handling, visuals, and accessibility sync for one
/// concrete option type `T`.
///
/// `Select<T>` itself is fully generic, but Bevy systems must be registered
/// per concrete type, so this plugin must be added once for every `T` used
/// with `Select`/`spawn_select` in the app (`BeverlyPlugin` only registers
/// `SelectPlugin::<String>` by default).
pub struct SelectPlugin<T = String>(PhantomData<fn() -> T>);

impl<T> Default for SelectPlugin<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T> Plugin for SelectPlugin<T>
where
    T: SelectSemanticValue + Clone + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<crate::primitives::semantic::SemanticAccessibilityPlugin>() {
            app.add_plugins(crate::primitives::semantic::SemanticAccessibilityPlugin);
        }
        app.add_message::<SelectChanged<T>>()
            .add_systems(Update, select_input_system::<T>)
            .add_systems(PostUpdate, (select_visual_system::<T>, select_a11y_system::<T>));
    }
}

// ============================================================
// SPAWN
// ============================================================

pub fn spawn_select<T>(
    parent: &mut ChildSpawnerCommands,
    select: Select<T>,
    font: Handle<Font>,
) -> Entity
where
    T: Clone + FmtDisplay + Send + Sync + 'static,
{
    let option_labels: Vec<String> = select
        .options
        .iter()
        .map(|option| option.to_string())
        .collect();
    let colors = dark_theme().colors;
    let placeholder = select.placeholder.clone();
    let disabled = select.disabled;

    let mut root = parent.spawn((
        Node {
            width: select.width,
            flex_direction: FlexDirection::Column,
            position_type: PositionType::Relative,
            ..default()
        },
        SelectRoot,
        select,
    ));

    let root_id = root.id();

    root.with_children(|parent| {
        parent
            .spawn((
                Button,
                SelectButton { owner: root_id },
                a11y::TabIndex(if disabled { -1 } else { 0 }),
                SemanticNode::new(SemanticRole::ComboBox).label(placeholder.clone()),
                Node {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(40.0),
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    border_radius: BorderRadius::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                Surface::rounded_rect_fill(10.0, Paint::solid(colors.surface)),
            ))
            .with_children(|button| {
                button.spawn((
                    SelectLabel { owner: root_id },
                    ThemedText::new(TextRole::Body),
                    Text::new(""),
                    TextFont {
                        font: FontSource::Handle(font.clone()),
                        font_size: FontSize::Px(16.0),
                        ..default()
                    },
                    TextColor(colors.text),
                ));

                button.spawn((
                    SelectArrow { owner: root_id },
                    IconNode::new(Icon::feather("chevron-down"))
                        .size(18.0)
                        .color(colors.text_muted),
                    Node {
                        width: px(18.0),
                        height: px(18.0),
                        ..default()
                    },
                ));
            });

        parent
            .spawn((
                SelectDropdown { owner: root_id },
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    display: Display::None,
                    row_gap: Val::Px(4.0),
                    margin: UiRect::top(Val::Px(4.0)),
                    padding: UiRect::all(Val::Px(4.0)),
                    border_radius: BorderRadius::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                Surface::rounded_rect_fill(10.0, Paint::solid(colors.surface_elevated)),
                ZIndex(2),
            ))
            .with_children(|menu| {
                for (index, option) in option_labels.iter().enumerate() {
                    menu.spawn((
                        Button,
                        SelectOption {
                            parent: root_id,
                            index,
                        },
                        {
                            let mut semantic = SemanticNode::new(SemanticRole::ListBoxOption)
                                .label(option.clone());
                            semantic.state.selected = false;
                            semantic
                        },
                        Node {
                            width: Val::Percent(100.0),
                            min_height: Val::Px(38.0),
                            padding: UiRect::horizontal(Val::Px(12.0)),
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                        Surface::rounded_rect_fill(8.0, Paint::solid(colors.surface)),
                    ))
                    .with_children(|option_node| {
                        option_node.spawn((
                            ThemedText::new(TextRole::Label),
                            Text::new(option.clone()),
                            TextFont {
                                font: FontSource::Handle(font.clone()),
                                font_size: FontSize::Px(15.0),
                                ..default()
                            },
                            TextColor(colors.text),
                        ));
                    });
                }
            });
    });

    root_id
}

// ============================================================
// INPUT
// ============================================================

fn select_input_system<T>(
    mut interaction_query: Query<
        (&Interaction, &SelectButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut select_query: Query<&mut Select<T>>,
    option_query: Query<(&Interaction, &SelectOption), (Changed<Interaction>, With<Button>)>,
    mut dropdowns: Query<(&SelectDropdown, &mut Node)>,
    mut events: MessageWriter<SelectChanged<T>>,
    keyboard: Res<ButtonInput<KeyCode>>,
)
where
    T: Clone + Send + Sync + 'static,
{
    for (interaction, button) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Ok(mut select) = select_query.get_mut(button.owner) else {
            continue;
        };

        if select.disabled {
            continue;
        }

        select.is_open = !select.is_open;

        for (dropdown, mut node) in &mut dropdowns {
            if dropdown.owner != button.owner {
                continue;
            }
            node.display = if select.is_open {
                Display::Flex
            } else {
                Display::None
            };
        }
    }

    for (interaction, option) in &option_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Ok(mut select) = select_query.get_mut(option.parent) else {
            continue;
        };

        if select.disabled || option.index >= select.options.len() {
            continue;
        }

        select.selected = Some(option.index);
        select.is_open = false;

        for (dropdown, mut node) in &mut dropdowns {
            if dropdown.owner == option.parent {
                node.display = Display::None;
            }
        }

        events.write(SelectChanged::new(
            option.parent,
            option.index,
            select.options[option.index].clone(),
        ));
    }

    for mut select in &mut select_query {
        if select.disabled || !select.is_open {
            continue;
        }

        if keyboard.just_pressed(KeyCode::ArrowDown) {
            let next = match select.selected {
                Some(index) => (index + 1).min(select.options.len().saturating_sub(1)),
                None => 0,
            };
            select.selected = Some(next);
        }

        if keyboard.just_pressed(KeyCode::ArrowUp) {
            let previous = match select.selected {
                Some(index) => index.saturating_sub(1),
                None => 0,
            };
            select.selected = Some(previous);
        }

        if keyboard.just_pressed(KeyCode::Escape) {
            select.is_open = false;
        }
    }
}

// ============================================================
// ACCESSIBILITY
// ============================================================

/// Keeps the trigger's expanded/label state and each option's selected
/// state in sync with the `Select` component for screen readers.
fn select_a11y_system<T>(
    select_query: Query<(Entity, &Select<T>)>,
    mut button_query: Query<(&SelectButton, &mut SemanticNode), Without<SelectOption>>,
    mut option_query: Query<(&SelectOption, &mut SemanticNode), Without<SelectButton>>,
)
where
    T: SelectSemanticValue + Clone + Send + Sync + 'static,
{
    for (entity, select) in &select_query {
        let label_text = select
            .selected
            .and_then(|index| select.options.get(index))
            .map(SelectSemanticValue::accessible_label)
            .unwrap_or_else(|| select.placeholder.clone());

        for (button, mut node) in &mut button_query {
            if button.owner != entity {
                continue;
            }
            if node.label.as_deref() != Some(label_text.as_str()) {
                node.label = Some(label_text.clone());
            }
            if node.state.expanded != Some(select.is_open) {
                node.state.expanded = Some(select.is_open);
            }
            if node.state.disabled != select.disabled {
                node.state.disabled = select.disabled;
            }
        }

        for (option, mut node) in &mut option_query {
            if option.parent != entity {
                continue;
            }
            let selected = Some(option.index) == select.selected;
            if node.state.selected != selected {
                node.state.selected = selected;
            }
        }
    }
}

// ============================================================
// VISUAL STATE
// ============================================================

fn select_visual_system<T>(
    theme: Res<ThemeResource>,
    select_query: Query<(Entity, &Select<T>)>,
    mut surface_queries: ParamSet<(
        Query<
            (&SelectButton, &mut Surface),
            (Without<SelectDropdown>, Without<SelectOption>),
        >,
        Query<
            (&SelectDropdown, &mut Surface),
            (Without<SelectButton>, Without<SelectOption>),
        >,
    )>,
    mut label_query: Query<(&SelectLabel, &mut Text, &mut TextColor)>,
    mut arrow_query: Query<(&SelectArrow, &mut IconNode)>,
    mut option_query: Query<(&SelectOption, &Interaction, &mut Surface, &Children)>,
)
where
    T: SelectSemanticValue + Clone + Send + Sync + 'static,
{
    let colors = theme.current.colors;

    for (entity, select) in &select_query {
        for (button, mut surface) in &mut surface_queries.p0() {
            if button.owner == entity {
                surface.fill = Paint::solid(colors.surface);
            }
        }

        for (dropdown, mut surface) in &mut surface_queries.p1() {
            if dropdown.owner == entity {
                surface.fill = Paint::solid(colors.surface_elevated);
            }
        }

        let label_text = select
            .selected
            .and_then(|index| select.options.get(index))
            .map(SelectSemanticValue::accessible_label)
            .unwrap_or_else(|| select.placeholder.clone());

        for (label, mut text, mut text_color) in &mut label_query {
            if label.owner == entity {
                *text = Text::new(label_text.clone());
                *text_color = TextColor(if select.selected.is_some() {
                    colors.text
                } else {
                    colors.text_muted
                });
            }
        }

        for (arrow, mut icon) in &mut arrow_query {
            if arrow.owner == entity {
                icon.icon = if select.is_open {
                    Icon::feather("chevron-up")
                } else {
                    Icon::feather("chevron-down")
                };
                icon.color = if select.is_open {
                    colors.text
                } else {
                    colors.text_muted
                };
            }
        }

        for (option, interaction, mut surface, children) in &mut option_query {
            if option.parent != entity {
                continue;
            }

            let selected = select.selected == Some(option.index);
            let hovered =
                *interaction == Interaction::Hovered || *interaction == Interaction::Pressed;

            surface.fill = Paint::solid(if selected {
                colors.primary
            } else if hovered {
                colors.secondary
            } else {
                colors.surface
            });

            let _ = children;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Region {
        UsEast,
        UsWest,
    }

    impl FmtDisplay for Region {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(match self {
                Region::UsEast => "US East",
                Region::UsWest => "US West",
            })
        }
    }

    fn spawn_region_select(app: &mut App) -> (Entity, Entity, Entity) {
        let root = app
            .world_mut()
            .spawn(Select::new(vec![Region::UsEast, Region::UsWest]))
            .id();
        let button = app
            .world_mut()
            .spawn((
                Button,
                SelectButton { owner: root },
                SemanticNode::new(SemanticRole::ComboBox).label("Select a region..."),
            ))
            .id();
        let option = app
            .world_mut()
            .spawn((
                Button,
                SelectOption { parent: root, index: 1 },
                {
                    let mut semantic = SemanticNode::new(SemanticRole::ListBoxOption)
                        .label(Region::UsWest.accessible_label());
                    semantic.state.selected = false;
                    semantic
                },
            ))
            .id();
        (root, button, option)
    }

    /// Regression test for the P0 bug where `select_input_system`/
    /// `select_a11y_system` were hard-coded to `Select<String>`: a
    /// `Select<Region>` would silently never react to input or update its
    /// accessibility tree at all.
    #[test]
    fn select_input_and_a11y_systems_work_for_non_string_option_types() {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>()
            .add_message::<SelectChanged<Region>>()
            .add_systems(Update, select_input_system::<Region>)
            .add_systems(PostUpdate, select_a11y_system::<Region>);
        let (root, button, option) = spawn_region_select(&mut app);
        app.update();

        // Opening the trigger must flip `is_open` and expose it via `aria-expanded`.
        app.world_mut().entity_mut(button).insert(Interaction::Pressed);
        app.update();
        assert!(app.world().get::<Select<Region>>(root).unwrap().is_open);
        assert_eq!(
            app.world().get::<SemanticNode>(button).unwrap().state.expanded,
            Some(true),
        );

        // Picking an option must select the real `Region` value (not a
        // re-parsed string), emit a typed `SelectChanged<Region>`, and mark
        // the option's SemanticNode as selected.
        app.world_mut().entity_mut(option).insert(Interaction::Pressed);
        app.update();
        assert_eq!(app.world().get::<Select<Region>>(root).unwrap().selected, Some(1));

        let changes: Vec<_> = app
            .world_mut()
            .resource_mut::<Messages<SelectChanged<Region>>>()
            .drain()
            .collect();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].index, 1);
        assert_eq!(changes[0].value, Region::UsWest);

        app.world_mut().entity_mut(option).insert(Interaction::None);
        app.update();
        assert!(app.world().get::<SemanticNode>(option).unwrap().state.selected);
        assert_eq!(
            app.world().get::<SemanticNode>(button).unwrap().label.as_deref(),
            Some("US West"),
        );
    }
}
