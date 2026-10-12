//! Minimal example demonstrating Beverly cards.

use beverly::prelude::*;
use bevy::prelude::*;

#[path = "support/mod.rs"]
mod demo_support;
use demo_support::DemoAppExt;

#[derive(Component)]
struct PublishButton;

#[derive(Component)]
struct PublishState;

#[derive(Component)]
struct ResponsiveCard;

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, publish)
        .add_systems(PostUpdate, fit_cards.before(bevy::ui::UiSystems::Prepare))
        .run_demo();
}

fn setup(mut commands: Commands, theme: Res<ThemeResource>) {
    spawn_themed_page(&mut commands, |root| {
        root.spawn(Node {
            width: Val::Percent(100.0),
            max_width: Val::Px(760.0),
            min_width: Val::Px(0.0),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(16.0),
            ..default()
        })
        .with_children(|content| {
            content.spawn((ThemedTitle::new(TitleLevel::H2), Text::new("Project cards")));
            for (title, body, state) in [
                (
                    "Website refresh",
                    "Updated navigation and a clearer project overview are ready for review.",
                    "Draft",
                ),
                (
                    "Design library",
                    "Shared colors, typography and controls are available to the whole team.",
                    "Published",
                ),
                (
                    "Release notes",
                    "The next update includes keyboard navigation and improved mobile layouts.",
                    "In review",
                ),
            ] {
                let card = spawn_card(
                    content,
                    CardStyle {
                        padding: 0.0,
                        border_radius: 8.0,
                        ..default()
                    },
                    &theme,
                    |card| {
                        card.spawn((
                            CardHeader,
                            Node {
                                min_width: Val::Px(0.0),
                                flex_shrink: 0.0,
                                padding: UiRect::all(Val::Px(16.0)),
                                ..default()
                            },
                        ))
                        .with_children(|header| {
                            header.spawn((ThemedTitle::new(TitleLevel::H4), Text::new(title)));
                        });
                        card.spawn((
                            CardBody,
                            Node {
                                min_width: Val::Px(0.0),
                                flex_shrink: 0.0,
                                padding: UiRect::horizontal(Val::Px(16.0)),
                                ..default()
                            },
                        ))
                        .with_children(|section| {
                            section.spawn((ThemedText::new(TextRole::Body), Text::new(body)));
                        });
                        card.spawn((
                            CardFooter,
                            Node {
                                min_width: Val::Px(0.0),
                                flex_shrink: 0.0,
                                flex_wrap: FlexWrap::Wrap,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(12.0),
                                row_gap: Val::Px(8.0),
                                padding: UiRect::all(Val::Px(16.0)),
                                ..default()
                            },
                        ))
                        .with_children(|footer| {
                            let label = footer
                                .spawn((ThemedText::new(TextRole::Muted), Text::new(state)))
                                .id();
                            if state == "Draft" {
                                footer.commands().entity(label).insert(PublishState);
                                footer.spawn((
                                    PublishButton,
                                    BeverlyButton::primary("Publish").sized(ButtonSize::Sm),
                                ));
                            }
                        });
                    },
                );
                content.commands().entity(card).insert((
                    ResponsiveCard,
                    Node {
                        width: Val::Percent(100.0),
                        min_width: Val::Px(0.0),
                        flex_shrink: 0.0,
                        flex_direction: FlexDirection::Column,
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                ));
            }
        });
    });
}

fn fit_cards(mut cards: Query<&mut Node, With<ResponsiveCard>>) {
    for mut node in &mut cards {
        node.width = Val::Percent(100.0);
        node.height = Val::Auto;
    }
}

fn publish(
    mut buttons: Query<
        (&Interaction, &mut BeverlyButton),
        (Changed<Interaction>, With<PublishButton>),
    >,
    mut labels: Query<&mut Text, With<PublishState>>,
) {
    for (interaction, mut button) in &mut buttons {
        if *interaction == Interaction::Pressed && !button.disabled {
            button.disabled = true;
            button.label = "Published".into();
            for mut label in &mut labels {
                label.0 = "Published".into();
            }
        }
    }
}
