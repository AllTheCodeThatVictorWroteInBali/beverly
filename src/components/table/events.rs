use crate::primitives::focus::{FocusOrigin, FocusRequest};
use crate::primitives::semantic::SemanticNode;
use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::input_focus::InputFocus;
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
            Entity,
            &Interaction,
            &ChildOf,
            Option<&TableRowNode>,
            Option<&TableCellNode>,
            Option<&TableHeaderNode>,
        ),
        Changed<Interaction>,
    >,
    mut events: MessageWriter<TableEvent>,
    mut semantics: Query<(&TableRowNode, &ChildOf, &mut SemanticNode)>,
) {
    for (entity, interaction, parent, row, cell, header) in &mut interaction_query {
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
            if let Ok((row, _, _)) = semantics.get(parent.parent()) {
                if row.selectable {
                    events.write(TableEvent::RowSelected {
                        table_id: row.table_id.clone(),
                        row_id: row.row_id.clone(),
                    });
                    select_row(parent.parent(), &mut semantics);
                }
            }
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
                select_row(entity, &mut semantics);
            }
        }
    }
}

fn select_row(entity: Entity, rows: &mut Query<(&TableRowNode, &ChildOf, &mut SemanticNode)>) {
    let Ok((_, parent, _)) = rows.get(entity) else {
        return;
    };
    let table = parent.parent();
    for (_, parent, mut semantic) in rows.iter_mut() {
        if parent.parent() == table {
            semantic.state.selected = false;
        }
    }
    let mut selected = rows.get_mut(entity).unwrap();
    selected.2.state.selected = true;
}

#[cfg(test)]
mod tests {
    use super::super::component::{Table, TableCell, TableColumn, TableConfig, TableRow};
    use super::*;
    use bevy::input::keyboard::Key;

    #[test]
    fn table_keyboard_navigates_and_selects_exactly_one_row() {
        let mut app = App::new();
        app.init_resource::<InputFocus>()
            .add_message::<KeyboardInput>()
            .add_message::<FocusRequest>()
            .add_message::<TableEvent>()
            .add_systems(
                Update,
                (table_keyboard_system, table_interaction_system).chain(),
            );
        let parent = app.world_mut().spawn(Node::default()).id();
        app.world_mut()
            .commands()
            .entity(parent)
            .with_children(|children| {
                Table::spawn_into(
                    children,
                    TableConfig {
                        id: "records".to_owned(),
                        selectable_rows: true,
                        columns: vec![TableColumn {
                            id: "name".to_owned(),
                            label: "Name".to_owned(),
                            width: 100.0,
                        }],
                        rows: vec![
                            TableRow {
                                id: "first".to_owned(),
                                cells: vec![TableCell {
                                    text: "First".to_owned(),
                                }],
                            },
                            TableRow {
                                id: "second".to_owned(),
                                cells: vec![TableCell {
                                    text: "Second".to_owned(),
                                }],
                            },
                        ],
                        ..default()
                    },
                );
            });
        app.world_mut().flush();
        let mut rows: Vec<_> = app
            .world_mut()
            .query::<(Entity, &TableRowNode)>()
            .iter(app.world())
            .map(|(entity, row)| (entity, row.row_index))
            .collect();
        rows.sort_by_key(|(_, index)| *index);
        let mut cells: Vec<_> = app
            .world_mut()
            .query::<(Entity, &TableCellNode, &ChildOf)>()
            .iter(app.world())
            .map(|(entity, _, parent)| (entity, parent.parent()))
            .collect();
        cells.sort_by_key(|(_, row)| rows.iter().position(|(entity, _)| entity == row).unwrap());
        app.insert_resource(InputFocus::from_entity(cells[0].0));
        app.world_mut().write_message(KeyboardInput {
            key_code: KeyCode::ArrowDown,
            logical_key: Key::Space,
            state: ButtonState::Pressed,
            repeat: false,
            text: None,
            window: parent,
        });
        app.update();
        let requests: Vec<_> = app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<FocusRequest>>()
            .drain()
            .collect();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].target, cells[1].0);
        for cell in &cells {
            app.insert_resource(InputFocus::from_entity(cell.0));
            app.world_mut().write_message(KeyboardInput {
                key_code: KeyCode::Enter,
                logical_key: Key::Enter,
                state: ButtonState::Pressed,
                repeat: false,
                text: None,
                window: parent,
            });
            app.update();
        }
        assert!(
            !app.world()
                .get::<SemanticNode>(rows[0].0)
                .unwrap()
                .state
                .selected
        );
        assert!(
            app.world()
                .get::<SemanticNode>(rows[1].0)
                .unwrap()
                .state
                .selected
        );
        let events: Vec<_> = app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<TableEvent>>()
            .drain()
            .collect();
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, TableEvent::RowSelected { .. }))
                .count(),
            2
        );
    }
}

pub(crate) fn table_keyboard_system(
    mut keyboard: MessageReader<KeyboardInput>,
    focus: Option<Res<InputFocus>>,
    cells: Query<(Entity, &TableCellNode, &ChildOf)>,
    rows: Query<(&TableRowNode, &ChildOf)>,
    headers: Query<(Entity, &TableHeaderNode, &ChildOf)>,
    parents: Query<&ChildOf>,
    mut requests: MessageWriter<FocusRequest>,
    mut nodes: Query<(&mut Interaction, &SemanticNode)>,
    mut indices: Query<&mut crate::primitives::a11y::TabIndex>,
) {
    let Some(focused) = focus.and_then(|focus| focus.get()) else {
        return;
    };
    let mut positions = Vec::new();
    let mut table = None;
    for (entity, _, parent) in &cells {
        if entity == focused {
            table = rows
                .get(parent.parent())
                .ok()
                .map(|(_, parent)| parent.parent());
        }
    }
    for (entity, _, parent) in &headers {
        if entity == focused {
            table = parents.get(parent.parent()).ok().map(ChildOf::parent);
        }
    }
    let Some(table) = table else { return };
    for (entity, cell, parent) in &cells {
        if let Ok((row, row_parent)) = rows.get(parent.parent()) {
            if row_parent.parent() == table {
                positions.push((entity, row.row_index + 1, cell.column_index));
            }
        }
    }
    for (entity, header, parent) in &headers {
        if parents
            .get(parent.parent())
            .is_ok_and(|parent| parent.parent() == table)
        {
            positions.push((entity, 0, header.column_index));
        }
    }
    positions.sort_by_key(|(_, row, column)| (*row, *column));
    let Some((_, row, column)) = positions
        .iter()
        .find(|(entity, _, _)| *entity == focused)
        .copied()
    else {
        return;
    };
    for event in keyboard.read() {
        if event.state != ButtonState::Pressed || event.repeat {
            continue;
        }
        if nodes
            .get(focused)
            .is_ok_and(|(_, semantic)| semantic.state.disabled)
        {
            continue;
        }
        if matches!(event.key_code, KeyCode::Enter | KeyCode::Space) {
            if let Ok((mut interaction, _)) = nodes.get_mut(focused) {
                *interaction = Interaction::Pressed;
            }
            continue;
        }
        let desired = match event.key_code {
            KeyCode::ArrowLeft => positions
                .iter()
                .rev()
                .find(|(_, target_row, target_column)| {
                    *target_row == row && *target_column < column
                }),
            KeyCode::ArrowRight => positions.iter().find(|(_, target_row, target_column)| {
                *target_row == row && *target_column > column
            }),
            KeyCode::ArrowUp => positions
                .iter()
                .rev()
                .find(|(_, target_row, target_column)| {
                    *target_row < row && *target_column == column
                }),
            KeyCode::ArrowDown => positions.iter().find(|(_, target_row, target_column)| {
                *target_row > row && *target_column == column
            }),
            KeyCode::Home => positions
                .iter()
                .find(|(_, target_row, _)| *target_row == row),
            KeyCode::End => positions
                .iter()
                .rev()
                .find(|(_, target_row, _)| *target_row == row),
            _ => None,
        };
        if let Some((target, _, _)) = desired {
            for (entity, _, _) in &positions {
                if let Ok(mut index) = indices.get_mut(*entity) {
                    index.0 = if entity == target { 0 } else { -1 };
                }
            }
            requests.write(FocusRequest {
                target: *target,
                origin: FocusOrigin::Keyboard,
            });
        }
    }
}
