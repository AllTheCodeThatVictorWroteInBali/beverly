use bevy::prelude::*;

use crate::rendering::{Paint, Surface};
use crate::components::text::{TextRole, ThemedText};

use super::component::{TableConfig, TableHeader, TableHeaderLabel, TableHeaderNode};

pub(crate) fn spawn_header(parent: &mut ChildSpawnerCommands, config: &TableConfig) {
    parent
        .spawn((
            TableHeader,
            Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(config.header_min_height),
                border: UiRect::bottom(px(1.0)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
            Surface::rounded_rect_fill(0.0, Paint::solid(Color::srgba(1.0, 1.0, 1.0, 0.08))),
        ))
        .with_children(|header| {
            for column in &config.columns {
                header
                    .spawn((
                        TableHeaderNode {
                            table_id: config.id.clone(),
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
                        Surface::rounded_rect_fill(0.0, Paint::solid(Color::srgba(1.0, 1.0, 1.0, 0.00))),
                    ))
                    .with_children(|button| {
                        button.spawn((
                            TableHeaderLabel,
                            ThemedText::new(TextRole::Label),
                            Text::new(column.label.clone()),
                            TextFont {
                                font_size: FontSize::Px(config.header_font_size),
                                ..default()
                            },
                            TextColor(Color::srgba(0.10, 0.12, 0.16, 0.90)),
                        ));
                    });
            }
        });
}
