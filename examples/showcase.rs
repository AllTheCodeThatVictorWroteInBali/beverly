//! Showcase of Beverly's major components and design system running as a
//! standalone Bevy app (no dependency on theblocks_studio).

use beverly::components::checkbox::CheckboxState;
use beverly::components::radio::{RadioChanged, RadioGroupBuilder, spawn_radio_group};
use beverly::components::slider::{SliderStyle, spawn_slider};
use beverly::components::toggle::{ToggleConfig, ToggleShadowMaterial, spawn_toggle};
use beverly::prelude::*;
use bevy::prelude::*;

#[path = "support/mod.rs"]
mod demo_support;
use demo_support::DemoAppExt;

#[derive(Component)]
enum ProjectAction {
    Save,
    Publish,
}

#[derive(Component)]
struct ProjectName;

#[derive(Component)]
struct ReviewRequired;

#[derive(Component)]
struct AutoDeploy;

#[derive(Component)]
struct GalleryCard;

#[derive(Component)]
struct ProjectStatus;

#[derive(Component)]
struct Traffic;

#[derive(Component)]
struct TrafficReadout;

#[derive(Component)]
struct RegionReadout;

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (project_actions, update_delivery))
        .add_systems(PostUpdate, fit_card.before(bevy::ui::UiSystems::Prepare))
        .run_demo();
}

fn setup(
    mut commands: Commands,
    theme: Res<ThemeResource>,
    mut materials: ResMut<Assets<ToggleShadowMaterial>>,
) {
    spawn_themed_page(&mut commands, |root| {
        root.spawn(Node {
            width: Val::Percent(100.0),
            max_width: Val::Px(760.0),
            min_width: Val::Px(0.0),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        })
        .with_children(|content| {
            content.spawn((
                ThemedTitle::new(TitleLevel::H2),
                Text::new("Beverly workspace"),
            ));
            let card = spawn_card(
                content,
                CardStyle {
                    padding: 16.0,
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
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(6.0),
                            ..default()
                        },
                    ))
                    .with_children(|header| {
                        header.spawn((
                            ThemedTitle::new(TitleLevel::H4),
                            Text::new("Project overview"),
                        ));
                        header.spawn((
                            ThemedText::new(TextRole::Muted),
                            Text::new("Production workspace / Web team"),
                        ));
                    });
                    card.spawn((
                        CardBody,
                        Node {
                            min_width: Val::Px(0.0),
                            flex_shrink: 0.0,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(8.0),
                            ..default()
                        },
                    ))
                    .with_children(|body| {
                        body.spawn((ThemedText::new(TextRole::Label), Text::new("Project name")));
                        let input = spawn_text_input(
                            body,
                            TextInputConfig::new("Project name")
                                .label("Project name")
                                .initial_value("Website refresh"),
                        );
                        body.commands().entity(input).insert(ProjectName);
                        body.spawn((
                            ThemedText::new(TextRole::Label),
                            Text::new("Release contact"),
                        ));
                        spawn_text_input(
                            body,
                            TextInputConfig::new("team@example.com")
                                .label("Release contact")
                                .kind(TextInputKind::Email),
                        );
                    });
                    card.spawn((
                        CardFooter,
                        Node {
                            min_width: Val::Px(0.0),
                            flex_shrink: 0.0,
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: Val::Px(8.0),
                            row_gap: Val::Px(8.0),
                            ..default()
                        },
                    ))
                    .with_children(|footer| {
                        footer.spawn((ProjectAction::Save, BeverlyButton::standard("Save draft")));
                        footer.spawn((ProjectAction::Publish, BeverlyButton::primary("Publish")));
                        footer.spawn(BeverlyButton::danger("Archive").disabled(true));
                    });
                },
            );
            content.commands().entity(card).insert((
                GalleryCard,
                Node {
                    width: Val::Percent(100.0),
                    min_width: Val::Px(0.0),
                    flex_shrink: 0.0,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(16.0),
                    padding: UiRect::all(Val::Px(16.0)),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    ..default()
                },
            ));
            content.spawn((
                ProjectStatus,
                ThemedText::new(TextRole::Body),
                Text::new("Draft / No changes saved"),
            ));
            content
                .spawn(Node {
                    min_width: Val::Px(0.0),
                    flex_shrink: 0.0,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(12.0),
                    ..default()
                })
                .with_children(|preferences| {
                    preferences.spawn((
                        ThemedTitle::new(TitleLevel::H4),
                        Text::new("Release preferences"),
                    ));
                    let checkbox = spawn_checkbox(
                        preferences,
                        CheckboxConfig::new()
                            .label("Require review before release")
                            .checked(true),
                    );
                    preferences
                        .commands()
                        .entity(checkbox)
                        .insert(ReviewRequired);
                    let toggle = spawn_toggle(
                        preferences,
                        ToggleConfig::new().label("Deploy automatically"),
                        &theme,
                        &mut materials,
                    );
                    preferences.commands().entity(toggle).insert(AutoDeploy);
                });
            content
                .spawn(Node {
                    min_width: Val::Px(0.0),
                    flex_shrink: 0.0,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(12.0),
                    ..default()
                })
                .with_children(|delivery| {
                    delivery.spawn((ThemedTitle::new(TitleLevel::H4), Text::new("Delivery")));
                    spawn_radio_group(
                        delivery,
                        RadioGroupBuilder::new("release-region")
                            .options([
                                ("global", "Global"),
                                ("europe", "Europe"),
                                ("americas", "Americas"),
                            ])
                            .selected("global")
                            .width(Val::Percent(100.0))
                            .spacing(Val::Px(6.0)),
                    );
                    delivery.spawn((
                        RegionReadout,
                        ThemedText::new(TextRole::Muted),
                        Text::new("Region: Global"),
                    ));
                    delivery.spawn((
                        TrafficReadout,
                        ThemedText::new(TextRole::Label),
                        Text::new("Traffic allocation: 50%"),
                    ));
                    let slider = spawn_slider(
                        delivery,
                        Slider::new(0.0, 100.0)
                            .value(50.0)
                            .step(5.0)
                            .label("Traffic allocation"),
                        SliderStyle::default(),
                        &theme,
                    );
                    delivery.commands().entity(slider).insert(Traffic).queue(
                        |mut entity: EntityWorldMut| {
                            if let Some(mut node) = entity.get_mut::<Node>() {
                                node.width = Val::Percent(100.0);
                                node.min_width = Val::Px(0.0);
                                node.flex_shrink = 0.0;
                            }
                        },
                    );
                });
        });
    });
}

fn fit_card(mut cards: Query<&mut Node, With<GalleryCard>>) {
    for mut node in &mut cards {
        node.width = Val::Percent(100.0);
        node.height = Val::Auto;
    }
}

fn project_actions(
    buttons: Query<(&Interaction, &ProjectAction), Changed<Interaction>>,
    names: Query<&TextInput, With<ProjectName>>,
    reviews: Query<&CheckboxState, With<ReviewRequired>>,
    deployments: Query<&Toggle, With<AutoDeploy>>,
    mut statuses: Query<&mut Text, With<ProjectStatus>>,
) {
    for (interaction, action) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let name = names
            .iter()
            .next()
            .map(|input| input.value.trim())
            .unwrap_or("");
        let message = if name.is_empty() {
            "Enter a project name first.".to_owned()
        } else {
            let state = match action {
                ProjectAction::Save => "Draft saved locally",
                ProjectAction::Publish if reviews.iter().any(|review| review.checked) => {
                    "Queued for review"
                }
                ProjectAction::Publish
                    if deployments.iter().any(|deployment| deployment.checked) =>
                {
                    "Deployment queued"
                }
                ProjectAction::Publish => "Ready for manual deployment",
            };
            format!("{name} / {state}")
        };
        for mut status in &mut statuses {
            status.0 = message.clone();
        }
    }
}

fn update_delivery(
    mut regions: MessageReader<RadioChanged>,
    sliders: Query<&Slider, With<Traffic>>,
    mut traffic_labels: Query<&mut Text, (With<TrafficReadout>, Without<RegionReadout>)>,
    mut region_labels: Query<&mut Text, (With<RegionReadout>, Without<TrafficReadout>)>,
) {
    for change in regions.read() {
        let region = match change.value.as_str() {
            "europe" => "Europe",
            "americas" => "Americas",
            _ => "Global",
        };
        for mut label in &mut region_labels {
            label.0 = format!("Region: {region}");
        }
    }
    for slider in &sliders {
        let value = format!("Traffic allocation: {:.0}%", slider.value);
        for mut label in &mut traffic_labels {
            if label.0 != value {
                label.0 = value.clone();
            }
        }
    }
}
