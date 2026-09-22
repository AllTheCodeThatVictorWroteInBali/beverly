use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::components::text::{TextRole, ThemedText};
use crate::theme::ThemeResource;
use crate::rendering::prelude::*;

#[derive(Component, Clone, Copy)]
pub struct Card {
    themed: bool,
    base_background: Color,
}

#[derive(Component, Clone, Copy)]
struct CardResizeHandle {
    owner: Entity,
}

#[derive(Component)]
struct CardResizeGlyph;

#[derive(Component, Clone, Copy)]
struct CardDragSurface {
    owner: Entity,
}

#[derive(Component, Clone)]
pub struct CardResponsiveContainer {
    pub layout_id: String,
    pub min_neighbor_scale: f32,
}

impl Default for CardResponsiveContainer {
    fn default() -> Self {
        Self::new("default-dashboard")
    }
}

impl CardResponsiveContainer {
    pub fn new(layout_id: impl Into<String>) -> Self {
        Self {
            layout_id: layout_id.into(),
            min_neighbor_scale: 0.72,
        }
    }
}

#[derive(Component, Clone)]
struct CardPersistentId(String);

#[derive(Resource, Default)]
struct ActiveCardResize(Option<CardResizeState>);

#[derive(Resource, Default)]
struct ActiveCardDrag(Option<CardDragState>);

#[derive(Clone, Copy)]
struct CardResizeState {
    card: Entity,
    start_cursor: Vec2,
    start_width: f32,
    start_height: f32,
}

#[derive(Clone, Copy)]
struct CardDragState {
    card: Entity,
    parent: Entity,
}

#[derive(Resource)]
struct CardLayoutMemory {
    by_container: HashMap<String, PersistedContainerLayout>,
    dirty: bool,
    hydrated: bool,
}

impl Default for CardLayoutMemory {
    fn default() -> Self {
        Self {
            by_container: HashMap::new(),
            dirty: false,
            hydrated: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct PersistedLayoutFile {
    containers: HashMap<String, PersistedContainerLayout>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct PersistedContainerLayout {
    order: Vec<String>,
    sizes: Vec<PersistedCardSize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PersistedCardSize {
    id: String,
    width: f32,
    height: f32,
}

#[derive(Component, Clone, Copy)]
struct CardSizeModel {
    base_width: f32,
    base_height: f32,
    target_width: f32,
    target_height: f32,
}

impl CardSizeModel {
    fn new(width: f32, height: f32) -> Self {
        Self {
            base_width: width,
            base_height: height,
            target_width: width,
            target_height: height,
        }
    }
}

const CARD_MIN_WIDTH: f32 = 140.0;
const CARD_MIN_HEIGHT: f32 = 96.0;
const DASHBOARD_LAYOUT_FILE: &str = "dashboard_layout.json";
#[allow(dead_code)]
const CARD_BG_DEFAULT: Color = Color::srgb_u8(255, 255, 255);
const CARD_BORDER_DEFAULT: Color = Color::srgb_u8(226, 232, 240);

#[derive(Clone, Copy)]
pub struct CardStyle {
    pub background: Color,
    pub border_radius: f32,
    pub padding: f32,
    pub width: f32,
    pub height: f32,
    pub themed: bool,
}

impl Default for CardStyle {
    fn default() -> Self {
        Self {
            background: Color::srgb(0.12, 0.12, 0.15),
            border_radius: 12.0,
            padding: 20.0,
            width: 300.0,
            height: 180.0,
            themed: true,
        }
    }
}

/// Spawn a reusable card container.
pub fn spawn_card(
    parent: &mut ChildSpawnerCommands,
    style: CardStyle,
    theme: &ThemeResource,
    content: impl FnOnce(&mut ChildSpawnerCommands),
) -> Entity {
    spawn_card_internal(parent, style, theme, None, content)
}

pub fn spawn_card_with_id(
    parent: &mut ChildSpawnerCommands,
    card_id: impl Into<String>,
    style: CardStyle,
    theme: &ThemeResource,
    content: impl FnOnce(&mut ChildSpawnerCommands),
) -> Entity {
    spawn_card_internal(parent, style, theme, Some(card_id.into()), content)
}

fn spawn_card_internal(
    parent: &mut ChildSpawnerCommands,
    style: CardStyle,
    theme: &ThemeResource,
    persistent_id: Option<String>,
    content: impl FnOnce(&mut ChildSpawnerCommands),
) -> Entity {
    let colors = theme.current.colors;
    let fill = if style.themed {
        colors.surface_paint()
    } else {
        Paint::solid(style.background)
    };

    let mut card = parent.spawn((
        Card {
            themed: style.themed,
            base_background: style.background,
        },
        Node {
            width: px(style.width),
            height: px(style.height),
            position_type: PositionType::Relative,
            padding: UiRect::all(px(style.padding)),
            border: UiRect::all(px(0.0)),
            flex_direction: FlexDirection::Column,
            border_radius: BorderRadius::all(px(style.border_radius)),
            ..default()
        },
        CardSizeModel::new(style.width, style.height),
        BackgroundColor(Color::NONE),
        BorderColor::all(Color::NONE),
        Surface::rounded_rect_fill(style.border_radius, fill).uniform_border(1.0, colors.border_paint()),
    ));

    if let Some(persistent_id) = persistent_id {
        card.insert(CardPersistentId(persistent_id));
    }

    let card_id = card.id();

    card.with_children(|card_node| {
        content(card_node);

        card_node.spawn((
            CardDragSurface { owner: card_id },
            Button,
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                right: px(0.0),
                top: px(0.0),
                bottom: px(0.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            ZIndex(1),
        ));

        card_node
            .spawn((
                CardResizeHandle { owner: card_id },
                Button,
                Node {
                    position_type: PositionType::Absolute,
                    width: px(20.0),
                    height: px(20.0),
                    right: px(6.0),
                    bottom: px(6.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::NONE),
                ZIndex(2),
            ))
            .with_children(|handle| {
                handle.spawn((
                    CardResizeGlyph,
                    ThemedText::new(TextRole::Muted),
                    Text::new("///"),
                    TextFont {
                        font_size: FontSize::Px(15.0),
                        ..default()
                    },
                    TextColor(Color::srgba(0.82, 0.86, 0.96, 0.95)),
                ));
            });
    });

    card_id
}

fn card_resize_handle_interaction_system(
    mut handles: Query<(&CardResizeHandle, &Interaction), (With<Button>, Changed<Interaction>)>,
    card_sizes: Query<&CardSizeModel, With<Card>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut active_resize: ResMut<ActiveCardResize>,
    mut active_drag: ResMut<ActiveCardDrag>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    for (handle, interaction) in &mut handles {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Some(cursor) = window.cursor_position() else {
            continue;
        };

        let Ok(card_size) = card_sizes.get(handle.owner) else {
            continue;
        };

        active_resize.0 = Some(CardResizeState {
            card: handle.owner,
            start_cursor: cursor,
            start_width: card_size.target_width,
            start_height: card_size.target_height,
        });
        active_drag.0 = None;
    }
}

fn card_resize_drag_system(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut card_sizes: Query<&mut CardSizeModel, With<Card>>,
    mut active_resize: ResMut<ActiveCardResize>,
    mut memory: ResMut<CardLayoutMemory>,
) {
    let Some(resize) = active_resize.0 else {
        return;
    };

    if !buttons.pressed(MouseButton::Left) {
        active_resize.0 = None;
        return;
    }

    let Ok(window) = windows.single() else {
        active_resize.0 = None;
        return;
    };

    let Some(cursor) = window.cursor_position() else {
        return;
    };

    let Ok(mut card_size) = card_sizes.get_mut(resize.card) else {
        active_resize.0 = None;
        return;
    };

    let delta = cursor - resize.start_cursor;
    let new_width = (resize.start_width + delta.x).max(CARD_MIN_WIDTH);
    let new_height = (resize.start_height + delta.y).max(CARD_MIN_HEIGHT);

    if (card_size.target_width - new_width).abs() > f32::EPSILON
        || (card_size.target_height - new_height).abs() > f32::EPSILON
    {
        card_size.target_width = new_width;
        card_size.target_height = new_height;
        memory.dirty = true;
    }
}

fn card_drag_surface_interaction_system(
    mut surfaces: Query<(&CardDragSurface, &Interaction), (With<Button>, Changed<Interaction>)>,
    card_parents: Query<&ChildOf, With<Card>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    active_resize: Res<ActiveCardResize>,
    mut active_drag: ResMut<ActiveCardDrag>,
    dashboards: Query<(), With<CardResponsiveContainer>>,
) {
    if active_resize.0.is_some() {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };

    for (surface, interaction) in &mut surfaces {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Some(_cursor) = window.cursor_position() else {
            continue;
        };

        let Ok(parent) = card_parents.get(surface.owner) else {
            continue;
        };

        let parent_entity = parent.parent();
        if dashboards.get(parent_entity).is_err() {
            continue;
        }

        active_drag.0 = Some(CardDragState {
            card: surface.owner,
            parent: parent_entity,
        });
    }
}

fn card_drag_reorder_system(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    active_resize: Res<ActiveCardResize>,
    surfaces: Query<(&CardDragSurface, &Interaction), With<Button>>,
    dashboard_children: Query<&Children, With<CardResponsiveContainer>>,
    mut active_drag: ResMut<ActiveCardDrag>,
    mut memory: ResMut<CardLayoutMemory>,
) {
    if active_resize.0.is_some() {
        active_drag.0 = None;
        return;
    }

    let Some(drag) = active_drag.0 else {
        return;
    };

    if !buttons.pressed(MouseButton::Left) {
        active_drag.0 = None;
        return;
    }

    let Ok(children) = dashboard_children.get(drag.parent) else {
        active_drag.0 = None;
        return;
    };

    let dragged_index = children.iter().position(|child| child == drag.card);
    let Some(dragged_index) = dragged_index else {
        active_drag.0 = None;
        return;
    };

    let hovered_owner = surfaces.iter().find_map(|(surface, interaction)| {
        (surface.owner != drag.card && *interaction == Interaction::Hovered)
            .then_some(surface.owner)
    });

    let Some(hovered_owner) = hovered_owner else {
        return;
    };

    let Some(target_index) = children.iter().position(|child| child == hovered_owner) else {
        return;
    };

    if target_index != dragged_index {
        commands
            .entity(drag.parent)
            .insert_children(target_index, &[drag.card]);
        memory.dirty = true;
    }
}

fn apply_card_target_size_system(mut cards: Query<(&mut Node, &CardSizeModel), With<Card>>) {
    for (mut node, size) in &mut cards {
        node.width = px(size.target_width.max(CARD_MIN_WIDTH));
        node.height = px(size.target_height.max(CARD_MIN_HEIGHT));
    }
}

fn card_dashboard_compaction_system(
    active_resize: Res<ActiveCardResize>,
    dashboards: Query<(&Children, &CardResponsiveContainer)>,
    card_sizes: Query<&CardSizeModel, With<Card>>,
    mut card_nodes: Query<&mut Node, With<Card>>,
) {
    let Some(resize) = active_resize.0 else {
        return;
    };

    for (children, dashboard) in &dashboards {
        if !children.iter().any(|child| child == resize.card) {
            continue;
        }

        let Ok(active_card_size) = card_sizes.get(resize.card) else {
            continue;
        };

        let width_growth = if active_card_size.base_width <= f32::EPSILON {
            1.0
        } else {
            active_card_size.target_width / active_card_size.base_width
        };
        let height_growth = if active_card_size.base_height <= f32::EPSILON {
            1.0
        } else {
            active_card_size.target_height / active_card_size.base_height
        };
        let growth = width_growth.max(height_growth).max(1.0);

        let shrink = (1.0 / growth.powf(0.45)).clamp(dashboard.min_neighbor_scale, 1.0);
        if (shrink - 1.0).abs() < f32::EPSILON {
            continue;
        }

        for child in children.iter() {
            if child == resize.card {
                continue;
            }

            let Ok(size) = card_sizes.get(child) else {
                continue;
            };
            let Ok(mut node) = card_nodes.get_mut(child) else {
                continue;
            };

            node.width = px((size.target_width * shrink).max(CARD_MIN_WIDTH));
            node.height = px((size.target_height * shrink).max(CARD_MIN_HEIGHT));
        }
    }
}

fn hydrate_card_layout_once_system(
    mut commands: Commands,
    mut memory: ResMut<CardLayoutMemory>,
    dashboards: Query<(Entity, &CardResponsiveContainer, &Children)>,
    mut card_sizes: Query<&mut CardSizeModel, With<Card>>,
    card_ids: Query<&CardPersistentId, With<Card>>,
) {
    if memory.hydrated {
        return;
    }

    for (dashboard_entity, dashboard, children) in &dashboards {
        let Some(layout) = memory.by_container.get(&dashboard.layout_id) else {
            continue;
        };

        let mut id_to_entity: HashMap<String, Entity> = HashMap::new();

        for child in children.iter() {
            let Ok(card_id) = card_ids.get(child) else {
                continue;
            };
            id_to_entity.insert(card_id.0.clone(), child);
        }

        for saved_size in &layout.sizes {
            let Some(entity) = id_to_entity.get(&saved_size.id).copied() else {
                continue;
            };

            if let Ok(mut size) = card_sizes.get_mut(entity) {
                size.target_width = saved_size.width.max(CARD_MIN_WIDTH);
                size.target_height = saved_size.height.max(CARD_MIN_HEIGHT);
            }
        }

        let mut reordered: Vec<Entity> = Vec::with_capacity(children.len());
        for card_id in &layout.order {
            if let Some(entity) = id_to_entity.get(card_id).copied() {
                reordered.push(entity);
            }
        }
        for child in children.iter() {
            if !reordered.contains(&child) {
                reordered.push(child);
            }
        }

        for (index, entity) in reordered.iter().copied().enumerate() {
            commands
                .entity(dashboard_entity)
                .insert_children(index, &[entity]);
        }
    }

    memory.hydrated = true;
}

fn persist_card_layout_if_dirty_system(
    active_resize: Res<ActiveCardResize>,
    active_drag: Res<ActiveCardDrag>,
    dashboards: Query<(&CardResponsiveContainer, &Children)>,
    card_ids: Query<&CardPersistentId, With<Card>>,
    card_sizes: Query<&CardSizeModel, With<Card>>,
    mut memory: ResMut<CardLayoutMemory>,
) {
    if !memory.dirty {
        return;
    }

    if active_resize.0.is_some() || active_drag.0.is_some() {
        return;
    }

    let mut containers: HashMap<String, PersistedContainerLayout> = HashMap::new();

    for (dashboard, children) in &dashboards {
        let mut order = Vec::new();
        let mut sizes = Vec::new();

        for child in children.iter() {
            let Ok(card_id) = card_ids.get(child) else {
                continue;
            };
            let Ok(size) = card_sizes.get(child) else {
                continue;
            };

            order.push(card_id.0.clone());
            sizes.push(PersistedCardSize {
                id: card_id.0.clone(),
                width: size.target_width,
                height: size.target_height,
            });
        }

        if !order.is_empty() {
            containers.insert(
                dashboard.layout_id.clone(),
                PersistedContainerLayout { order, sizes },
            );
        }
    }

    memory.by_container = containers.clone();

    let file = PersistedLayoutFile { containers };
    if save_layout_to_disk(&file).is_ok() {
        memory.dirty = false;
    }
}

fn card_resize_handle_visuals_system(
    handles: Query<(&Interaction, &Children), (With<CardResizeHandle>, Changed<Interaction>)>,
    mut glyphs: Query<&mut TextColor, With<CardResizeGlyph>>,
) {
    for (interaction, children) in &handles {
        let glyph_color = match *interaction {
            Interaction::Hovered => Color::srgba(0.92, 0.95, 1.0, 1.0),
            Interaction::Pressed => Color::srgba(0.72, 0.79, 0.95, 1.0),
            Interaction::None => Color::srgba(0.82, 0.86, 0.96, 0.95),
        };

        for child in children.iter() {
            if let Ok(mut color) = glyphs.get_mut(child) {
                color.0 = glyph_color;
            }
        }
    }
}

fn card_theme_system(
    theme: Res<ThemeResource>,
    mut cards: Query<(&Card, &mut Surface)>,
    mut glyphs: Query<&mut TextColor, With<CardResizeGlyph>>,
) {
    let colors = theme.current.colors;

    for (card, mut surface) in &mut cards {
        if card.themed {
            surface.fill = Paint::solid(colors.surface);
            surface.border = Some(Border::new(1.0, Paint::solid(colors.border)));
        } else {
            surface.fill = Paint::solid(card.base_background);
            surface.border = Some(Border::new(1.0, Paint::solid(CARD_BORDER_DEFAULT)));
        }
    }

    for mut glyph in &mut glyphs {
        glyph.0 = colors.text_muted;
    }
}

fn load_layout_from_disk() -> CardLayoutMemory {
    let path = dashboard_layout_path();
    let Ok(content) = fs::read_to_string(path) else {
        return CardLayoutMemory::default();
    };

    let Ok(file) = serde_json::from_str::<PersistedLayoutFile>(&content) else {
        return CardLayoutMemory::default();
    };

    CardLayoutMemory {
        by_container: file.containers,
        dirty: false,
        hydrated: false,
    }
}

fn save_layout_to_disk(layout: &PersistedLayoutFile) -> std::io::Result<()> {
    let path = dashboard_layout_path();
    let json = serde_json::to_string_pretty(layout).unwrap_or_else(|_| "{}".to_string());
    fs::write(path, json)
}

fn dashboard_layout_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(DASHBOARD_LAYOUT_FILE)
}

pub struct CardPlugin;

impl Plugin for CardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(load_layout_from_disk())
            .init_resource::<ActiveCardResize>()
            .init_resource::<ActiveCardDrag>()
            .add_systems(
                Update,
                (
                    hydrate_card_layout_once_system,
                    card_resize_handle_interaction_system,
                    card_resize_drag_system,
                    card_drag_surface_interaction_system,
                    card_drag_reorder_system,
                    apply_card_target_size_system,
                    card_dashboard_compaction_system,
                    card_resize_handle_visuals_system,
                    card_theme_system,
                    persist_card_layout_if_dirty_system,
                )
                    .chain(),
            );
    }
}
