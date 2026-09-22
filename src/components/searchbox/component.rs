use bevy::prelude::*;

use crate::rendering::{Paint, Surface};
use crate::icons::{Icon, IconNode};
use crate::components::input::{TextInputConfig, TextInputKind, spawn_text_input};
use crate::theme::ThemeResource;

#[derive(Component)]
pub struct SearchBox;

#[derive(Component)]
pub struct SearchBoxFilterButton;

pub fn spawn_searchbox(
    parent: &mut ChildSpawnerCommands,
    _font: Handle<Font>,
    theme: &ThemeResource,
    placeholder: impl Into<String>,
) -> Entity {
    let placeholder = placeholder.into();
    let colors = theme.current.colors;
    let is_dark = theme.current.mode == crate::theme::ThemeMode::Dark;
    let mut entity = None;

    parent
        .spawn((
            Node {
                width: percent(100),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(8.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|searchbox| {
            searchbox
                .spawn((
                    Button,
                    SearchBoxFilterButton,
                    crate::primitives::a11y::TabIndex(0),
                    crate::primitives::a11y::button_node("Filter search results"),
                    Node {
                        width: px(38.0),
                        height: px(38.0),
                        display: Display::Flex,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border: UiRect::all(px(1.0)),
                        border_radius: BorderRadius::all(px(12.0)),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    BorderColor::all(Color::NONE),
                    Surface::rounded_rect_fill(
                        12.0,
                        Paint::solid(colors.surface_elevated.with_alpha(if is_dark { 0.92 } else { 0.96 })),
                    )
                    .uniform_border(1.0, Paint::solid(colors.border.with_alpha(if is_dark { 0.84 } else { 0.70 }))),
                ))
                .with_children(|button| {
                    button.spawn((
                        IconNode::new(Icon::feather("filter"))
                            .size(15.0)
                            .color(colors.text_muted),
                        Node {
                            width: px(15.0),
                            height: px(15.0),
                            ..default()
                        },
                    ));
                });

            entity = Some(spawn_text_input(
                searchbox,
                TextInputConfig::new(placeholder.clone())
                    .kind(TextInputKind::Search)
                    .max_length(120)
                    .border_radius(10.0)
                    .width(Val::Auto)
                    .flex_grow(1.0)
                    .show_submit_button(false),
            ));
        });

    let entity = entity.expect("searchbox input should be created");

    parent.commands().entity(entity).insert(SearchBox);

    entity
}
