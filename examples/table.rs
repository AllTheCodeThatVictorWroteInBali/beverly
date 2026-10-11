//! Table demo: the snack desk's emergency inventory.
//! Run with `-- dark` to start in dark mode.

use beverly::components::table::{
    Table, TableCell, TableColumn, TableConfig, TableEvent, TableRow,
};
use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::prelude::*;
use bevy::prelude::*;

const INVENTORY_ID: &str = "snack-inventory";
const STOCK: [(&str, &str, &str, &str, &str); 6] = [
    (
        "biscuits",
        "Emergency biscuits",
        "24",
        "Snack Desk",
        "Under surveillance",
    ),
    (
        "tea",
        "Very serious tea",
        "12",
        "Kettle Committee",
        "Ready to brew",
    ),
    (
        "toast",
        "Strategic toast",
        "8",
        "Breakfast Division",
        "Lightly buttered",
    ),
    (
        "jam",
        "Contingency jam",
        "3",
        "Spread Operations",
        "Sticky situation",
    ),
    (
        "crumb",
        "Suspicious crumb",
        "1",
        "Crumb Inspector",
        "Evidence pending",
    ),
    (
        "napkins",
        "Official napkins",
        "40",
        "Mess Management",
        "Folded with pride",
    ),
];

#[derive(Component)]
struct InventoryReadout;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, update_readout)
        .run();
}

fn inventory_config() -> TableConfig {
    TableConfig {
        id: INVENTORY_ID.to_string(),
        columns: [
            ("item", "Item", 30.0),
            ("stock", "Stock", 10.0),
            ("custodian", "Custodian", 28.0),
            ("status", "Status", 32.0),
        ]
        .into_iter()
        .map(|(id, label, width)| TableColumn {
            id: id.to_string(),
            label: label.to_string(),
            width,
        })
        .collect(),
        rows: STOCK
            .into_iter()
            .map(|(id, item, stock, custodian, status)| TableRow {
                id: id.to_string(),
                cells: [item, stock, custodian, status]
                    .into_iter()
                    .map(|text| TableCell {
                        text: text.to_string(),
                    })
                    .collect(),
            })
            .collect(),
        striped: true,
        selectable_rows: true,
        ..default()
    }
}

fn setup(mut commands: Commands) {
    spawn_themed_page(&mut commands, |root| {
        root.spawn(Node {
            width: Val::Px(820.0),
            max_width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            row_gap: Val::Px(18.0),
            ..default()
        })
        .with_children(|content| {
            content.spawn((
                ThemedTitle::new(TitleLevel::H2),
                Text::new("Emergency snack inventory"),
            ));
            content.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("Six supplies. Five departments. One deeply suspicious crumb."),
            ));
            Table::spawn_into(content, inventory_config());
            content.spawn((
                InventoryReadout,
                ThemedText::new(TextRole::Body),
                Text::new("Last inspected: none. The biscuits are enjoying their privacy."),
            ));
        });
    });
}

fn update_readout(
    mut events: MessageReader<TableEvent>,
    mut readouts: Query<&mut Text, With<InventoryReadout>>,
) {
    for event in events.read() {
        let (table_id, row_id) = match event {
            TableEvent::RowClicked { table_id, row_id }
            | TableEvent::RowSelected { table_id, row_id }
            | TableEvent::CellClicked {
                table_id, row_id, ..
            } => (table_id, row_id),
            TableEvent::HeaderClicked { .. } => continue,
        };
        if table_id != INVENTORY_ID {
            continue;
        }
        let Some((_, item, stock, custodian, status)) = STOCK.iter().find(|row| row.0 == row_id)
        else {
            continue;
        };
        for mut text in &mut readouts {
            *text = Text::new(format!(
                "Last inspected: {item} ({stock}) - {custodian}. {status}."
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_click_updates_readout_and_ignores_other_tables() {
        let mut app = App::new();
        app.add_message::<TableEvent>()
            .add_systems(Update, update_readout);
        let readout = app
            .world_mut()
            .spawn((InventoryReadout, Text::new("before")))
            .id();
        app.world_mut().write_message(TableEvent::CellClicked {
            table_id: INVENTORY_ID.to_string(),
            row_id: "crumb".to_string(),
            column_id: "status".to_string(),
        });
        app.update();
        let expected = "Last inspected: Suspicious crumb (1) - Crumb Inspector. Evidence pending.";
        assert_eq!(app.world().get::<Text>(readout).unwrap().0, expected);
        app.world_mut().write_message(TableEvent::RowClicked {
            table_id: "another-table".to_string(),
            row_id: "biscuits".to_string(),
        });
        app.update();
        assert_eq!(app.world().get::<Text>(readout).unwrap().0, expected);
        let config = inventory_config();
        assert_eq!(
            config
                .columns
                .iter()
                .map(|column| column.width)
                .sum::<f32>(),
            100.0
        );
        assert!(
            config
                .rows
                .iter()
                .all(|row| row.cells.len() == config.columns.len())
        );
    }
}
