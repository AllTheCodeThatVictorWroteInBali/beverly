//! Glow shader demo: the snack desk's radiant biscuit tin.
//! Run with `-- dark` to start in dark mode.

use beverly::components::button::{BeverlyButton, ButtonMotionDisabled};
use beverly::components::slider::{Slider, SliderStyle, spawn_slider};
use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::prelude::*;
use beverly::primitives::interaction::{InteractionAction, InteractionActionEvent};
use beverly::primitives::semantic::{SemanticNode, SemanticRole};
use beverly::rendering::{OuterGlow, Paint, ShadowFalloff, Surface};
use bevy::prelude::*;

const COLORS: [(&str, Color); 4] = [
    ("Mint", Color::srgb(0.10, 0.78, 0.48)),
    ("Coral", Color::srgb(0.96, 0.28, 0.36)),
    ("Sky", Color::srgb(0.12, 0.58, 0.96)),
    ("Gold", Color::srgb(0.98, 0.72, 0.16)),
];

#[derive(Resource)]
struct GlowSettings {
    color: usize,
    values: [f32; 3],
    falloff: ShadowFalloff,
}

impl Default for GlowSettings {
    fn default() -> Self {
        Self {
            color: 0,
            values: [24.0, 4.0, 65.0],
            falloff: ShadowFalloff::Gaussian,
        }
    }
}

#[derive(Component)]
struct GlowSwatch(usize);
#[derive(Component)]
struct GlowControl(usize);
#[derive(Component)]
struct FalloffButton(ShadowFalloff);
#[derive(Component)]
struct GlowPreview;
#[derive(Component)]
struct GlowSample(f32);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .init_resource::<GlowSettings>()
        .add_systems(Startup, setup)
        .add_systems(Update, (update_settings, update_surfaces).chain())
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
                Text::new("The Radiant Biscuit Bureau"),
            ));
            content.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("Second breakfast has a certain glow about it."),
            ));
            content
                .spawn(Node {
                    column_gap: Val::Px(20.0),
                    ..default()
                })
                .with_children(|palette| {
                    for (index, (label, color)) in COLORS.into_iter().enumerate() {
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
                                    GlowSwatch(index),
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
                                sample
                                    .spawn((ThemedText::new(TextRole::Caption), Text::new(label)));
                            });
                    }
                });
            content
                .spawn(Node {
                    column_gap: Val::Px(24.0),
                    row_gap: Val::Px(12.0),
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                })
                .with_children(|controls| {
                    for (index, (label, min, max, value)) in [
                        ("Softness (px)", 0.0, 60.0, 24.0),
                        ("Spread (px)", -12.0, 24.0, 4.0),
                        ("Opacity (%)", 0.0, 100.0, 65.0),
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
                                row.commands().entity(slider).insert(GlowControl(index));
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
                    for (label, falloff) in [
                        ("Linear", ShadowFalloff::Linear),
                        ("Smooth", ShadowFalloff::Smooth),
                        ("Gaussian", ShadowFalloff::Gaussian),
                    ] {
                        modes.spawn((
                            FalloffButton(falloff),
                            BeverlyButton::standard(label).on("click", select_falloff),
                        ));
                    }
                });
            content
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(240.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|stage| {
                    stage
                        .spawn((
                            GlowPreview,
                            Node {
                                width: Val::Px(300.0),
                                height: Val::Px(110.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            Surface::rounded_rect_fill(
                                8.0,
                                Paint::solid(theme.current.colors.surface_elevated),
                            )
                            .outer_glow(configured_glow(&GlowSettings::default())),
                        ))
                        .with_children(|panel| {
                            panel.spawn((
                                ThemedText::new(TextRole::Body),
                                Text::new("The biscuits are positively radiant"),
                            ));
                        });
                });
            content
                .spawn(Node {
                    justify_content: JustifyContent::SpaceAround,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(40.0),
                    row_gap: Val::Px(32.0),
                    ..default()
                })
                .with_children(|samples| {
                    for (label, opacity) in [("Subtle", 0.2), ("Bright", 0.5), ("Radiant", 0.9)] {
                        samples
                            .spawn((
                                GlowSample(opacity),
                                Node {
                                    width: Val::Px(180.0),
                                    height: Val::Px(70.0),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                Surface::rounded_rect_fill(
                                    8.0,
                                    Paint::solid(theme.current.colors.surface_elevated),
                                ),
                            ))
                            .with_children(|panel| {
                                panel.spawn((ThemedText::new(TextRole::Label), Text::new(label)));
                            });
                    }
                });
        });
    });
}

fn configured_glow(settings: &GlowSettings) -> OuterGlow {
    OuterGlow::new(COLORS[settings.color].1)
        .with_blur(settings.values[0])
        .with_spread(settings.values[1])
        .with_opacity(settings.values[2] / 100.0)
        .with_falloff(settings.falloff)
}

fn select_falloff(commands: &mut Commands, button: Entity) {
    commands.queue(move |world: &mut World| {
        if let Some(mode) = world.get::<FalloffButton>(button) {
            let falloff = mode.0;
            world.resource_mut::<GlowSettings>().falloff = falloff;
        }
    });
}

fn update_settings(
    mut actions: MessageReader<InteractionActionEvent>,
    swatches: Query<&GlowSwatch>,
    controls: Query<(&GlowControl, &Slider)>,
    mut settings: ResMut<GlowSettings>,
) {
    for action in actions.read() {
        if action.action == InteractionAction::Activate {
            if let Ok(swatch) = swatches.get(action.target) {
                settings.color = swatch.0;
            }
        }
    }
    for (control, slider) in &controls {
        settings.values[control.0] = slider.value;
    }
}

fn update_surfaces(
    settings: Res<GlowSettings>,
    theme: Res<ThemeResource>,
    mut surfaces: Query<
        (&mut Surface, Option<&GlowSample>),
        Or<(With<GlowPreview>, With<GlowSample>)>,
    >,
    mut swatches: Query<
        (&GlowSwatch, &mut Surface, &mut SemanticNode),
        (Without<GlowPreview>, Without<GlowSample>),
    >,
    mut buttons: Query<(&FalloffButton, &mut BeverlyButton)>,
) {
    for (mut surface, sample) in &mut surfaces {
        let fill = Paint::solid(theme.current.colors.surface_elevated);
        if surface.fill != fill {
            surface.fill = fill;
        }
        let glow = if let Some(sample) = sample {
            OuterGlow::new(COLORS[settings.color].1)
                .with_blur(16.0)
                .with_spread(2.0)
                .with_opacity(sample.0)
        } else {
            configured_glow(&settings)
        };
        if surface.effects.outer_glow != Some(glow) {
            surface.effects.outer_glow = Some(glow);
        }
    }
    for (swatch, mut surface, mut semantic) in &mut swatches {
        let selected = swatch.0 == settings.color;
        semantic.state.pressed = Some(selected);
        if let Some(border) = surface.border.as_mut() {
            border.paint = Paint::solid(if selected {
                theme.current.colors.text
            } else {
                theme.current.colors.border
            });
        }
    }
    for (mode, mut button) in &mut buttons {
        let outline = mode.0 != settings.falloff;
        if button.outline != outline {
            button.outline = outline;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controls_map_to_glow_color_softness_spread_opacity_and_falloff() {
        let settings = GlowSettings {
            color: 2,
            values: [32.0, -4.0, 75.0],
            falloff: ShadowFalloff::Smooth,
        };
        let glow = configured_glow(&settings);
        assert_eq!(glow.color, COLORS[2].1);
        assert_eq!(glow.blur, 32.0);
        assert_eq!(glow.spread, -4.0);
        assert_eq!(glow.opacity, 0.75);
        assert_eq!(glow.falloff, ShadowFalloff::Smooth);
    }
}
