use bevy::prelude::*;

use crate::rendering::{GradientStop, LinearGradient as UiLinearGradient, Paint, Surface};
use crate::icons::{Icon, IconCommands};
use crate::components::text::HighlightableText;
use crate::theme::ThemeResource;

#[derive(Component)]
pub struct ListItem;

#[derive(Component)]
#[require(HighlightableText)]
pub struct ListItemTitle;

#[derive(Component)]
#[require(HighlightableText)]
pub struct ListItemSubtitle;

#[derive(Clone, Copy)]
pub struct ListItemStyle {
    pub height: f32,
    pub padding: f32,
    pub border_radius: f32,
}

impl Default for ListItemStyle {
    fn default() -> Self {
        Self {
            height: 70.0,
            padding: 18.0,
            border_radius: 14.0,
        }
    }
}

pub struct ListItemPlugin;

impl Plugin for ListItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, list_item_visual_system);
    }
}

pub fn spawn_list_item(
    parent: &mut ChildSpawnerCommands,
    title: impl Into<String>,
    subtitle: Option<String>,
    icon: Option<Icon>,
) -> Entity {
    let style = ListItemStyle::default();

    parent
        .spawn((
            Button,
            ListItem,
            Node {
                width: percent(100),
                height: px(style.height),
                padding: UiRect::horizontal(px(style.padding)),
                border: UiRect::all(px(1.0)),
                border_radius: BorderRadius::all(px(style.border_radius)),
                align_items: AlignItems::Center,
                column_gap: px(14.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderColor::all(Color::NONE),
            Surface::rounded_rect_fill(
                style.border_radius,
                Paint::linear(UiLinearGradient::vertical(vec![
                    GradientStop::new(0.0, Color::srgb(0.12, 0.12, 0.15)),
                    GradientStop::new(1.0, Color::srgb(0.15, 0.15, 0.19)),
                ])),
            )
            .uniform_border(1.0, Paint::solid(Color::srgba(1.0, 1.0, 1.0, 0.18))),
        ))
        .with_children(|item| {
            if let Some(icon) = icon {
                item.spawn(Node {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|icon_node| {
                    icon_node.spawn_icon_sized(icon, 20.0);
                });
            }

            item.spawn(Node {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                flex_grow: 1.0,
                ..default()
            })
            .with_children(|text| {
                text.spawn((
                    ListItemTitle,
                    Text::new(title.into()),
                    TextFont {
                        font_size: FontSize::Px(18.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.10, 0.12, 0.16)),
                ));

                if let Some(subtitle) = subtitle {
                    text.spawn((
                        ListItemSubtitle,
                        Text::new(subtitle),
                        TextFont {
                            font_size: FontSize::Px(14.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.42, 0.45, 0.50)),
                    ));
                }
            });

            item.spawn(Node {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|icon_node| {
                icon_node.spawn_icon_muted(Icon::feather("chevron-right"), 18.0);
            });
        })
        .id()
}

fn list_item_visual_system(
    theme: Res<ThemeResource>,
    mut item_query: Query<
        (&Interaction, &mut Surface),
        (With<ListItem>, With<Button>),
    >,
    mut text_query: Query<
        (
            &mut TextColor,
            Option<&ListItemTitle>,
            Option<&ListItemSubtitle>,
        ),
        Or<(With<ListItemTitle>, With<ListItemSubtitle>)>,
    >,
) {
    let colors = theme.current.colors;

    for (interaction, mut surface) in &mut item_query {
        match *interaction {
            Interaction::Pressed => {
                surface.fill = Paint::linear(UiLinearGradient::vertical(vec![
                    GradientStop::new(0.0, colors.primary.with_alpha(0.92)),
                    GradientStop::new(1.0, colors.primary_active.with_alpha(0.90)),
                ]));
                if let Some(border) = surface.border.as_mut() {
                    border.paint = Paint::solid(colors.primary_active.with_alpha(0.90));
                }
            }
            Interaction::Hovered => {
                surface.fill = Paint::linear(UiLinearGradient::vertical(vec![
                    GradientStop::new(0.0, colors.surface_elevated.with_alpha(0.98)),
                    GradientStop::new(1.0, colors.surface.with_alpha(0.92)),
                ]));
                if let Some(border) = surface.border.as_mut() {
                    border.paint = Paint::solid(colors.border_strong.with_alpha(0.58));
                }
            }
            Interaction::None => {
                surface.fill = Paint::linear(UiLinearGradient::vertical(vec![
                    GradientStop::new(0.0, colors.surface_elevated.with_alpha(0.88)),
                    GradientStop::new(1.0, colors.surface.with_alpha(0.80)),
                ]));
                if let Some(border) = surface.border.as_mut() {
                    border.paint = Paint::solid(colors.border.with_alpha(0.32));
                }
            }
        }
    }

    for (mut text_color, title, subtitle) in &mut text_query {
        if title.is_some() {
            text_color.0 = colors.text;
        } else if subtitle.is_some() {
            text_color.0 = colors.text_muted;
        }
    }
}
