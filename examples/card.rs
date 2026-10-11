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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, sync_demo_theme)
        .run();
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

        spawn_card(root, style, &theme, |card| {
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
                    column_gap: Val::Px(8.0),
                    padding: UiRect::all(Val::Px(16.0)),
                    ..default()
                },
            ))
            .with_children(|footer| {
                footer.spawn(BeverlyButton::standard("Cancel").sized(ButtonSize::Sm));
                footer.spawn(BeverlyButton::primary("Deploy").sized(ButtonSize::Sm));
            });
        });
    });
}
