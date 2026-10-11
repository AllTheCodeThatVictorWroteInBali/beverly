//! Outer shadow demo: give the biscuit desk some depth.
//! Run with `-- dark` to start in dark mode.

use beverly::components::button::BeverlyButton;
use beverly::components::slider::{Slider, SliderStyle, spawn_slider};
use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::prelude::*;
use beverly::rendering::{OuterShadow, Paint, ShadowFalloff, Surface};
use bevy::prelude::*;

#[derive(Resource)]
struct ShadowSettings {
    values: [f32; 5],
    falloff: ShadowFalloff,
}

impl Default for ShadowSettings {
    fn default() -> Self {
        Self {
            values: [0.0, 12.0, 24.0, 0.0, 35.0],
            falloff: ShadowFalloff::Smooth,
        }
    }
}

#[derive(Component)]
struct ShadowControl(usize);

#[derive(Component)]
struct ShadowPreview;

#[derive(Component)]
struct PresetSurface;

#[derive(Component)]
struct FalloffButton(ShadowFalloff);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .init_resource::<ShadowSettings>()
        .add_systems(Startup, setup)
        .add_systems(Update, (update_controls, update_surfaces).chain())
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
                Text::new("The Biscuit Elevation Office"),
            ));
            content.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("Giving second breakfast a little more depth."),
            ));
            content
                .spawn(Node {
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(24.0),
                    row_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|controls| {
                    for (index, (label, min, max, value)) in [
                        ("Horizontal offset (px)", -40.0, 40.0, 0.0),
                        ("Vertical offset (px)", -40.0, 40.0, 12.0),
                        ("Softness (px)", 0.0, 60.0, 24.0),
                        ("Spread (px)", -16.0, 24.0, 0.0),
                        ("Opacity (%)", 0.0, 100.0, 35.0),
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
                                row.commands().entity(slider).insert(ShadowControl(index));
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
                            ShadowPreview,
                            Node {
                                width: Val::Px(300.0),
                                height: Val::Px(120.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            Surface::rounded_rect_fill(
                                8.0,
                                Paint::solid(theme.current.colors.surface_elevated),
                            )
                            .uniform_border(1.0, Paint::solid(theme.current.colors.border))
                            .outer_shadow(configured_shadow(&ShadowSettings::default())),
                        ))
                        .with_children(|panel| {
                            panel.spawn((
                                ThemedText::new(TextRole::Body),
                                Text::new("A promotion for the biscuit tin"),
                            ));
                        });
                });
            content
                .spawn(Node {
                    column_gap: Val::Px(40.0),
                    row_gap: Val::Px(24.0),
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|presets| {
                    for (label, shadow) in [
                        ("Small", OuterShadow::small(Color::BLACK)),
                        ("Regular", OuterShadow::regular(Color::BLACK)),
                        ("Large", OuterShadow::large(Color::BLACK)),
                    ] {
                        presets
                            .spawn((
                                PresetSurface,
                                Node {
                                    width: Val::Px(200.0),
                                    height: Val::Px(70.0),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                Surface::rounded_rect_fill(
                                    8.0,
                                    Paint::solid(theme.current.colors.surface_elevated),
                                )
                                .uniform_border(1.0, Paint::solid(theme.current.colors.border))
                                .outer_shadow(shadow),
                            ))
                            .with_children(|panel| {
                                panel.spawn((ThemedText::new(TextRole::Label), Text::new(label)));
                            });
                    }
                });
        });
    });
}

fn configured_shadow(settings: &ShadowSettings) -> OuterShadow {
    OuterShadow::new(Color::BLACK)
        .with_offset(Vec2::new(settings.values[0], settings.values[1]))
        .with_blur(settings.values[2])
        .with_spread(settings.values[3])
        .with_opacity(settings.values[4] / 100.0)
        .with_falloff(settings.falloff)
}

fn select_falloff(commands: &mut Commands, button: Entity) {
    commands.queue(move |world: &mut World| {
        if let Some(mode) = world.get::<FalloffButton>(button) {
            let falloff = mode.0;
            world.resource_mut::<ShadowSettings>().falloff = falloff;
        }
    });
}

fn update_controls(
    controls: Query<(&ShadowControl, &Slider)>,
    mut settings: ResMut<ShadowSettings>,
) {
    for (control, slider) in &controls {
        settings.values[control.0] = slider.value;
    }
}

fn update_surfaces(
    settings: Res<ShadowSettings>,
    theme: Res<ThemeResource>,
    mut surfaces: Query<
        (&mut Surface, Has<ShadowPreview>),
        Or<(With<ShadowPreview>, With<PresetSurface>)>,
    >,
    mut buttons: Query<(&FalloffButton, &mut BeverlyButton)>,
) {
    for (mut surface, preview) in &mut surfaces {
        let fill = Paint::solid(theme.current.colors.surface_elevated);
        if surface.fill != fill {
            surface.fill = fill;
        }
        if let Some(border) = surface.border.as_mut() {
            border.paint = Paint::solid(theme.current.colors.border);
        }
        if preview {
            surface.effects.outer_shadow = Some(configured_shadow(&settings));
        }
    }
    for (mode, mut button) in &mut buttons {
        if button.outline != (mode.0 != settings.falloff) {
            button.outline = mode.0 != settings.falloff;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controls_map_to_shadow_parameters() {
        let settings = ShadowSettings {
            values: [-10.0, 20.0, 40.0, -4.0, 75.0],
            falloff: ShadowFalloff::Gaussian,
        };
        let shadow = configured_shadow(&settings);
        assert_eq!(shadow.offset, Vec2::new(-10.0, 20.0));
        assert_eq!(shadow.blur, 40.0);
        assert_eq!(shadow.spread, -4.0);
        assert_eq!(shadow.opacity, 0.75);
        assert_eq!(shadow.falloff, ShadowFalloff::Gaussian);
    }
}
