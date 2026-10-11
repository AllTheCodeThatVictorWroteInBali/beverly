//! Background color shader demo: the snack desk's paint department.
//! Run with `-- dark` to start in dark mode.

use beverly::components::button::ButtonMotionDisabled;
use beverly::components::slider::{Slider, SliderStyle, spawn_slider};
use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::prelude::*;
use beverly::primitives::interaction::{InteractionAction, InteractionActionEvent};
use beverly::primitives::semantic::{SemanticNode, SemanticRole};
use beverly::rendering::{Paint, Surface};
use bevy::prelude::*;

const COLORS: [(&str, Color); 5] = [
    ("Mint", Color::srgb(0.12, 0.68, 0.48)),
    ("Coral", Color::srgb(0.94, 0.30, 0.34)),
    ("Sky", Color::srgb(0.12, 0.55, 0.90)),
    ("Lemon", Color::srgb(0.95, 0.72, 0.15)),
    ("Ink", Color::srgb(0.18, 0.18, 0.20)),
];

#[derive(Resource)]
struct PaintSettings {
    selected: usize,
    opacity: f32,
}

impl Default for PaintSettings {
    fn default() -> Self {
        Self {
            selected: 0,
            opacity: 0.65,
        }
    }
}

#[derive(Component)]
struct ColorSwatch(usize);

#[derive(Component)]
struct OpacityControl;

#[derive(Component)]
struct PaintReadout;

#[derive(Component)]
enum PreviewFill {
    Solid,
    Transparent,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .init_resource::<PaintSettings>()
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
                Text::new("Snack Desk Paint Department"),
            ));
            content.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("Today's assignment: give the biscuit tin a fresh coat."),
            ));
            content
                .spawn(Node {
                    column_gap: Val::Px(20.0),
                    row_gap: Val::Px(12.0),
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                })
                .with_children(|palette| {
                    for (index, (name, color)) in COLORS.into_iter().enumerate() {
                        palette
                            .spawn(Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                row_gap: Val::Px(6.0),
                                ..default()
                            })
                            .with_children(|swatch| {
                                let mut semantic =
                                    SemanticNode::new(SemanticRole::Button).label(name);
                                semantic.state.pressed = Some(index == 0);
                                swatch.spawn((
                                    Button,
                                    ButtonMotionDisabled,
                                    ColorSwatch(index),
                                    semantic,
                                    beverly::primitives::a11y::TabIndex(0),
                                    Node {
                                        width: Val::Px(40.0),
                                        height: Val::Px(40.0),
                                        ..default()
                                    },
                                    Surface::rounded_rect_fill(6.0, Paint::solid(color))
                                        .uniform_border(
                                            2.0,
                                            Paint::solid(theme.current.colors.border),
                                        ),
                                ));
                                swatch.spawn((ThemedText::new(TextRole::Caption), Text::new(name)));
                            });
                    }
                });
            content.spawn((ThemedText::new(TextRole::Label), Text::new("Opacity")));
            let slider = spawn_slider(
                content,
                Slider::new(0.0, 100.0)
                    .value(65.0)
                    .step(1.0)
                    .label("Fill opacity"),
                SliderStyle {
                    width: 320.0,
                    track_color: theme.current.colors.border,
                    fill_color: theme.current.colors.text,
                    thumb_color: theme.current.colors.surface_elevated,
                    ..default()
                },
                &theme,
            );
            content.commands().entity(slider).insert(OpacityControl);
            content.spawn((
                PaintReadout,
                ThemedText::new(TextRole::Body),
                Text::new("Mint - 65% opacity"),
            ));
            content
                .spawn(Node {
                    column_gap: Val::Px(20.0),
                    row_gap: Val::Px(20.0),
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                })
                .with_children(|previews| {
                    for (label, kind) in [
                        ("Solid fill", 0),
                        ("Transparent fill", 1),
                        ("Layered fills", 2),
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
                                sample
                                    .spawn(Node {
                                        width: Val::Px(240.0),
                                        height: Val::Px(192.0),
                                        position_type: PositionType::Relative,
                                        ..default()
                                    })
                                    .with_children(|canvas| {
                                        if kind == 0 {
                                            canvas.spawn((
                                                PreviewFill::Solid,
                                                Node {
                                                    width: Val::Percent(100.0),
                                                    height: Val::Percent(100.0),
                                                    ..default()
                                                },
                                                Surface::rounded_rect_fill(
                                                    0.0,
                                                    Paint::solid(COLORS[0].1),
                                                ),
                                            ));
                                        } else {
                                            checkerboard(canvas);
                                            if kind == 2 {
                                                canvas.spawn((
                                                    Node {
                                                        position_type: PositionType::Absolute,
                                                        left: Val::Px(12.0),
                                                        top: Val::Px(12.0),
                                                        width: Val::Px(144.0),
                                                        height: Val::Px(120.0),
                                                        ..default()
                                                    },
                                                    Surface::rounded_rect_fill(
                                                        0.0,
                                                        Paint::solid(COLORS[1].1),
                                                    ),
                                                ));
                                            }
                                            canvas.spawn((
                                                PreviewFill::Transparent,
                                                Node {
                                                    position_type: PositionType::Absolute,
                                                    left: Val::Px(if kind == 2 {
                                                        84.0
                                                    } else {
                                                        0.0
                                                    }),
                                                    top: Val::Px(if kind == 2 {
                                                        60.0
                                                    } else {
                                                        0.0
                                                    }),
                                                    width: Val::Px(if kind == 2 {
                                                        144.0
                                                    } else {
                                                        240.0
                                                    }),
                                                    height: Val::Px(if kind == 2 {
                                                        120.0
                                                    } else {
                                                        192.0
                                                    }),
                                                    ..default()
                                                },
                                                Surface::rounded_rect_fill(
                                                    0.0,
                                                    Paint::solid(COLORS[0].1.with_alpha(0.65)),
                                                ),
                                            ));
                                        }
                                    });
                            });
                    }
                });
        });
    });
}

fn checkerboard(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|board| {
            for row in 0..8 {
                board
                    .spawn(Node {
                        height: Val::Px(24.0),
                        ..default()
                    })
                    .with_children(|strip| {
                        for column in 0..10 {
                            let grey = if (row + column) % 2 == 0 { 0.92 } else { 0.75 };
                            strip.spawn((
                                Node {
                                    width: Val::Px(24.0),
                                    height: Val::Px(24.0),
                                    flex_shrink: 0.0,
                                    ..default()
                                },
                                Surface::rounded_rect_fill(
                                    0.0,
                                    Paint::solid(Color::srgb(grey, grey, grey)),
                                ),
                            ));
                        }
                    });
            }
        });
}

fn update_settings(
    mut actions: MessageReader<InteractionActionEvent>,
    swatches: Query<&ColorSwatch>,
    sliders: Query<&Slider, With<OpacityControl>>,
    mut settings: ResMut<PaintSettings>,
) {
    for action in actions.read() {
        if action.action == InteractionAction::Activate {
            if let Ok(swatch) = swatches.get(action.target) {
                settings.selected = swatch.0;
            }
        }
    }
    if let Ok(slider) = sliders.single() {
        settings.opacity = (slider.value / 100.0).clamp(0.0, 1.0);
    }
}

fn update_previews(
    settings: Res<PaintSettings>,
    theme: Res<ThemeResource>,
    mut previews: Query<(&PreviewFill, &mut Surface), Without<ColorSwatch>>,
    mut swatches: Query<(&ColorSwatch, &mut Surface, &mut SemanticNode), Without<PreviewFill>>,
    mut readouts: Query<&mut Text, With<PaintReadout>>,
) {
    let (name, color) = COLORS[settings.selected];
    for (kind, mut surface) in &mut previews {
        let alpha = match kind {
            PreviewFill::Solid => 1.0,
            PreviewFill::Transparent => settings.opacity,
        };
        let fill = Paint::solid(color.with_alpha(alpha));
        if surface.fill != fill {
            surface.fill = fill;
        }
    }
    for (swatch, mut surface, mut semantic) in &mut swatches {
        let selected = swatch.0 == settings.selected;
        semantic.state.pressed = Some(selected);
        if let Some(border) = surface.border.as_mut() {
            border.paint = Paint::solid(if selected {
                theme.current.colors.text
            } else {
                theme.current.colors.border
            });
        }
    }
    let label = format!("{name} - {:.0}% opacity", settings.opacity * 100.0);
    for mut text in &mut readouts {
        if text.0 != label {
            *text = Text::new(label.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solid_fill_stays_opaque_while_transparent_fill_tracks_opacity_and_color() {
        let mut app = App::new();
        app.init_resource::<PaintSettings>()
            .init_resource::<ThemeResource>()
            .add_systems(Update, update_previews);
        let solid = app
            .world_mut()
            .spawn((
                PreviewFill::Solid,
                Surface::rounded_rect_fill(0.0, Paint::solid(Color::NONE)),
            ))
            .id();
        let transparent = app
            .world_mut()
            .spawn((
                PreviewFill::Transparent,
                Surface::rounded_rect_fill(0.0, Paint::solid(Color::NONE)),
            ))
            .id();
        for opacity in [0.0, 0.5, 1.0] {
            *app.world_mut().resource_mut::<PaintSettings>() = PaintSettings {
                selected: 2,
                opacity,
            };
            app.update();
            assert_eq!(
                app.world().get::<Surface>(solid).unwrap().fill,
                Paint::solid(COLORS[2].1)
            );
            assert_eq!(
                app.world().get::<Surface>(transparent).unwrap().fill,
                Paint::solid(COLORS[2].1.with_alpha(opacity))
            );
        }
    }
}
