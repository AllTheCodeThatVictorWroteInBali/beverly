//! Card demo: a card with a header, body and footer.
//! Run with `-- dark` to start in dark mode; the top-right button toggles light/dark.

use beverly::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
struct DemoDivider;

#[derive(Component)]
struct DemoTitle;

#[derive(Component)]
struct DemoBody;

#[derive(Component)]
struct DemoCard;

#[path = "support/mod.rs"]
mod demo_support;
use demo_support::DemoAppExt;

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, sync_demo_theme)
        .add_systems(
            PostUpdate,
            fit_demo_card.before(bevy::ui::UiSystems::Prepare),
        )
        .run_demo();
}

fn fit_demo_card(
    mut cards: Query<&mut Node, With<DemoCard>>,
    sections: Query<
        (&Node, &Children),
        (
            Or<(With<CardHeader>, With<CardBody>, With<CardFooter>)>,
            Without<DemoCard>,
        ),
    >,
    computed: Query<&ComputedNode>,
) {
    let mut content_height = 2.0;
    for (node, children) in &sections {
        let heights: Vec<_> = children
            .iter()
            .filter_map(|child| {
                computed
                    .get(child)
                    .ok()
                    .map(|node| node.size().y * node.inverse_scale_factor())
            })
            .collect();
        let padding = [node.padding.top, node.padding.bottom]
            .into_iter()
            .filter_map(|value| {
                if let Val::Px(value) = value {
                    Some(value)
                } else {
                    None
                }
            })
            .sum::<f32>();
        content_height += padding
            + if node.flex_direction == FlexDirection::Column {
                let gap = if let Val::Px(value) = node.row_gap {
                    value
                } else {
                    0.0
                };
                heights.iter().sum::<f32>() + gap * heights.len().saturating_sub(1) as f32
            } else {
                heights.into_iter().fold(0.0, f32::max)
            };
    }
    for mut node in &mut cards {
        node.min_height = Val::Px(content_height.max(260.0));
    }
}

/// Re-applies the colors the example sets by hand whenever the theme changes.
fn sync_demo_theme(
    theme: Res<ThemeResource>,
    mut dividers: Query<&mut BackgroundColor, With<DemoDivider>>,
    mut titles: Query<&mut ThemedTitle, With<DemoTitle>>,
    mut bodies: Query<&mut ThemedText, With<DemoBody>>,
) {
    if !theme.is_changed() {
        return;
    }

    let colors = theme.current.colors;
    // White title in dark mode (black in light mode); grey body text in both.
    let (title_color, body_color) = match theme.current.mode {
        ThemeMode::Dark => (Color::WHITE, Color::srgb_u8(163, 163, 163)),
        ThemeMode::Light => (Color::BLACK, Color::srgb_u8(82, 82, 82)),
    };

    for mut divider in &mut dividers {
        divider.0 = colors.border;
    }
    for mut title in &mut titles {
        title.color_override = Some(title_color);
    }
    for mut body in &mut bodies {
        body.color_override = Some(body_color);
    }
}

fn setup(mut commands: Commands, theme: Res<ThemeResource>) {
    spawn_themed_page(&mut commands, |root| {
        let style = CardStyle {
            width: 380.0,
            height: 260.0,
            padding: 0.0,
            ..default()
        };

        let card = spawn_card(root, style, &theme, |card| {
            let divider = || {
                (
                    DemoDivider,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(1.0),
                        ..default()
                    },
                    BackgroundColor(theme.current.colors.border),
                )
            };

            card.spawn((
                CardHeader,
                Node {
                    flex_direction: FlexDirection::Column,
                    min_width: Val::Px(0.0),
                    flex_shrink: 0.0,
                    row_gap: Val::Px(4.0),
                    padding: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
            ))
            .with_children(|header| {
                header.spawn((
                    DemoTitle,
                    ThemedTitle::new(TitleLevel::H4),
                    Text::new("Deploy project"),
                ));
                header.spawn((
                    ThemedText::new(TextRole::Muted),
                    Text::new("Ship your latest changes to production."),
                ));
            });

            card.spawn(divider());

            card.spawn((
                    CardBody,
                    Node {
                        flex_grow: 1.0,
                        min_width: Val::Px(0.0),
                        flex_shrink: 0.0,
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                ))
                .with_children(|body| {
                    body.spawn((
                        DemoBody,
                        ThemedText::new(TextRole::Body),
                        Text::new("Your project will be built and deployed to all regions. This usually takes about a minute."),
                    ));
                });

            card.spawn(divider());

            card.spawn((
                CardFooter,
                Node {
                    justify_content: JustifyContent::FlexStart,
                    min_width: Val::Px(0.0),
                    flex_shrink: 0.0,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(8.0),
                    row_gap: Val::Px(8.0),
                    padding: UiRect::all(Val::Px(16.0)),
                    ..default()
                },
            ))
            .with_children(|footer| {
                footer.spawn(BeverlyButton::standard("Cancel").sized(ButtonSize::Sm));
                footer.spawn(BeverlyButton::primary("Deploy").sized(ButtonSize::Sm));
            });
        });
        root.commands().entity(card).insert((
            DemoCard,
            Node {
                width: Val::Px(style.width),
                max_width: Val::Percent(100.0),
                min_width: Val::Px(0.0),
                height: Val::Px(style.height),
                min_height: Val::Px(260.0),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Column,
                border_radius: BorderRadius::all(Val::Px(style.border_radius)),
                ..default()
            },
        ));
    });
}
