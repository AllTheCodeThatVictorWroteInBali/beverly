use bevy::prelude::*;

use crate::rendering::{Paint, Surface};
use super::data::TableResponse;
use super::events::{TableEvent, table_interaction_system};
use super::header::spawn_header;
use super::rows::spawn_row;
use crate::animation::loading::LoadingAssets;
use crate::theme::ThemeResource;

const HEADER_FONT_SIZE: f32 = 14.0;
const CELL_FONT_SIZE: f32 = 14.0;
const ROW_MIN_HEIGHT: f32 = 46.0;
const HEADER_MIN_HEIGHT: f32 = 48.0;
const CELL_PADDING_X: f32 = 12.0;
const CELL_PADDING_Y: f32 = 10.0;

/// Root Bevy component for a table.
#[derive(Component)]
pub struct Table {
    pub id: String,
}

/// Defines a table column in Bevy layout space.
#[derive(Clone, Debug)]
pub struct TableColumn {
    pub id: String,
    pub label: String,
    /// Percentage width for the column.
    pub width: f32,
}

/// A single cell rendered in a row.
#[derive(Clone, Debug)]
pub struct TableCell {
    pub text: String,
}

/// A row rendered by the table.
#[derive(Clone, Debug)]
pub struct TableRow {
    pub id: String,
    pub cells: Vec<TableCell>,
}

/// Local table configuration used by the renderer.
#[derive(Clone, Debug)]
pub struct TableConfig {
    pub id: String,
    pub columns: Vec<TableColumn>,
    pub rows: Vec<TableRow>,

    pub show_header: bool,
    pub selectable_rows: bool,
    pub striped: bool,

    pub header_font_size: f32,
    pub cell_font_size: f32,
    pub row_min_height: f32,
    pub header_min_height: f32,
    pub cell_padding_x: f32,
    pub cell_padding_y: f32,
}

impl Default for TableConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            columns: Vec::new(),
            rows: Vec::new(),
            show_header: true,
            selectable_rows: false,
            striped: false,
            header_font_size: HEADER_FONT_SIZE,
            cell_font_size: CELL_FONT_SIZE,
            row_min_height: ROW_MIN_HEIGHT,
            header_min_height: HEADER_MIN_HEIGHT,
            cell_padding_x: CELL_PADDING_X,
            cell_padding_y: CELL_PADDING_Y,
        }
    }
}

/// Plugin containing table systems.
pub struct TablePlugin;

impl Plugin for TablePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TableEvent>().add_systems(
            Update,
            (
                table_interaction_system,
                table_visual_system,
                apply_table_demo_font_system,
            ),
        );
    }
}

/// Marker components used by interaction and rendering systems.
#[derive(Component)]
pub(crate) struct TableHeader;

#[derive(Component)]
pub(crate) struct TableRowNode {
    pub table_id: String,
    pub row_id: String,
    pub selectable: bool,
    pub row_index: usize,
    pub striped: bool,
}

#[derive(Component)]
pub(crate) struct TableCellNode {
    pub table_id: String,
    pub row_id: String,
    pub column_id: String,
}

#[derive(Component)]
pub(crate) struct TableHeaderNode {
    pub table_id: String,
    pub column_id: String,
}

#[derive(Component)]
pub(crate) struct TableHeaderLabel;

#[derive(Component)]
pub(crate) struct TableCellLabel;

impl Table {
    /// Spawns a table from backend-agnostic JSON contract data.
    pub fn spawn_from_data(commands: &mut Commands, data: &TableResponse) -> Entity {
        let column_count = data.columns.len().max(1) as f32;
        let columns = data
            .columns
            .iter()
            .map(|column| TableColumn {
                id: column.id.clone(),
                label: column.label.clone(),
                width: column
                    .width
                    .unwrap_or(100.0 / column_count)
                    .clamp(1.0, 100.0),
            })
            .collect::<Vec<_>>();

        let rows = data
            .rows
            .iter()
            .map(|row| {
                let cells = columns
                    .iter()
                    .map(|column| TableCell {
                        text: row
                            .cells
                            .get(&column.id)
                            .map(|value| value.to_text())
                            .unwrap_or_default(),
                    })
                    .collect::<Vec<_>>();

                TableRow {
                    id: row.id.clone(),
                    cells,
                }
            })
            .collect::<Vec<_>>();

        let config = TableConfig {
            id: data.id.clone(),
            columns,
            rows,
            show_header: data.show_header,
            selectable_rows: data.selectable_rows,
            striped: data.striped,
            ..default()
        };

        Self::spawn(commands, config)
    }

    /// Spawns a table from already-normalized UI data.
    pub fn spawn(commands: &mut Commands, config: TableConfig) -> Entity {
        let table_id = config.id.clone();
        let root_background = Color::srgba(1.0, 1.0, 1.0, 0.05);

        let entity = commands
            .spawn((
                Table {
                    id: config.id.clone(),
                },
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    border: UiRect::all(px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(14.0)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                BorderColor::all(Color::NONE),
                Surface::rounded_rect_fill(14.0, Paint::solid(root_background))
                    .uniform_border(1.0, Paint::solid(Color::srgba(1.0, 1.0, 1.0, 0.16))),
            ))
            .with_children(|parent| {
                if config.show_header {
                    spawn_header(parent, &config);
                }

                for (index, row) in config.rows.iter().enumerate() {
                    spawn_row(parent, &table_id, &config, row, index);
                }
            })
            .id();

        entity
    }
}

fn table_visual_system(
    theme: Res<ThemeResource>,
    mut visual_queries: ParamSet<(
        Query<&mut Surface, (With<Table>, Without<TableHeader>, Without<TableRowNode>)>,
        Query<&mut Surface, (With<TableHeader>, Without<Table>, Without<TableRowNode>)>,
        Query<
            (&TableRowNode, &Interaction, &mut Surface),
            (Without<Table>, Without<TableHeader>),
        >,
        Query<
            (
                &mut TextColor,
                Option<&TableHeaderLabel>,
                Option<&TableCellLabel>,
            ),
            Or<(With<TableHeaderLabel>, With<TableCellLabel>)>,
        >,
    )>,
) {
    let colors = theme.current.colors;

    for mut surface in &mut visual_queries.p0() {
        surface.fill = Paint::solid(colors.surface.with_alpha(0.66));
    }

    for mut surface in &mut visual_queries.p1() {
        surface.fill = Paint::solid(colors.surface_elevated.with_alpha(0.92));
    }

    for (row, interaction, mut surface) in &mut visual_queries.p2() {
        let base = if row.striped && row.row_index % 2 == 1 {
            colors.surface.with_alpha(0.34)
        } else {
            colors.surface.with_alpha(0.14)
        };

        surface.fill = Paint::solid(match *interaction {
            Interaction::Pressed => colors.secondary.with_alpha(0.94),
            Interaction::Hovered => colors.surface_elevated.with_alpha(0.96),
            Interaction::None => base,
        });
    }

    for (mut text_color, header, cell) in &mut visual_queries.p3() {
        if header.is_some() {
            text_color.0 = colors.text_muted;
        } else if cell.is_some() {
            text_color.0 = colors.text;
        }
    }
}

fn apply_table_demo_font_system(
    loading_assets: Option<Res<LoadingAssets>>,
    mut labels: Query<&mut TextFont, Or<(With<TableHeaderLabel>, With<TableCellLabel>)>>,
) {
    let Some(loading_assets) = loading_assets else {
        return;
    };

    for mut font in &mut labels {
        font.font = FontSource::Handle(loading_assets.font.clone());
    }
}
