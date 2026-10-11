//! Backdrop blur demo: the snack desk's frosted observation window.
//! Run with `-- dark` to start in dark mode.

use beverly::components::slider::{Slider, SliderStyle, spawn_slider};
use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::prelude::*;
use beverly::rendering::{Backdrop, BackdropQuality, Paint, Surface};
use bevy::prelude::*;

const STRIPE_WIDTH: f32 = 64.0;

#[derive(Component)]
struct BlurControl(usize);

#[derive(Component)]
struct FrostedWindow;

#[derive(Component)]
struct MovingStripe(usize);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, update_preview)
        .run();
}

fn setup(mut commands: Commands, theme: Res<ThemeResource>) {
    spawn_themed_page(&mut commands, |root| {
        root.spawn(Node {
            width: Val::Px(760.0),
            max_width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        })
        .with_children(|content| {
            content.spawn((
                ThemedTitle::new(TitleLevel::H2),
                Text::new("The Frosted Biscuit Observatory"),
            ));
            content.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("Monitoring the snack situation through a tasteful haze."),
            ));
            content
                .spawn(Node {
                    column_gap: Val::Px(20.0),
                    row_gap: Val::Px(12.0),
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                })
                .with_children(|controls| {
                    for (index, (label, max, value)) in [
                        ("Blur (px)", 64.0, 18.0),
                        ("Tint (%)", 100.0, 20.0),
                        ("Saturation (%)", 200.0, 100.0),
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        controls
                            .spawn(Node {
                                width: Val::Px(240.0),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(6.0),
                                ..default()
                            })
                            .with_children(|row| {
                                row.spawn((ThemedText::new(TextRole::Label), Text::new(label)));
                                let slider = spawn_slider(
                                    row,
                                    Slider::new(0.0, max).value(value).step(1.0).label(label),
                                    SliderStyle {
                                        width: 240.0,
                                        show_value: true,
                                        track_color: theme.current.colors.border,
                                        fill_color: theme.current.colors.text,
                                        thumb_color: theme.current.colors.surface_elevated,
                                        ..default()
                                    },
                                    &theme,
                                );
                                row.commands().entity(slider).insert(BlurControl(index));
                            });
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
                .with_children(|scene| {
                    scene.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                        Surface::rounded_rect_fill(
                            0.0,
                            Paint::solid(Color::srgb(0.08, 0.23, 0.25)),
                        ),
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
                                top: Val::Px(0.0),
                                width: Val::Px(STRIPE_WIDTH),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            Surface::rounded_rect_fill(
                                0.0,
                                Paint::solid(colors[index % colors.len()]),
                            ),
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
                            .size(24.0)
                            .color(Color::WHITE),
                        Text::new("BISCUITS / TEA / TOAST"),
                    ));
                    for (left, label, frosted) in
                        [(40.0, "Clear view", false), (410.0, "Frosted view", true)]
                    {
                        let surface = Surface::rounded_rect_fill(8.0, Paint::solid(Color::NONE))
                            .uniform_border(1.0, Paint::solid(Color::WHITE.with_alpha(0.65)))
                            .with_backdrop(if frosted {
                                configured_backdrop(
                                    [18.0, 20.0, 100.0],
                                    theme.current.colors.surface,
                                )
                            } else {
                                Backdrop::new()
                                    .with_tint(Color::BLACK)
                                    .with_tint_opacity(0.12)
                            });
                        let mut window = scene.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(left),
                                top: Val::Px(90.0),
                                width: Val::Px(310.0),
                                height: Val::Px(180.0),
                                padding: UiRect::all(Val::Px(22.0)),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            surface,
                        ));
                        if frosted {
                            window.insert(FrostedWindow);
                        }
                        window.with_children(|panel| {
                            panel.spawn((
                                ThemedText::new(TextRole::Label).color(Color::WHITE),
                                Text::new(label),
                            ));
                        });
                    }
                });
        });
    });
}

fn configured_backdrop(values: [f32; 3], tint: Color) -> Backdrop {
    Backdrop::new()
        .with_blur(values[0])
        .with_tint(tint)
        .with_tint_opacity(values[1] / 100.0)
        .with_saturation(values[2] / 100.0)
        .with_quality(BackdropQuality::High)
}

fn update_preview(
    time: Res<Time>,
    theme: Res<ThemeResource>,
    policy: Res<beverly::theme::AccessibilityVisualPolicyResource>,
    controls: Query<(&BlurControl, &Slider)>,
    mut windows: Query<&mut Surface, With<FrostedWindow>>,
    mut stripes: Query<(&MovingStripe, &mut Node)>,
) {
    let mut values = [18.0, 20.0, 100.0];
    for (control, slider) in &controls {
        values[control.0] = slider.value;
    }
    let backdrop = Some(configured_backdrop(values, theme.current.colors.surface));
    for mut surface in &mut windows {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controls_map_to_blur_tint_and_saturation_without_glass_distortion() {
        let backdrop = configured_backdrop([32.0, 45.0, 150.0], Color::WHITE);
        assert_eq!(backdrop.blur, 32.0);
        assert_eq!(backdrop.tint_opacity, 0.45);
        assert_eq!(backdrop.saturation, 1.5);
        assert_eq!(backdrop.quality, BackdropQuality::High);
        assert!(backdrop.liquid_glass.is_none());
    }
}
