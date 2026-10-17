use bevy::prelude::*;

use crate::components::search::component::{
    Search, SearchBar, SearchChanged, SearchCompletion, SearchCompletionRequested,
    SearchPlaceholderText, SearchProviders, SearchSubmitted, SearchSuggestionSelected,
    SearchValueText,
};
use crate::rendering::{Border, Paint, Surface};
use crate::theme::ThemeResource;

pub fn focus_search_bar(
    interaction_query: Query<(Entity, &Interaction), (Changed<Interaction>, With<SearchBar>)>,
    mut searches: Query<(Entity, &mut Search), With<SearchBar>>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    let clicked = interaction_query
        .iter()
        .find_map(|(entity, interaction)| (*interaction == Interaction::Pressed).then_some(entity));

    if let Some(clicked_entity) = clicked {
        for (search_entity, mut search) in &mut searches {
            search.focused = search_entity == clicked_entity;
        }
        return;
    }

    if mouse.just_pressed(MouseButton::Left) {
        for (_, mut search) in &mut searches {
            search.focused = false;
        }
    }
}

pub fn update_search_visuals(
    theme: Res<ThemeResource>,
    mut searches: Query<(Entity, &Search, &mut Surface), With<SearchBar>>,
    mut value_query: Query<
        (&ChildOf, &mut Text),
        (With<SearchValueText>, Without<SearchPlaceholderText>),
    >,
    mut placeholder_query: Query<
        (&ChildOf, &mut Text, &mut Visibility),
        (With<SearchPlaceholderText>, Without<SearchValueText>),
    >,
) {
    let colors = theme.current.colors;

    for (entity, search, mut surface) in &mut searches {
        surface.fill = Paint::solid(colors.surface_elevated);
        surface.border = Some(Border::new(
            1.0,
            Paint::solid(if search.focused {
                colors.focus
            } else {
                colors.border
            }),
        ));

        for (parent, mut text) in &mut value_query {
            if parent.parent() == entity {
                *text = Text::new(search.query.clone());
            }
        }

        for (parent, mut placeholder_text, mut visibility) in &mut placeholder_query {
            if parent.parent() == entity {
                *placeholder_text = Text::new(search.placeholder.clone());
                *visibility = if search.query.is_empty() {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
}

pub fn search_keyboard_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut searches: Query<(&mut Search, &SearchCompletion)>,
    mut changed: MessageWriter<SearchChanged>,
    mut submitted: MessageWriter<SearchSubmitted>,
    mut selected: MessageWriter<SearchSuggestionSelected>,
) {
    for (mut search, completion) in searches.iter_mut() {
        if search.disabled || !search.focused {
            continue;
        }

        // ----------------------------------------------------
        // DOWN
        // ----------------------------------------------------

        if keyboard.just_pressed(KeyCode::ArrowDown) {
            if completion.suggestions.is_empty() {
                continue;
            }

            let next = match search.selected {
                None => 0,
                Some(index) => (index + 1) % completion.suggestions.len(),
            };

            search.selected = Some(next);
        }

        // ----------------------------------------------------
        // UP
        // ----------------------------------------------------

        if keyboard.just_pressed(KeyCode::ArrowUp) {
            if completion.suggestions.is_empty() {
                continue;
            }

            let next = match search.selected {
                None => completion.suggestions.len() - 1,
                Some(0) => completion.suggestions.len() - 1,
                Some(index) => index - 1,
            };

            search.selected = Some(next);
        }

        // ----------------------------------------------------
        // TAB
        // ----------------------------------------------------

        if keyboard.just_pressed(KeyCode::Tab)
            && !keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
        {
            if let Some(index) = search.selected {
                if let Some(suggestion) = completion.suggestions.get(index) {
                    search.query = suggestion.value.clone();
                    search.selected = None;

                    changed.write(SearchChanged {
                        search_id: search.id.clone(),
                        query: search.query.clone(),
                    });
                }
            }
        }

        // ----------------------------------------------------
        // ENTER
        // ----------------------------------------------------

        if keyboard.just_pressed(KeyCode::Enter) {
            if let Some(index) = search.selected {
                if let Some(suggestion) = completion.suggestions.get(index) {
                    search.query = suggestion.value.clone();

                    selected.write(SearchSuggestionSelected {
                        search_id: search.id.clone(),
                        value: suggestion.value.clone(),
                    });

                    search.selected = None;
                }
            } else {
                submitted.write(SearchSubmitted {
                    search_id: search.id.clone(),
                    query: search.query.clone(),
                });
            }
        }

        // ----------------------------------------------------
        // ESCAPE
        // ----------------------------------------------------

        if keyboard.just_pressed(KeyCode::Escape) {
            search.show_suggestions = false;
            search.selected = None;
        }
    }
}

#[derive(Component)]
struct SearchEditorState(String);

#[derive(Component)]
struct SuggestionItem {
    owner: Entity,
    index: usize,
}

#[derive(Component)]
struct SuggestionList {
    entity: Entity,
    values: Vec<crate::components::search::SearchSuggestion>,
}

pub(crate) fn sync_search_editor(world: &mut World) {
    let entities: Vec<_> = world
        .query_filtered::<Entity, With<Search>>()
        .iter(world)
        .collect();
    let focus = world
        .get_resource::<bevy::input_focus::InputFocus>()
        .and_then(|focus| focus.get());
    let clicks: Vec<_> = world
        .query_filtered::<(&SuggestionItem, &Interaction), Changed<Interaction>>()
        .iter(world)
        .filter(|(_, interaction)| **interaction == Interaction::Pressed)
        .map(|(item, _)| (item.owner, item.index))
        .collect();
    for entity in entities {
        let input = world.get::<Children>(entity).and_then(|children| {
            children.iter().find(|child| {
                world
                    .get::<crate::components::input::TextInput>(*child)
                    .is_some()
            })
        });
        let Some(input) = input else { continue };
        for (_, index) in clicks.iter().filter(|(owner, _)| *owner == entity) {
            let value = world
                .get::<SearchCompletion>(entity)
                .and_then(|completion| completion.suggestions.get(*index))
                .map(|suggestion| suggestion.value.clone());
            if let Some(value) = value {
                let mut search = world.get_mut::<Search>(entity).unwrap();
                if !search.disabled {
                    search.query = value.clone();
                    search.show_suggestions = false;
                    search.selected = None;
                    let search_id = search.id.clone();
                    world.write_message(SearchSuggestionSelected { search_id, value });
                    world.write_message(crate::primitives::focus::FocusRequest {
                        target: input,
                        origin: crate::primitives::focus::FocusOrigin::Keyboard,
                    });
                }
            }
        }
        let previous = world
            .get::<SearchEditorState>(entity)
            .map(|state| state.0.clone());
        let query = world.get::<Search>(entity).unwrap().query.clone();
        let focused = focus == Some(input)
            || focus.is_some_and(|focused| {
                world
                    .get::<SuggestionItem>(focused)
                    .is_some_and(|item| item.owner == entity)
            });
        let edited = world
            .get::<crate::components::input::TextInput>(input)
            .unwrap()
            .value
            .clone();
        let disabled = world.get::<Search>(entity).unwrap().disabled;
        world.get_mut::<Search>(entity).unwrap().focused = focused && !disabled;
        world
            .get_mut::<crate::components::input::TextInput>(input)
            .unwrap()
            .disabled = disabled;
        if previous.as_ref() != Some(&query) {
            let mut control = world
                .get_mut::<crate::components::input::TextInput>(input)
                .unwrap();
            control.value = query.clone();
            control.cursor = query.chars().count();
        } else if !disabled && edited != query {
            let mut search = world.get_mut::<Search>(entity).unwrap();
            search.query = edited;
            search.show_suggestions = focused;
            search.selected = None;
            let search_id = search.id.clone();
            let query = search.query.clone();
            world.write_message(SearchChanged {
                search_id: search_id.clone(),
                query: query.clone(),
            });
            world.write_message(SearchCompletionRequested { search_id, query });
        }
        let query = world.get::<Search>(entity).unwrap().query.clone();
        world.entity_mut(entity).insert(SearchEditorState(query));
    }
}

pub(crate) fn sync_search_suggestions(world: &mut World) {
    use crate::primitives::semantic::{SemanticNode, SemanticRole};
    let colors = world.resource::<ThemeResource>().current.colors;
    let entities: Vec<_> = world
        .query_filtered::<Entity, With<Search>>()
        .iter(world)
        .collect();
    for entity in entities {
        let input = world.get::<Children>(entity).and_then(|children| {
            children.iter().find(|child| {
                world
                    .get::<crate::components::input::TextInput>(*child)
                    .is_some()
            })
        });
        let Some(input) = input else { continue };
        let values = world
            .get::<SearchCompletion>(entity)
            .map(|completion| completion.suggestions.clone())
            .unwrap_or_default();
        let search = world.get::<Search>(entity).unwrap();
        let visible =
            search.focused && search.show_suggestions && !search.disabled && !values.is_empty();
        let selected = search.selected;
        let query = search.query.clone();
        let label = search.placeholder.clone();
        let recreate = world
            .get::<SuggestionList>(entity)
            .is_none_or(|list| list.values != values);
        if recreate {
            if let Some(list) = world.entity_mut(entity).take::<SuggestionList>() {
                world.despawn(list.entity);
            }
            let mut semantic =
                SemanticNode::new(SemanticRole::ListBox).label(format!("{label} suggestions"));
            semantic.relationships.controlled_by = Some(input);
            semantic.actions.clear();
            let list = world
                .spawn((
                    semantic,
                    Node {
                        position_type: PositionType::Absolute,
                        top: percent(100),
                        width: percent(100),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(4),
                        padding: UiRect::all(px(6)),
                        ..default()
                    },
                    BackgroundColor(colors.surface_elevated),
                    ZIndex(20),
                    ChildOf(entity),
                ))
                .id();
            for (index, suggestion) in values.iter().enumerate() {
                let mut semantic =
                    SemanticNode::new(SemanticRole::ListBoxOption).label(suggestion.label.clone());
                semantic.description = suggestion.description.clone();
                let option = world
                    .spawn((
                        semantic,
                        Button,
                        SuggestionItem {
                            owner: entity,
                            index,
                        },
                        Node {
                            padding: UiRect::all(px(8)),
                            min_height: px(36),
                            ..default()
                        },
                        ChildOf(list),
                    ))
                    .id();
                world.spawn((
                    Text::new(suggestion.label.clone()),
                    TextColor(colors.text),
                    crate::components::text::ThemedText::new(
                        crate::components::text::TextRole::Body,
                    ),
                    ChildOf(option),
                ));
            }
            world.entity_mut(entity).insert(SuggestionList {
                entity: list,
                values,
            });
        }
        let list = world.get::<SuggestionList>(entity).unwrap().entity;
        world.get_mut::<Node>(list).unwrap().display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        world.get_mut::<SemanticNode>(list).unwrap().state.hidden = !visible;
        world
            .entity_mut(list)
            .insert(BackgroundColor(colors.surface_elevated));
        let options: Vec<_> = world
            .get::<Children>(list)
            .map(|children| children.iter().collect())
            .unwrap_or_default();
        let mut active = None;
        for (index, option) in options.iter().enumerate() {
            let mut semantic = world.get_mut::<SemanticNode>(*option).unwrap();
            semantic.state.hidden = !visible;
            semantic.state.selected = selected == Some(index);
            world
                .entity_mut(*option)
                .insert(BackgroundColor(if selected == Some(index) {
                    colors.surface
                } else {
                    colors.surface_elevated
                }));
            if visible && selected == Some(index) {
                active = Some(*option);
            }
        }
        if let Some(mut semantic) = world.get_mut::<SemanticNode>(input) {
            semantic.state.expanded = Some(visible);
            semantic.relationships.controls = vec![list];
            semantic.relationships.active_descendant = active;
        }
        if world
            .get::<crate::components::input::TextInput>(input)
            .unwrap()
            .value
            != query
        {
            let mut control = world
                .get_mut::<crate::components::input::TextInput>(input)
                .unwrap();
            control.value = query.clone();
            control.cursor = query.chars().count();
        }
        world.get_mut::<SearchEditorState>(entity).unwrap().0 = query;
    }
}

#[cfg(test)]
mod accessibility_tests {
    use super::*;
    use crate::components::search::{SearchPlugin, SearchSuggestion, spawn_search};
    use crate::primitives::semantic::{SemanticNode, SemanticRole};
    use bevy::input_focus::InputFocus;

    #[test]
    fn search_editor_focus_and_suggestions_expose_a_listbox_and_dismiss() {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(ThemeResource::default())
            .add_message::<crate::primitives::focus::FocusRequest>()
            .add_plugins(SearchPlugin);
        let parent = app.world_mut().spawn(Node::default()).id();
        let mut root = None;
        app.world_mut()
            .commands()
            .entity(parent)
            .with_children(|children| {
                root = Some(spawn_search(
                    children,
                    Handle::default(),
                    &ThemeResource::default(),
                ));
            });
        app.world_mut().flush();
        let root = root.unwrap();
        let input = app.world().get::<Children>(root).unwrap()[0];
        app.insert_resource(InputFocus::from_entity(input));
        app.world_mut()
            .get_mut::<SearchCompletion>(root)
            .unwrap()
            .suggestions = vec![SearchSuggestion::new("report", "Report")];
        app.world_mut()
            .get_mut::<Search>(root)
            .unwrap()
            .show_suggestions = true;
        app.world_mut().get_mut::<Search>(root).unwrap().selected = Some(0);
        app.update();
        assert!(app.world().get::<Search>(root).unwrap().focused);
        let semantic = app.world().get::<SemanticNode>(input).unwrap();
        assert_eq!(semantic.state.expanded, Some(true));
        let list = semantic.relationships.controls[0];
        let option = semantic.relationships.active_descendant.unwrap();
        assert_eq!(
            app.world().get::<SemanticNode>(list).unwrap().role,
            SemanticRole::ListBox
        );
        assert_eq!(
            app.world()
                .get::<SemanticNode>(option)
                .unwrap()
                .label
                .as_deref(),
            Some("Report")
        );
        assert_eq!(
            app.world().get::<Node>(list).unwrap().display,
            Display::Flex
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
        assert_eq!(
            app.world()
                .get::<SemanticNode>(input)
                .unwrap()
                .state
                .expanded,
            Some(false)
        );
        assert_eq!(
            app.world().get::<Node>(list).unwrap().display,
            Display::None
        );
        app.world_mut().get_mut::<Search>(root).unwrap().disabled = true;
        app.update();
        assert!(
            app.world()
                .get::<crate::components::input::TextInput>(input)
                .unwrap()
                .disabled
        );
    }
}

pub fn search_completion_system(
    mut requests: MessageReader<SearchCompletionRequested>,
    mut searches: Query<(&Search, &mut SearchCompletion)>,
    providers: Res<SearchProviders>,
) {
    for request in requests.read() {
        for (search, mut completion) in searches.iter_mut() {
            if search.id != request.search_id {
                continue;
            }

            completion.suggestions =
                providers.complete(&request.search_id, &request.query, search.max_suggestions);
        }
    }
}
