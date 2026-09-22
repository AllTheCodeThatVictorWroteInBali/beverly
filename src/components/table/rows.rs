use bevy::prelude::*;

use crate::rendering::{Paint, Surface};
use crate::components::text::{TextRole, ThemedText};

use super::component::{TableCellLabel, TableCellNode, TableConfig, TableRow, TableRowNode};

pub(crate) fn spawn_row(
    parent: &mut ChildSpawnerCommands,
    table_id: &str,
    config: &TableConfig,
    row: &TableRow,
    index: usize,
) {
    let row_background = if config.striped && index % 2 == 1 {
        Color::srgba(1.0, 1.0, 1.0, 0.04)
    } else {
        Color::srgba(1.0, 1.0, 1.0, 0.00)
    };

    parent
        .spawn((
            TableRowNode {
                table_id: table_id.to_string(),
                row_id: row.id.clone(),
                selectable: config.selectable_rows,
                row_index: index,
                striped: config.striped,
            },
            Button,
            Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(config.row_min_height),
                border: UiRect::bottom(px(1.0)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderColor::all(Color::NONE),
            Surface::rounded_rect_fill(0.0, Paint::solid(row_background))
                .uniform_border(1.0, Paint::solid(Color::srgba(1.0, 1.0, 1.0, 0.06))),
        ))
        .with_children(|row_node| {
            for (column_index, cell) in row.cells.iter().enumerate() {
                let Some(column) = config.columns.get(column_index) else {
                    continue;
                };

                row_node
                    .spawn((
                        TableCellNode {
                            table_id: table_id.to_string(),
                            row_id: row.id.clone(),
                            column_id: column.id.clone(),
                        },
                        Button,
                        Node {
                            width: Val::Percent(column.width),
                            min_height: Val::Percent(100.0),
                            padding: UiRect::axes(
                                Val::Px(config.cell_padding_x),
                                Val::Px(config.cell_padding_y),
                            ),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|cell_node| {
                        cell_node.spawn((
                            TableCellLabel,
                            ThemedText::new(TextRole::Body),
                            Text::new(cell.text.clone()),
                            TextFont {
                                font_size: FontSize::Px(config.cell_font_size),
                                ..default()
                            },
                            TextColor(Color::srgb(0.10, 0.12, 0.16)),
                        ));
                    });
            }
        });
}
