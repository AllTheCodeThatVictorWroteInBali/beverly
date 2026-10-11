//! Gradient shader demo: mix a little color into the snack desk.
//! Run with `-- dark` to start in dark mode.

use beverly::components::button::ButtonMotionDisabled;
use beverly::components::slider::{Slider, SliderStyle, spawn_slider};
use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::prelude::*;
use beverly::primitives::interaction::{InteractionAction, InteractionActionEvent};
use beverly::primitives::semantic::{SemanticNode, SemanticRole};
use beverly::rendering::{
    AngularGradient, GradientStop, LinearGradient, Paint, RadialGradient, Surface,
};
use bevy::prelude::*;

const PALETTES: [(&str, [Color; 3]); 3] = [
    (
        "Fresh",
        [
            Color::srgb(0.08, 0.62, 0.42),
            Color::srgb(0.98, 0.82, 0.20),
            Color::srgb(0.94, 0.28, 0.34),
        ],
    ),
    (
        "Seaside",
        [
            Color::srgb(0.06, 0.36, 0.78),
            Color::srgb(0.16, 0.82, 0.75),
            Color::srgb(0.96, 0.76, 0.26),
        ],
    ),
    (
        "Mono",
        [
            Color::srgb(0.08, 0.08, 0.08),
            Color::srgb(0.50, 0.50, 0.50),
            Color::srgb(0.95, 0.95, 0.95),
        ],
    ),
];

#[derive(Resource)]
struct GradientSettings {
    palette: usize,
    angle: f32,
    middle: f32,
    radius: f32,
}

impl Default for GradientSettings {
    fn default() -> Self {
        Self {
            palette: 0,
            angle: 45.0,
            middle: 0.5,
            radius: 0.65,
        }
    }
}

#[derive(Component)]
struct PaletteSwatch(usize);

#[derive(Component)]
struct GradientControl(usize);

#[derive(Component, Clone, Copy)]
enum GradientPreview {
    Linear,
    Radial,
    Angular,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .init_resource::<GradientSettings>()
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
            row_gap: Val::Px(20.0),
            ..default()
        })
        .with_children(|content| {
            content.spawn((
                ThemedTitle::new(TitleLevel::H2),
                Text::new("The Color Mixing Committee"),
            ));
            content.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("Three colors. Endless deliberation. Tea optional."),
            ));
            content
                .spawn(Node {
                    column_gap: Val::Px(20.0),
                    ..default()
                })
                .with_children(|palette| {
                    for (index, (label, colors)) in PALETTES.into_iter().enumerate() {
                        palette
                            .spawn(Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                row_gap: Val::Px(6.0),
                                ..default()
                            })
                            .with_children(|sample| {
                                let mut semantic =
                                    SemanticNode::new(SemanticRole::Button).label(label);
                                semantic.state.pressed = Some(index == 0);
                                sample.spawn((
                                    Button,
                                    ButtonMotionDisabled,
                                    PaletteSwatch(index),
                                    semantic,
                                    beverly::primitives::a11y::TabIndex(0),
                                    Node {
                                        width: Val::Px(80.0),
                                        height: Val::Px(40.0),
                                        ..default()
                                    },
                                    Surface::rounded_rect_fill(
                                        6.0,
                                        Paint::linear(LinearGradient::horizontal(stops(
                                            colors, 0.5,
                                        ))),
                                    )
                                    .uniform_border(2.0, Paint::solid(theme.current.colors.border)),
                                ));
                                sample
                                    .spawn((ThemedText::new(TextRole::Caption), Text::new(label)));
                            });
                    }
                });
            content
                .spawn(Node {
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(20.0),
                    row_gap: Val::Px(12.0),
                    ..default()
                })
                .with_children(|controls| {
                    for (index, (label, max, value)) in [
                        ("Angle (degrees)", 360.0, 45.0),
                        ("Middle stop (%)", 100.0, 50.0),
                        ("Radial size (%)", 100.0, 65.0),
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
                                let entity = spawn_slider(
                                    row,
                                    Slider::new(if index == 2 { 10.0 } else { 0.0 }, max)
                                        .value(value)
                                        .step(1.0)
                                        .label(label),
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
                                row.commands().entity(entity).insert(GradientControl(index));
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
                    for (kind, label) in [
                        (GradientPreview::Linear, "Linear"),
                        (GradientPreview::Radial, "Radial"),
                        (GradientPreview::Angular, "Angular"),
                    ] {
                        previews
                            .spawn(Node {
                                width: Val::Px(240.0),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(10.0),
                                ..default()
                            })
                            .with_children(|sample| {
                                sample.spawn((ThemedText::new(TextRole::Label), Text::new(label)));
                                sample.spawn((
                                    kind,
                                    Node {
                                        width: Val::Px(240.0),
                                        height: Val::Px(240.0),
                                        ..default()
                                    },
                                    Surface::rounded_rect_fill(
                                        8.0,
                                        preview_paint(kind, &GradientSettings::default()),
                                    ),
                                ));
                            });
                    }
                });
        });
    });
}

fn stops(colors: [Color; 3], middle: f32) -> Vec<GradientStop> {
    vec![
        GradientStop::new(0.0, colors[0]),
        GradientStop::new(middle, colors[1]),
        GradientStop::new(1.0, colors[2]),
    ]
}

fn preview_paint(kind: GradientPreview, settings: &GradientSettings) -> Paint {
    let stops = stops(PALETTES[settings.palette].1, settings.middle);
    match kind {
        GradientPreview::Linear => {
            Paint::linear(LinearGradient::angle_degrees(settings.angle, stops))
        }
        GradientPreview::Radial => Paint::radial(RadialGradient::circular(
            Vec2::splat(0.5),
            settings.radius,
            stops,
        )),
        GradientPreview::Angular => Paint::angular(AngularGradient::angle_degrees(
            Vec2::splat(0.5),
            settings.angle,
            stops,
        )),
    }
}

fn update_settings(
    mut actions: MessageReader<InteractionActionEvent>,
    swatches: Query<&PaletteSwatch>,
    controls: Query<(&GradientControl, &Slider)>,
    mut settings: ResMut<GradientSettings>,
) {
    for action in actions.read() {
        if action.action == InteractionAction::Activate {
            if let Ok(swatch) = swatches.get(action.target) {
                settings.palette = swatch.0;
            }
        }
    }
    for (control, slider) in &controls {
        match control.0 {
            0 => settings.angle = slider.value,
            1 => settings.middle = slider.value / 100.0,
            _ => settings.radius = slider.value / 100.0,
        }
    }
}

fn update_previews(
    settings: Res<GradientSettings>,
    theme: Res<ThemeResource>,
    mut previews: Query<(&GradientPreview, &mut Surface), Without<PaletteSwatch>>,
    mut swatches: Query<
        (&PaletteSwatch, &mut Surface, &mut SemanticNode),
        Without<GradientPreview>,
    >,
) {
    for (kind, mut surface) in &mut previews {
        let paint = preview_paint(*kind, &settings);
        if surface.fill != paint {
            surface.fill = paint;
        }
    }
    for (swatch, mut surface, mut semantic) in &mut swatches {
        let selected = settings.palette == swatch.0;
        semantic.state.pressed = Some(selected);
        if let Some(border) = surface.border.as_mut() {
            border.paint = Paint::solid(if selected {
                theme.current.colors.text
            } else {
                theme.current.colors.border
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gradient_controls_map_to_angle_stops_and_radius() {
        let settings = GradientSettings {
            palette: 1,
            angle: 90.0,
            middle: 0.25,
            radius: 0.8,
        };
        let expected = stops(PALETTES[1].1, 0.25);
        assert_eq!(
            preview_paint(GradientPreview::Linear, &settings),
            Paint::linear(LinearGradient::angle_degrees(90.0, expected.clone()))
        );
        assert_eq!(
            preview_paint(GradientPreview::Radial, &settings),
            Paint::radial(RadialGradient::circular(
                Vec2::splat(0.5),
                0.8,
                expected.clone()
            ))
        );
        assert_eq!(
            preview_paint(GradientPreview::Angular, &settings),
            Paint::angular(AngularGradient::angle_degrees(
                Vec2::splat(0.5),
                90.0,
                expected
            ))
        );
    }
}
