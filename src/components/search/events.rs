use bevy::prelude::*;

use crate::rendering::{Border, Paint, Surface};
use crate::components::search::component::{
    Search, SearchBar, SearchChanged, SearchCompletion, SearchCompletionRequested,
    SearchPlaceholderText, SearchProviders, SearchSubmitted, SearchSuggestionSelected,
    SearchValueText,
};
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
    mut value_query: Query<(&ChildOf, &mut Text), (With<SearchValueText>, Without<SearchPlaceholderText>)>,
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
        if !search.focused {
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

        if keyboard.just_pressed(KeyCode::Tab) {
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
