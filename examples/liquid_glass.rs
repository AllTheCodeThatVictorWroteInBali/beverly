//! Liquid glass demo: the snack desk's magnifying biscuit lens.
//! Run with `-- dark` to start in dark mode.

use beverly::components::button::BeverlyButton;
use beverly::components::slider::{Slider, SliderStyle, spawn_slider};
use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::prelude::*;
use beverly::rendering::{Backdrop, BackdropQuality, GlassProfile, LiquidGlass, Paint, Surface};
use bevy::prelude::*;

const STRIPE_WIDTH: f32 = 64.0;
const PROFILES: [(&str, GlassProfile); 4] = [
    ("Convex", GlassProfile::Convex),
    ("Squircle", GlassProfile::Squircle),
    ("Concave", GlassProfile::Concave),
    ("Lip", GlassProfile::Lip),
];

#[derive(Resource)]
struct GlassSettings {
    values: [f32; 5],
    profile: GlassProfile,
}

impl Default for GlassSettings {
    fn default() -> Self {
        Self {
            values: [8.0, 16.0, 146.0, 28.0, 18.0],
            profile: GlassProfile::Squircle,
        }
    }
}

#[derive(Component)]
struct GlassControl(usize);

#[derive(Component)]
struct ProfileButton(GlassProfile);

#[derive(Component)]
struct GlassLens;

#[derive(Component)]
struct MovingStripe(usize);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .init_resource::<GlassSettings>()
        .add_systems(Startup, setup)
        .add_systems(Update, (update_controls, update_scene).chain())
        .run();
}

fn setup(mut commands: Commands, theme: Res<ThemeResource>) {
    spawn_themed_page(&mut commands, |root| {
        root.spawn(Node {
            width: Val::Px(760.0),
            max_width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(18.0),
            ..default()
        })
        .with_children(|content| {
            content.spawn((
                ThemedTitle::new(TitleLevel::H2),
                Text::new("The Biscuit Magnification Bureau"),
            ));
            content.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("Inspecting the crumbs, one refracted pixel at a time."),
            ));
            content
                .spawn(Node {
                    column_gap: Val::Px(20.0),
                    row_gap: Val::Px(10.0),
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                })
                .with_children(|controls| {
                    for (index, (label, min, max, value)) in [
                        ("Thickness (px)", 0.0, 24.0, 8.0),
                        ("Bezel width (px)", 1.0, 48.0, 16.0),
                        ("Refraction (%)", 100.0, 180.0, 146.0),
                        ("Specular (%)", 0.0, 60.0, 28.0),
                        ("Color fringe (0.001)", 0.0, 80.0, 18.0),
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
                                row.commands().entity(slider).insert(GlassControl(index));
                            });
                    }
                });
            content
                .spawn(Node {
                    column_gap: Val::Px(8.0),
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                })
                .with_children(|modes| {
                    for (label, profile) in PROFILES {
                        modes.spawn((
                            ProfileButton(profile),
                            BeverlyButton::standard(label).on("click", select_profile),
                        ));
                    }
                });
            content
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(320.0),
                    position_type: PositionType::Relative,
                    overflow: Overflow::clip(),
                    ..default()
                })
                .with_children(spawn_scene);
        });
    });
}

fn spawn_scene(scene: &mut ChildSpawnerCommands) {
    scene.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        Surface::rounded_rect_fill(0.0, Paint::solid(Color::srgb(0.08, 0.23, 0.25))),
    ));
    let colors = [
        Color::srgb(0.98, 0.76, 0.18),
        Color::srgb(0.94, 0.26, 0.32),
        Color::srgb(0.12, 0.72, 0.50),
        Color::srgb(0.18, 0.55, 0.94),
    ];
    for index in 0..14 {
        scene.spawn((
            MovingStripe(index),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(index as f32 * STRIPE_WIDTH - STRIPE_WIDTH),
                width: Val::Px(STRIPE_WIDTH),
                height: Val::Percent(100.0),
                ..default()
            },
            Surface::rounded_rect_fill(0.0, Paint::solid(colors[index % colors.len()])),
        ));
    }
    // Fine lines and large text give the lens straight edges to bend.
    for index in 0..8 {
        scene.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(index as f32 * 40.0 + 20.0),
                width: Val::Percent(100.0),
                height: Val::Px(2.0),
                ..default()
            },
            Surface::rounded_rect_fill(0.0, Paint::solid(Color::WHITE.with_alpha(0.55))),
        ));
    }
    scene.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(24.0),
            top: Val::Px(24.0),
            ..default()
        },
        ThemedText::new(TextRole::Body)
            .size(28.0)
            .color(Color::WHITE),
        Text::new("BISCUITS / TEA / TOAST"),
    ));
    for (left, top, width, height, radius) in [
        (60.0, 90.0, 300.0, 140.0, 70.0),
        (470.0, 80.0, 160.0, 160.0, 80.0),
    ] {
        scene.spawn((
            GlassLens,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(left),
                top: Val::Px(top),
                width: Val::Px(width),
                height: Val::Px(height),
                ..default()
            },
            Surface::rounded_rect_fill(radius, Paint::solid(Color::NONE))
                .uniform_border(1.0, Paint::solid(Color::WHITE.with_alpha(0.5)))
                .with_backdrop(configured_backdrop(&GlassSettings::default())),
        ));
    }
}

fn configured_glass(settings: &GlassSettings) -> LiquidGlass {
    LiquidGlass {
        thickness: settings.values[0],
        bezel_width: settings.values[1],
        refractive_index: settings.values[2] / 100.0,
        specular_intensity: settings.values[3] / 100.0,
        chromatic_aberration: settings.values[4] / 1000.0,
        profile: settings.profile,
        ..default()
    }
}

fn configured_backdrop(settings: &GlassSettings) -> Backdrop {
    Backdrop::new()
        .with_blur(2.0)
        .with_tint(Color::WHITE)
        .with_tint_opacity(0.06)
        .with_saturation(1.15)
        .with_quality(BackdropQuality::High)
        .with_liquid_glass(configured_glass(settings))
}

fn select_profile(commands: &mut Commands, button: Entity) {
    commands.queue(move |world: &mut World| {
        if let Some(mode) = world.get::<ProfileButton>(button) {
            let profile = mode.0;
            world.resource_mut::<GlassSettings>().profile = profile;
        }
    });
}

fn update_controls(controls: Query<(&GlassControl, &Slider)>, mut settings: ResMut<GlassSettings>) {
    for (control, slider) in &controls {
        settings.values[control.0] = slider.value;
    }
}

fn update_scene(
    time: Res<Time>,
    settings: Res<GlassSettings>,
    policy: Res<beverly::theme::AccessibilityVisualPolicyResource>,
    mut lenses: Query<&mut Surface, With<GlassLens>>,
    mut stripes: Query<(&MovingStripe, &mut Node)>,
    mut buttons: Query<(&ProfileButton, &mut BeverlyButton)>,
) {
    let backdrop = Some(configured_backdrop(&settings));
    for mut surface in &mut lenses {
        if surface.backdrop != backdrop {
            surface.backdrop = backdrop;
        }
    }
    let offset = if policy.current.reduced_motion {
        0.0
    } else {
        time.elapsed_secs().sin() * 28.0
    };
    for (stripe, mut node) in &mut stripes {
        node.left = Val::Px(stripe.0 as f32 * STRIPE_WIDTH - STRIPE_WIDTH + offset);
    }
    for (mode, mut button) in &mut buttons {
        let outline = mode.0 != settings.profile;
        if button.outline != outline {
            button.outline = outline;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controls_map_to_in_range_optical_parameters() {
        let settings = GlassSettings {
            values: [24.0, 48.0, 180.0, 60.0, 80.0],
            profile: GlassProfile::Lip,
        };
        let glass = configured_glass(&settings);
        assert_eq!(glass, glass.sanitized());
        assert_eq!(glass.thickness, 24.0);
        assert_eq!(glass.bezel_width, 48.0);
        assert_eq!(glass.refractive_index, 1.8);
        assert_eq!(glass.specular_intensity, 0.6);
        assert_eq!(glass.chromatic_aberration, 0.08);
        assert_eq!(glass.profile, GlassProfile::Lip);
        assert_eq!(configured_backdrop(&settings).liquid_glass, Some(glass));
    }
}
