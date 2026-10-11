//! Noise and texture demo: the snack desk's toasted-grain laboratory.
//! Run with `-- dark` to start in dark mode.

use beverly::components::button::BeverlyButton;
use beverly::components::slider::{Slider, SliderStyle, spawn_slider};
use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::prelude::*;
use beverly::rendering::{
    Backdrop, BackdropQuality, GradientStop, LinearGradient, Noise, NoiseTarget, Paint, Surface,
};
use beverly::theme::AccessibilityVisualPolicyResource;
use bevy::prelude::*;

const BORDER: Color = Color::srgb(0.18, 0.55, 0.94);
const TARGETS: [(&str, NoiseTarget); 3] = [
    ("Surface", NoiseTarget::Surface),
    ("Fill", NoiseTarget::Fill),
    ("Border", NoiseTarget::Border),
];

#[derive(Resource)]
struct NoiseSettings {
    /// Grain size in pixels, strength in percent, seed, animation speed.
    values: [f32; 4],
    target: NoiseTarget,
    animated: bool,
}

impl Default for NoiseSettings {
    fn default() -> Self {
        Self {
            values: [8.0, 12.0, 0.0, 2.0],
            target: NoiseTarget::Surface,
            animated: false,
        }
    }
}

#[derive(Component)]
struct NoiseControl(usize);

#[derive(Component)]
struct TargetButton(NoiseTarget);

#[derive(Component)]
struct MotionButton(bool);

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
enum NoisePreview {
    Main,
    Light,
    Mid,
    Black,
    Frosted,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .init_resource::<NoiseSettings>()
        .add_systems(Startup, setup)
        .add_systems(Update, (update_settings, update_previews).chain())
        .run();
}

fn setup(mut commands: Commands, theme: Res<ThemeResource>) {
    spawn_themed_page(&mut commands, |root| {
        root.spawn(Node {
            width: Val::Px(760.0),
            max_width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(16.0),
            ..default()
        })
        .with_children(|content| {
            content.spawn((
                ThemedTitle::new(TitleLevel::H2),
                Text::new("The Toasted Grain Laboratory"),
            ));
            content.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("A little texture keeps the biscuit from looking too perfect."),
            ));
            content
                .spawn(Node {
                    column_gap: Val::Px(24.0),
                    row_gap: Val::Px(10.0),
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                })
                .with_children(|controls| {
                    for (index, (label, min, max, value)) in [
                        ("Grain size (px)", 1.0, 64.0, 8.0),
                        ("Strength (%)", 0.0, 100.0, 12.0),
                        ("Seed", 0.0, 20.0, 0.0),
                        ("Animation speed", 0.0, 16.0, 2.0),
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        controls
                            .spawn(Node {
                                width: Val::Px(232.0),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(4.0),
                                ..default()
                            })
                            .with_children(|row| {
                                row.spawn((ThemedText::new(TextRole::Label), Text::new(label)));
                                let slider = spawn_slider(
                                    row,
                                    Slider::new(min, max).value(value).step(1.0).label(label),
                                    SliderStyle {
                                        width: 232.0,
                                        show_value: true,
                                        track_color: theme.current.colors.border,
                                        fill_color: theme.current.colors.text,
                                        thumb_color: theme.current.colors.surface_elevated,
                                        ..default()
                                    },
                                    &theme,
                                );
                                row.commands().entity(slider).insert(NoiseControl(index));
                            });
                    }
                });
            content
                .spawn(Node {
                    column_gap: Val::Px(8.0),
                    row_gap: Val::Px(8.0),
                    flex_wrap: FlexWrap::Wrap,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|buttons| {
                    buttons.spawn((ThemedText::new(TextRole::Label), Text::new("Apply to")));
                    for (label, target) in TARGETS {
                        buttons.spawn((
                            TargetButton(target),
                            BeverlyButton::standard(label).on("click", select_target),
                        ));
                    }
                    buttons.spawn((
                        Node {
                            margin: UiRect::left(Val::Px(16.0)),
                            ..default()
                        },
                        ThemedText::new(TextRole::Label),
                        Text::new("Motion"),
                    ));
                    for (label, animated) in [("Still", false), ("Animated", true)] {
                        buttons.spawn((
                            MotionButton(animated),
                            BeverlyButton::standard(label).on("click", select_motion),
                        ));
                    }
                });
            content
                .spawn(Node {
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(24.0),
                    row_gap: Val::Px(18.0),
                    ..default()
                })
                .with_children(spawn_previews);
        });
    });
}

fn spawn_previews(previews: &mut ChildSpawnerCommands) {
    let noise = configured_noise(&NoiseSettings::default(), true);
    previews
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|sample| {
            sample.spawn((
                ThemedText::new(TextRole::Label),
                Text::new("Gradient + border"),
            ));
            sample.spawn((
                NoisePreview::Main,
                Node {
                    width: Val::Px(300.0),
                    height: Val::Px(170.0),
                    ..default()
                },
                preview_surface(NoisePreview::Main, noise),
            ));
        });
    previews
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|sample| {
            sample.spawn((ThemedText::new(TextRole::Label), Text::new("Flat fills")));
            sample
                .spawn(Node {
                    column_gap: Val::Px(12.0),
                    ..default()
                })
                .with_children(|row| {
                    for (kind, caption) in [
                        (NoisePreview::Light, "Light"),
                        (NoisePreview::Mid, "Mid"),
                        (NoisePreview::Black, "Black"),
                    ] {
                        row.spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(6.0),
                            ..default()
                        })
                        .with_children(|column| {
                            column.spawn((
                                kind,
                                Node {
                                    width: Val::Px(84.0),
                                    height: Val::Px(84.0),
                                    ..default()
                                },
                                preview_surface(kind, noise),
                            ));
                            column.spawn((ThemedText::new(TextRole::Caption), Text::new(caption)));
                        });
                    }
                });
            sample.spawn((
                ThemedText::new(TextRole::Caption),
                Text::new("Noise scales brightness, so black stays flat."),
            ));
        });
    previews
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|sample| {
            sample.spawn((ThemedText::new(TextRole::Label), Text::new("Frosted glass")));
            sample
                .spawn(Node {
                    width: Val::Px(300.0),
                    height: Val::Px(170.0),
                    position_type: PositionType::Relative,
                    overflow: Overflow::clip(),
                    ..default()
                })
                .with_children(|scene| {
                    let colors = [
                        Color::srgb(0.98, 0.76, 0.18),
                        Color::srgb(0.94, 0.26, 0.32),
                        Color::srgb(0.12, 0.72, 0.50),
                        Color::srgb(0.18, 0.55, 0.94),
                    ];
                    for index in 0..8 {
                        scene.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(index as f32 * 40.0),
                                width: Val::Px(40.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            Surface::rounded_rect_fill(
                                0.0,
                                Paint::solid(colors[index % colors.len()]),
                            ),
                        ));
                    }
                    scene
                        .spawn((
                            NoisePreview::Frosted,
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(40.0),
                                top: Val::Px(40.0),
                                width: Val::Px(220.0),
                                height: Val::Px(90.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            preview_surface(NoisePreview::Frosted, noise),
                        ))
                        .with_children(|window| {
                            window.spawn((
                                ThemedText::new(TextRole::Label).color(Color::WHITE),
                                Text::new("Frosted"),
                            ));
                        });
                });
            sample.spawn((
                ThemedText::new(TextRole::Caption),
                Text::new("The Surface target also grains the blurred backdrop."),
            ));
        });
}

fn configured_noise(settings: &NoiseSettings, motion_allowed: bool) -> Noise {
    Noise::grain(settings.values[0], settings.values[1] / 100.0)
        .with_seed(settings.values[2])
        .with_animated(settings.animated && motion_allowed)
        .with_speed(settings.values[3])
        .with_target(settings.target)
}

fn preview_surface(kind: NoisePreview, noise: Noise) -> Surface {
    let flat = |grey: f32| {
        Surface::rounded_rect_fill(8.0, Paint::solid(Color::srgb(grey, grey, grey)))
            .uniform_border(6.0, Paint::solid(BORDER))
    };
    let surface = match kind {
        NoisePreview::Main => Surface::rounded_rect_fill(
            8.0,
            Paint::linear(LinearGradient::angle_degrees(
                45.0,
                vec![
                    GradientStop::new(0.0, Color::srgb(0.98, 0.76, 0.18)),
                    GradientStop::new(1.0, Color::srgb(0.94, 0.26, 0.32)),
                ],
            )),
        )
        .uniform_border(10.0, Paint::solid(BORDER)),
        NoisePreview::Light => flat(0.92),
        NoisePreview::Mid => flat(0.5),
        NoisePreview::Black => flat(0.0),
        NoisePreview::Frosted => Surface::rounded_rect_fill(8.0, Paint::solid(Color::NONE))
            .uniform_border(1.0, Paint::solid(Color::WHITE.with_alpha(0.6)))
            .with_backdrop(
                Backdrop::new()
                    .with_blur(10.0)
                    .with_tint(Color::WHITE)
                    .with_tint_opacity(0.15)
                    .with_quality(BackdropQuality::High),
            ),
    };
    surface.with_noise(noise)
}

fn select_target(commands: &mut Commands, button: Entity) {
    commands.queue(move |world: &mut World| {
        if let Some(target) = world.get::<TargetButton>(button).map(|button| button.0) {
            world.resource_mut::<NoiseSettings>().target = target;
        }
    });
}

fn select_motion(commands: &mut Commands, button: Entity) {
    commands.queue(move |world: &mut World| {
        if let Some(animated) = world.get::<MotionButton>(button).map(|button| button.0) {
            world.resource_mut::<NoiseSettings>().animated = animated;
        }
    });
}

fn update_settings(controls: Query<(&NoiseControl, &Slider)>, mut settings: ResMut<NoiseSettings>) {
    for (control, slider) in &controls {
        if settings.values[control.0] != slider.value {
            settings.values[control.0] = slider.value;
        }
    }
}

fn update_previews(
    settings: Res<NoiseSettings>,
    policy: Res<AccessibilityVisualPolicyResource>,
    mut previews: Query<(&NoisePreview, &mut Surface)>,
    mut targets: Query<(&TargetButton, &mut BeverlyButton), Without<MotionButton>>,
    mut motions: Query<(&MotionButton, &mut BeverlyButton), Without<TargetButton>>,
) {
    let noise = Some(configured_noise(&settings, !policy.current.reduced_motion));
    for (_, mut surface) in &mut previews {
        if surface.noise != noise {
            surface.noise = noise;
        }
    }
    for (button, mut view) in &mut targets {
        let outline = button.0 != settings.target;
        if view.outline != outline {
            view.outline = outline;
        }
    }
    for (button, mut view) in &mut motions {
        let outline = button.0 != settings.animated;
        if view.outline != outline {
            view.outline = outline;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controls_map_to_noise_and_motion_respects_the_policy() {
        let settings = NoiseSettings {
            values: [20.0, 35.0, 7.0, 4.0],
            target: NoiseTarget::Border,
            animated: true,
        };
        let noise = configured_noise(&settings, true);
        assert_eq!(noise.scale, 20.0);
        assert_eq!(noise.strength, 0.35);
        assert_eq!(noise.seed, 7.0);
        assert_eq!(noise.speed, 4.0);
        assert!(noise.animated);
        assert_eq!(noise.target, NoiseTarget::Border);
        assert_eq!(noise, noise.sanitized());
        assert!(!configured_noise(&settings, false).animated);
    }

    #[test]
    fn every_preview_follows_the_shared_noise_and_black_fill_stays_black() {
        let mut app = App::new();
        app.init_resource::<AccessibilityVisualPolicyResource>()
            .insert_resource(NoiseSettings {
                values: [12.0, 50.0, 3.0, 2.0],
                target: NoiseTarget::Fill,
                animated: false,
            })
            .add_systems(Update, update_previews);
        let kinds = [
            NoisePreview::Main,
            NoisePreview::Light,
            NoisePreview::Mid,
            NoisePreview::Black,
            NoisePreview::Frosted,
        ];
        for kind in kinds {
            app.world_mut()
                .spawn((kind, preview_surface(kind, Noise::default())));
        }
        app.update();
        let mut previews = app.world_mut().query::<(&NoisePreview, &Surface)>();
        let expected = Some(configured_noise(
            app.world().resource::<NoiseSettings>(),
            true,
        ));
        assert_eq!(previews.iter(app.world()).count(), kinds.len());
        for (kind, surface) in previews.iter(app.world()) {
            assert_eq!(surface.noise, expected, "{kind:?}");
        }
        let black = previews
            .iter(app.world())
            .find(|(kind, _)| **kind == NoisePreview::Black)
            .unwrap()
            .1;
        assert_eq!(
            black.fill,
            Paint::solid(Color::srgb(0.0, 0.0, 0.0)),
            "the black swatch keeps a pure black fill"
        );
    }

    #[test]
    fn buttons_select_target_and_motion() {
        let mut world = World::new();
        world.insert_resource(NoiseSettings::default());
        let target = world.spawn(TargetButton(NoiseTarget::Border)).id();
        let motion = world.spawn(MotionButton(true)).id();
        select_target(&mut world.commands(), target);
        select_motion(&mut world.commands(), motion);
        world.flush();
        let settings = world.resource::<NoiseSettings>();
        assert_eq!(settings.target, NoiseTarget::Border);
        assert!(settings.animated);
    }
}
