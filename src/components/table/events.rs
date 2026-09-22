use bevy::prelude::*;

use super::component::{TableCellNode, TableHeaderNode, TableRowNode};

/// Events emitted by the table component.
#[derive(Message, Debug, Clone)]
pub enum TableEvent {
    RowClicked {
        table_id: String,
        row_id: String,
    },

    CellClicked {
        table_id: String,
        row_id: String,
        column_id: String,
    },

    HeaderClicked {
        table_id: String,
        column_id: String,
    },

    RowSelected {
        table_id: String,
        row_id: String,
    },
}

pub(crate) fn table_interaction_system(
    mut interaction_query: Query<
        (
            &Interaction,
            Option<&TableRowNode>,
            Option<&TableCellNode>,
            Option<&TableHeaderNode>,
        ),
        Changed<Interaction>,
    >,
    mut events: MessageWriter<TableEvent>,
) {
    for (interaction, row, cell, header) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if let Some(header) = header {
            events.write(TableEvent::HeaderClicked {
                table_id: header.table_id.clone(),
                column_id: header.column_id.clone(),
            });
            continue;
        }

        if let Some(cell) = cell {
            events.write(TableEvent::CellClicked {
                table_id: cell.table_id.clone(),
                row_id: cell.row_id.clone(),
                column_id: cell.column_id.clone(),
            });
            continue;
        }

        if let Some(row) = row {
            events.write(TableEvent::RowClicked {
                table_id: row.table_id.clone(),
                row_id: row.row_id.clone(),
            });

            if row.selectable {
                events.write(TableEvent::RowSelected {
                    table_id: row.table_id.clone(),
                    row_id: row.row_id.clone(),
                });
            }
        }
    }
}
