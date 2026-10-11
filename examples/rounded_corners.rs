//! Rounded corners shader demo: soften the snack desk's sharp edges.
//! Run with `-- dark` to start in dark mode.

use beverly::components::slider::{Slider, SliderStyle, spawn_slider};
use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::prelude::*;
use beverly::rendering::{Paint, Shape, Surface};
use bevy::prelude::*;

#[derive(Component)]
struct RadiusControl(usize);

#[derive(Component, Clone, Copy)]
enum CornerPreview {
    Uniform,
    Independent,
    Pill,
    Circle,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, update_previews)
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
                Text::new("The Corner Committee"),
            ));
            content.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("Softening the sharp edges of biscuit bureaucracy."),
            ));
            content
                .spawn(Node {
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(24.0),
                    row_gap: Val::Px(12.0),
                    ..default()
                })
                .with_children(|controls| {
                    for (index, (label, value)) in [
                        ("Uniform radius", 16.0),
                        ("Top left", 0.0),
                        ("Top right", 24.0),
                        ("Bottom right", 48.0),
                        ("Bottom left", 12.0),
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
                                    Slider::new(0.0, 60.0).value(value).step(1.0).label(label),
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
                                row.commands().entity(slider).insert(RadiusControl(index));
                            });
                    }
                });
            content
                .spawn(Node {
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(20.0),
                    row_gap: Val::Px(20.0),
                    ..default()
                })
                .with_children(|previews| {
                    for (kind, label, message, color) in [
                        (
                            CornerPreview::Uniform,
                            "Uniform corners",
                            "No sharp remarks",
                            Color::srgb(0.10, 0.48, 0.34),
                        ),
                        (
                            CornerPreview::Independent,
                            "Independent corners",
                            "Each corner has opinions",
                            Color::srgb(0.72, 0.20, 0.26),
                        ),
                        (
                            CornerPreview::Pill,
                            "Pill",
                            "A rounded resolution",
                            Color::srgb(0.12, 0.40, 0.72),
                        ),
                        (
                            CornerPreview::Circle,
                            "Circle",
                            "Full circle",
                            Color::srgb(0.36, 0.31, 0.38),
                        ),
                    ] {
                        previews
                            .spawn(Node {
                                width: Val::Px(370.0),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(8.0),
                                ..default()
                            })
                            .with_children(|sample| {
                                sample.spawn((ThemedText::new(TextRole::Label), Text::new(label)));
                                sample
                                    .spawn(Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Px(120.0),
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    })
                                    .with_children(|slot| {
                                        slot.spawn((
                                            kind,
                                            Node {
                                                width: Val::Px(
                                                    if matches!(kind, CornerPreview::Circle) {
                                                        120.0
                                                    } else {
                                                        370.0
                                                    },
                                                ),
                                                height: Val::Px(120.0),
                                                padding: UiRect::all(Val::Px(20.0)),
                                                align_items: AlignItems::Center,
                                                justify_content: JustifyContent::Center,
                                                ..default()
                                            },
                                            Surface::new(
                                                preview_shape(kind, [16.0, 0.0, 24.0, 48.0, 12.0]),
                                                Paint::solid(color),
                                            ),
                                        ))
                                        .with_children(
                                            |panel| {
                                                panel.spawn((
                                                    ThemedText::new(TextRole::Body)
                                                        .color(Color::WHITE),
                                                    Text::new(message),
                                                ));
                                            },
                                        );
                                    });
                            });
                    }
                });
        });
    });
}

fn preview_shape(kind: CornerPreview, radii: [f32; 5]) -> Shape {
    match kind {
        CornerPreview::Uniform => Shape::rounded_rect(radii[0]),
        CornerPreview::Independent => {
            Shape::rounded_rect_corners(radii[1], radii[2], radii[3], radii[4])
        }
        CornerPreview::Pill | CornerPreview::Circle => Shape::rounded_rect(60.0),
    }
}

fn update_previews(
    controls: Query<(&RadiusControl, &Slider)>,
    mut previews: Query<(&CornerPreview, &mut Surface)>,
) {
    let mut radii = [16.0, 0.0, 24.0, 48.0, 12.0];
    for (control, slider) in &controls {
        radii[control.0] = slider.value;
    }
    for (kind, mut surface) in &mut previews {
        let shape = preview_shape(*kind, radii);
        if surface.shape != shape {
            surface.shape = shape;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radius_controls_update_uniform_and_independent_shapes_without_changing_pill_or_circle() {
        let mut app = App::new();
        app.add_systems(Update, update_previews);
        for (index, value) in [30.0, 1.0, 2.0, 3.0, 4.0].into_iter().enumerate() {
            app.world_mut()
                .spawn((RadiusControl(index), Slider::new(0.0, 60.0).value(value)));
        }
        for kind in [
            CornerPreview::Uniform,
            CornerPreview::Independent,
            CornerPreview::Pill,
            CornerPreview::Circle,
        ] {
            app.world_mut().spawn((
                kind,
                Surface::rounded_rect_fill(0.0, Paint::solid(Color::WHITE)),
            ));
        }
        app.update();
        let mut previews = app.world_mut().query::<(&CornerPreview, &Surface)>();
        for (kind, surface) in previews.iter(app.world()) {
            let expected = match kind {
                CornerPreview::Uniform => Shape::rounded_rect(30.0),
                CornerPreview::Independent => Shape::rounded_rect_corners(1.0, 2.0, 3.0, 4.0),
                CornerPreview::Pill | CornerPreview::Circle => Shape::rounded_rect(60.0),
            };
            assert_eq!(surface.shape, expected);
        }
    }
}
