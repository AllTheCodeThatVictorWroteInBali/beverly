use bevy::prelude::*;

use crate::theme::ThemeResource;

use super::{
    backdrop::{Backdrop, BackdropQuality},
    border::{Border, BorderWidths},
    effect::{InnerShadow, OuterGlow, OuterShadow, ShadowFalloff},
    noise::{Noise, NoiseTarget},
    paint::{AngularGradient, GradientStop, LinearGradient, Paint, RadialGradient},
    shape::{RoundedRect, Shape},
    surface::Surface,
};

#[derive(Resource, Clone, Copy, Debug)]
pub struct UiRenderingDemoConfig {
    pub enabled: bool,
}

impl Default for UiRenderingDemoConfig {
    fn default() -> Self {
        let enabled = std::env::var("UI_RENDERING_DEMO")
            .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "on"))
            .unwrap_or(false);

        Self { enabled }
    }
}

#[derive(Component)]
struct DemoRoot;

#[derive(Component, Clone, Copy)]
pub(super) struct AnimatedRadius {
    min: f32,
    max: f32,
    speed: f32,
}

#[derive(Component, Clone, Copy)]
pub(super) struct AnimatedBorderWidth {
    min: f32,
    max: f32,
    speed: f32,
}

#[derive(Component, Clone, Copy)]
pub(super) struct AnimatedGradientRotation {
    speed: f32,
}

pub(super) fn spawn_demo_if_enabled(
    mut commands: Commands,
    theme: Res<ThemeResource>,
    config: Res<UiRenderingDemoConfig>,
) {
    if !config.enabled {
        return;
    }

    let colors = theme.current.colors;
    let shadow_tokens = theme.current.shadows;
    let panel_border = Color::srgba(0.48, 0.58, 0.72, 0.35);
    let light_bg = Color::srgba(0.96, 0.97, 0.99, 1.0);
    let dark_bg = Color::srgba(0.08, 0.11, 0.18, 1.0);

    commands
        .spawn((
            Name::new("ui_rendering_demo_root"),
            DemoRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(24.0),
                top: px(24.0),
                width: px(780.0),
                max_width: percent(95.0),
                padding: UiRect::all(px(16.0)),
                flex_direction: FlexDirection::Column,
                row_gap: px(10.0),
                border: UiRect::all(px(1.0)),
                ..default()
            },
            ZIndex(10000),
            BorderColor::all(panel_border),
            BackgroundColor(Color::srgba(0.10, 0.13, 0.20, 0.86)),
            Pickable::IGNORE,
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("UI Rendering SDF Matrix"),
                TextColor(colors.text),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
            ));

            spawn_demo_item(
                root,
                "Fill only",
                Vec2::new(180.0, 62.0),
                Surface::new(
                    Shape::RoundedRect(RoundedRect::new(12.0)),
                    Paint::solid(colors.primary),
                ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            let grad_two = vec![
                GradientStop::new(0.0, colors.primary),
                GradientStop::new(1.0, colors.primary_hover),
            ];
            let grad_three = vec![
                GradientStop::new(0.0, colors.primary),
                GradientStop::new(0.5, colors.info),
                GradientStop::new(1.0, colors.success),
            ];
            let grad_four = vec![
                GradientStop::new(0.0, colors.error),
                GradientStop::new(0.33, colors.warning),
                GradientStop::new(0.66, colors.info),
                GradientStop::new(1.0, colors.error),
            ];

            spawn_demo_item(
                root,
                "Linear horizontal",
                Vec2::new(220.0, 62.0),
                Surface::new(
                    Shape::RoundedRect(RoundedRect::new(12.0)),
                    Paint::linear(LinearGradient::horizontal(grad_two.clone())),
                ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Linear vertical",
                Vec2::new(220.0, 62.0),
                Surface::new(
                    Shape::RoundedRect(RoundedRect::new(12.0)),
                    Paint::linear(LinearGradient::vertical(grad_two.clone())),
                ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Linear 45 deg",
                Vec2::new(220.0, 62.0),
                Surface::new(
                    Shape::RoundedRect(RoundedRect::new(12.0)),
                    Paint::linear(LinearGradient::angle_degrees(45.0, grad_three.clone())),
                ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Linear 90 deg",
                Vec2::new(220.0, 62.0),
                Surface::new(
                    Shape::RoundedRect(RoundedRect::new(12.0)),
                    Paint::linear(LinearGradient::angle_degrees(90.0, grad_three.clone())),
                ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Linear 180 deg",
                Vec2::new(220.0, 62.0),
                Surface::new(
                    Shape::RoundedRect(RoundedRect::new(12.0)),
                    Paint::linear(LinearGradient::angle_degrees(180.0, grad_three.clone())),
                ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Radial centered",
                Vec2::new(220.0, 62.0),
                Surface::new(
                    Shape::RoundedRect(RoundedRect::new(12.0)),
                    Paint::radial(RadialGradient::circular(
                        Vec2::new(0.5, 0.5),
                        0.5,
                        grad_three.clone(),
                    )),
                ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Radial off-center",
                Vec2::new(220.0, 62.0),
                Surface::new(
                    Shape::RoundedRect(RoundedRect::new(12.0)),
                    Paint::radial(RadialGradient::new(
                        Vec2::new(0.25, 0.5),
                        Vec2::new(0.7, 0.5),
                        grad_three.clone(),
                    )),
                ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Radial small radius",
                Vec2::new(220.0, 62.0),
                Surface::new(
                    Shape::RoundedRect(RoundedRect::new(12.0)),
                    Paint::radial(RadialGradient::circular(
                        Vec2::new(0.5, 0.5),
                        0.2,
                        grad_four.clone(),
                    )),
                ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Radial large radius",
                Vec2::new(220.0, 62.0),
                Surface::new(
                    Shape::RoundedRect(RoundedRect::new(12.0)),
                    Paint::radial(RadialGradient::new(
                        Vec2::new(0.5, 0.5),
                        Vec2::new(1.2, 0.8),
                        grad_four.clone(),
                    )),
                ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Angular centered seam-safe",
                Vec2::new(220.0, 62.0),
                Surface::new(
                    Shape::RoundedRect(RoundedRect::new(12.0)),
                    Paint::angular(AngularGradient::new(
                        Vec2::new(0.5, 0.5),
                        0.0,
                        grad_four.clone(),
                    )),
                ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Angular rotated",
                Vec2::new(220.0, 62.0),
                Surface::new(
                    Shape::RoundedRect(RoundedRect::new(12.0)),
                    Paint::angular(AngularGradient::angle_degrees(
                        Vec2::new(0.5, 0.5),
                        120.0,
                        grad_four.clone(),
                    )),
                ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Linear two stops",
                Vec2::new(220.0, 62.0),
                Surface::new(
                    Shape::RoundedRect(RoundedRect::new(12.0)),
                    Paint::linear(LinearGradient::horizontal(grad_two.clone())),
                ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Linear three stops",
                Vec2::new(220.0, 62.0),
                Surface::new(
                    Shape::RoundedRect(RoundedRect::new(12.0)),
                    Paint::linear(LinearGradient::horizontal(grad_three.clone())),
                ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Linear four stops",
                Vec2::new(220.0, 62.0),
                Surface::new(
                    Shape::RoundedRect(RoundedRect::new(12.0)),
                    Paint::linear(LinearGradient::horizontal(grad_four.clone())),
                ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            let alpha_stops_a = vec![
                GradientStop::new(0.0, Color::srgba(0.98, 0.15, 0.15, 1.0)),
                GradientStop::new(1.0, Color::srgba(0.98, 0.15, 0.15, 0.0)),
            ];
            let alpha_stops_b = vec![
                GradientStop::new(0.0, Color::srgba(0.20, 0.55, 0.98, 0.0)),
                GradientStop::new(1.0, Color::srgba(0.20, 0.55, 0.98, 1.0)),
            ];

            spawn_demo_item(
                root,
                "Alpha opaque to transparent",
                Vec2::new(220.0, 62.0),
                Surface::new(
                    Shape::RoundedRect(RoundedRect::new(12.0)),
                    Paint::linear(LinearGradient::horizontal(alpha_stops_a)),
                ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Alpha transparent to opaque",
                Vec2::new(220.0, 62.0),
                Surface::new(
                    Shape::RoundedRect(RoundedRect::new(12.0)),
                    Paint::linear(LinearGradient::horizontal(alpha_stops_b)),
                ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Gradient border compatibility",
                Vec2::new(220.0, 62.0),
                Surface::rounded_rect_fill(14.0, colors.surface)
                    .border(Border::new(
                        4.0,
                        Paint::linear(LinearGradient::angle_degrees(35.0, grad_four.clone())),
                    )),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_section_header(root, "Milestone #9: Gradient border as reusable paint");

            let border_linear = Paint::linear(LinearGradient::angle_degrees(
                32.0,
                vec![
                    GradientStop::new(0.0, Color::srgba(0.18, 0.68, 1.0, 1.0)),
                    GradientStop::new(0.5, Color::srgba(0.64, 0.32, 1.0, 1.0)),
                    GradientStop::new(1.0, Color::srgba(1.0, 0.42, 0.72, 1.0)),
                ],
            ));

            let border_radial = Paint::radial(RadialGradient::new(
                Vec2::new(0.5, 0.5),
                Vec2::new(0.85, 0.75),
                vec![
                    GradientStop::new(0.0, Color::srgba(1.0, 1.0, 1.0, 1.0)),
                    GradientStop::new(0.45, Color::srgba(0.46, 0.72, 1.0, 0.95)),
                    GradientStop::new(1.0, Color::srgba(0.16, 0.24, 0.64, 1.0)),
                ],
            ));

            let border_angular = Paint::angular(AngularGradient::angle_degrees(
                Vec2::new(0.5, 0.5),
                20.0,
                vec![
                    GradientStop::new(0.0, Color::srgba(1.0, 0.28, 0.24, 1.0)),
                    GradientStop::new(0.25, Color::srgba(1.0, 0.78, 0.24, 1.0)),
                    GradientStop::new(0.5, Color::srgba(0.22, 0.94, 0.46, 1.0)),
                    GradientStop::new(0.75, Color::srgba(0.22, 0.62, 1.0, 1.0)),
                    GradientStop::new(1.0, Color::srgba(1.0, 0.28, 0.24, 1.0)),
                ],
            ));

            spawn_demo_item(
                root,
                "Basic: solid fill + linear gradient border",
                Vec2::new(240.0, 72.0),
                Surface::rounded_rect_fill(16.0, Color::srgba(1.0, 1.0, 1.0, 0.08))
                    .border(Border::new(2.0, border_linear.clone())),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Gradient fill + gradient border",
                Vec2::new(260.0, 84.0),
                Surface::new(
                    Shape::rounded_rect(18.0),
                    Paint::linear(LinearGradient::angle_degrees(
                        145.0,
                        vec![
                            GradientStop::new(0.0, Color::srgba(0.06, 0.16, 0.38, 0.95)),
                            GradientStop::new(1.0, Color::srgba(0.22, 0.44, 0.92, 0.85)),
                        ],
                    )),
                )
                .border(Border::new(3.0, border_linear.clone())),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Asymmetric corners 4/24/12/32 with gradient border",
                Vec2::new(260.0, 84.0),
                Surface::new(
                    Shape::rounded_rect_corners(4.0, 24.0, 12.0, 32.0),
                    Paint::solid(Color::srgba(0.98, 0.99, 1.0, 0.10)),
                )
                .border(Border::new(4.0, border_linear.clone())),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Gradient border type: Linear",
                Vec2::new(220.0, 70.0),
                Surface::rounded_rect_fill(16.0, colors.surface).border(Border::new(3.0, border_linear.clone())),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Gradient border type: Radial",
                Vec2::new(220.0, 70.0),
                Surface::rounded_rect_fill(16.0, colors.surface).border(Border::new(3.0, border_radial.clone())),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Gradient border type: Angular",
                Vec2::new(220.0, 70.0),
                Surface::rounded_rect_fill(16.0, colors.surface).border(Border::new(3.0, border_angular.clone())),
                None,
                None,
                light_bg,
                dark_bg,
            );

            for width in [1.0, 2.0, 4.0, 8.0] {
                spawn_demo_item(
                    root,
                    &format!("Gradient border width {:.0}px", width),
                    Vec2::new(220.0, 72.0),
                    Surface::rounded_rect_fill(18.0, colors.surface).border(Border::new(width, border_linear.clone())),
                    None,
                    None,
                    light_bg,
                    dark_bg,
                );
            }

            spawn_demo_item(
                root,
                "Gradient border + outer shadow + inner shadow + glow",
                Vec2::new(280.0, 86.0),
                Surface::rounded_rect_fill(20.0, Color::srgba(1.0, 1.0, 1.0, 0.06))
                    .border(Border::new(3.0, border_linear.clone()))
                    .outer_shadow(
                        OuterShadow::new(Color::BLACK)
                            .with_offset(Vec2::new(0.0, 10.0))
                            .with_blur(20.0)
                            .with_spread(2.0)
                            .with_opacity(0.28),
                    )
                    .inner_shadow(
                        InnerShadow::new(Color::BLACK)
                            .with_offset(Vec2::new(0.0, 2.0))
                            .with_blur(10.0)
                            .with_spread(1.0)
                            .with_opacity(0.24),
                    )
                    .outer_glow(
                        OuterGlow::new(Color::srgba(0.35, 0.55, 1.0, 1.0))
                            .with_blur(14.0)
                            .with_spread(1.0)
                            .with_opacity(0.24),
                    ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Glass + gradient border",
                Vec2::new(300.0, 92.0),
                Surface::rounded_rect_fill(22.0, Color::srgba(1.0, 1.0, 1.0, 0.06))
                    .with_backdrop(
                        Backdrop::new()
                            .with_blur(20.0)
                            .with_tint(Color::srgba(0.90, 0.95, 1.0, 1.0))
                            .with_tint_opacity(0.14)
                            .with_brightness(0.94)
                            .with_saturation(0.88)
                            .with_contrast(1.08)
                            .with_quality(BackdropQuality::High),
                    )
                    .border(Border::new(2.0, border_linear.clone()))
                    .outer_shadow(
                        OuterShadow::new(Color::BLACK)
                            .with_offset(Vec2::new(0.0, 12.0))
                            .with_blur(22.0)
                            .with_spread(2.0)
                            .with_opacity(0.22),
                    ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "State: normal",
                Vec2::new(220.0, 64.0),
                Surface::rounded_rect_fill(14.0, colors.surface).border(Border::new(2.0, border_linear.clone())),
                None,
                None,
                light_bg,
                dark_bg,
            );
            spawn_demo_item(
                root,
                "State: hovered",
                Vec2::new(220.0, 64.0),
                Surface::rounded_rect_fill(14.0, colors.surface_elevated).border(Border::new(
                    2.0,
                    Paint::linear(LinearGradient::angle_degrees(
                        32.0,
                        vec![
                            GradientStop::new(0.0, Color::srgba(0.34, 0.82, 1.0, 1.0)),
                            GradientStop::new(1.0, Color::srgba(1.0, 0.52, 0.84, 1.0)),
                        ],
                    )),
                )),
                None,
                None,
                light_bg,
                dark_bg,
            );
            spawn_demo_item(
                root,
                "State: pressed",
                Vec2::new(220.0, 64.0),
                Surface::rounded_rect_fill(14.0, Color::srgba(0.09, 0.12, 0.20, 1.0)).border(Border::new(
                    2.0,
                    Paint::linear(LinearGradient::angle_degrees(
                        32.0,
                        vec![
                            GradientStop::new(0.0, Color::srgba(0.14, 0.42, 0.78, 1.0)),
                            GradientStop::new(1.0, Color::srgba(0.48, 0.20, 0.72, 1.0)),
                        ],
                    )),
                )),
                None,
                None,
                light_bg,
                dark_bg,
            );
            spawn_demo_item(
                root,
                "State: focused",
                Vec2::new(220.0, 64.0),
                Surface::rounded_rect_fill(14.0, colors.surface)
                    .border(Border::new(2.0, border_linear.clone()))
                    .with_focus_ring(
                        crate::rendering::FocusRing::outside(
                            2.0,
                            2.0,
                            Paint::solid(colors.focus),
                        )
                        .with_secondary(crate::rendering::FocusRingLayer::new(
                            3.0,
                            0.0,
                            Paint::solid(Color::WHITE),
                        ))
                        .with_glow(colors.focus_outer_glow()),
                    ),
                None,
                None,
                light_bg,
                dark_bg,
            );
            spawn_demo_item(
                root,
                "State: selected",
                Vec2::new(220.0, 64.0),
                Surface::rounded_rect_fill(14.0, Color::srgba(0.10, 0.14, 0.28, 0.95)).border(Border::new(3.0, border_angular.clone())),
                None,
                None,
                light_bg,
                dark_bg,
            );
            spawn_demo_item(
                root,
                "State: disabled",
                Vec2::new(220.0, 64.0),
                Surface::rounded_rect_fill(14.0, Color::srgba(0.32, 0.35, 0.40, 0.75)).border(Border::new(
                    2.0,
                    Paint::linear(LinearGradient::horizontal(vec![
                        GradientStop::new(0.0, Color::srgba(0.58, 0.60, 0.64, 0.65)),
                        GradientStop::new(1.0, Color::srgba(0.46, 0.48, 0.52, 0.65)),
                    ])),
                )),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Animated gradient border",
                Vec2::new(260.0, 84.0),
                Surface::rounded_rect_fill(20.0, Color::srgba(0.06, 0.09, 0.15, 0.92)).border(Border::new(
                    3.0,
                    Paint::angular(AngularGradient::angle_degrees(
                        Vec2::new(0.5, 0.5),
                        0.0,
                        vec![
                            GradientStop::new(0.0, Color::srgba(1.0, 0.24, 0.22, 1.0)),
                            GradientStop::new(0.2, Color::srgba(1.0, 0.78, 0.20, 1.0)),
                            GradientStop::new(0.4, Color::srgba(0.24, 0.94, 0.44, 1.0)),
                            GradientStop::new(0.6, Color::srgba(0.20, 0.66, 1.0, 1.0)),
                            GradientStop::new(0.8, Color::srgba(0.72, 0.40, 1.0, 1.0)),
                            GradientStop::new(1.0, Color::srgba(1.0, 0.24, 0.22, 1.0)),
                        ],
                    )),
                )),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Subtle dark gradient stress",
                Vec2::new(520.0, 96.0),
                Surface::new(
                    Shape::RoundedRect(RoundedRect::new(16.0)),
                    Paint::linear(LinearGradient::horizontal(vec![
                        GradientStop::new(0.0, Color::srgba(0.08, 0.11, 0.15, 1.0)),
                        GradientStop::new(1.0, Color::srgba(0.10, 0.12, 0.17, 1.0)),
                    ])),
                )
                .uniform_border(1.0, Paint::solid(Color::srgba(0.55, 0.62, 0.70, 0.35))),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "High contrast gradient stress",
                Vec2::new(520.0, 96.0),
                Surface::new(
                    Shape::RoundedRect(RoundedRect::new(16.0)),
                    Paint::linear(LinearGradient::angle_degrees(
                        135.0,
                        vec![
                            GradientStop::new(0.0, Color::WHITE),
                            GradientStop::new(1.0, Color::BLACK),
                        ],
                    )),
                ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "1px border",
                Vec2::new(180.0, 62.0),
                Surface::rounded_rect_fill(12.0, colors.primary)
                    .uniform_border(1.0, Paint::solid(colors.border_strong)),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "2px border",
                Vec2::new(180.0, 62.0),
                Surface::rounded_rect_fill(12.0, colors.primary)
                    .uniform_border(2.0, Paint::solid(colors.border_strong)),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "4px border",
                Vec2::new(180.0, 62.0),
                Surface::rounded_rect_fill(12.0, colors.primary)
                    .uniform_border(4.0, Paint::solid(colors.border_strong)),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Square + border",
                Vec2::new(180.0, 62.0),
                Surface::new(
                    Shape::RoundedRect(RoundedRect::new(0.0)),
                    Paint::solid(colors.primary),
                )
                .uniform_border(2.0, Paint::solid(colors.warning)),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Small radius + border",
                Vec2::new(180.0, 62.0),
                Surface::rounded_rect_fill(6.0, colors.success)
                    .uniform_border(2.0, Paint::solid(colors.border_strong)),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Large radius + thin border",
                Vec2::new(180.0, 62.0),
                Surface::rounded_rect_fill(26.0, colors.success)
                    .uniform_border(1.0, Paint::solid(colors.border_strong)),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Large radius + thick border",
                Vec2::new(180.0, 62.0),
                Surface::rounded_rect_fill(26.0, colors.success)
                    .uniform_border(6.0, Paint::solid(colors.border_strong)),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Pill + border",
                Vec2::new(180.0, 62.0),
                Surface::rounded_rect_fill(999.0, colors.secondary)
                    .uniform_border(2.0, Paint::solid(colors.primary_active)),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Asymmetric corners + border",
                Vec2::new(180.0, 62.0),
                Surface::new(
                    Shape::rounded_rect_corners(24.0, 6.0, 24.0, 2.0),
                    Paint::solid(colors.info),
                )
                .uniform_border(3.0, Paint::solid(colors.primary_active)),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Top-only border",
                Vec2::new(180.0, 62.0),
                Surface::rounded_rect_fill(16.0, colors.info).border(Border::per_side(
                    BorderWidths::sides(4.0, 0.0, 0.0, 0.0),
                    Paint::solid(colors.warning),
                )),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Bottom-only border",
                Vec2::new(180.0, 62.0),
                Surface::rounded_rect_fill(16.0, colors.info).border(Border::per_side(
                    BorderWidths::sides(0.0, 0.0, 4.0, 0.0),
                    Paint::solid(colors.warning),
                )),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Left-only border",
                Vec2::new(180.0, 62.0),
                Surface::rounded_rect_fill(16.0, colors.info).border(Border::per_side(
                    BorderWidths::sides(0.0, 0.0, 0.0, 6.0),
                    Paint::solid(colors.warning),
                )),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Right-only border",
                Vec2::new(180.0, 62.0),
                Surface::rounded_rect_fill(16.0, colors.info).border(Border::per_side(
                    BorderWidths::sides(0.0, 6.0, 0.0, 0.0),
                    Paint::solid(colors.warning),
                )),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Per-side mixed border",
                Vec2::new(180.0, 62.0),
                Surface::rounded_rect_fill(18.0, colors.info).border(Border::per_side(
                    BorderWidths::sides(4.0, 1.0, 2.0, 6.0),
                    Paint::solid(colors.primary_active),
                )),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Small radius + thick border",
                Vec2::new(180.0, 62.0),
                Surface::rounded_rect_fill(4.0, colors.secondary)
                    .uniform_border(8.0, Paint::solid(colors.primary_active)),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Tiny element",
                Vec2::new(24.5, 12.5),
                Surface::rounded_rect_fill(3.0, colors.warning)
                    .uniform_border(1.0, Paint::solid(colors.text)),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Huge element",
                Vec2::new(520.25, 96.75),
                Surface::new(
                    Shape::rounded_rect_corners(12.0, 48.0, 48.0, 12.0),
                    Paint::solid(colors.primary_hover),
                )
                .border(Border::per_side(
                    BorderWidths::sides(1.5, 6.0, 4.0, 2.5),
                    Paint::solid(colors.text),
                )),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Fractional size/radius/border",
                Vec2::new(141.5, 37.25),
                Surface::rounded_rect_fill(7.5, colors.primary_hover)
                    .uniform_border(0.5, Paint::solid(colors.text)),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Animated radius + border",
                Vec2::new(180.0, 62.0),
                Surface::rounded_rect_fill(2.0, colors.secondary)
                    .uniform_border(2.0, Paint::solid(colors.primary_active)),
                Some(AnimatedRadius {
                    min: 2.0,
                    max: 31.0,
                    speed: 2.0,
                }),
                Some(AnimatedBorderWidth {
                    min: 0.0,
                    max: 6.0,
                    speed: 1.5,
                }),
                light_bg,
                dark_bg,
            );

            spawn_demo_section_header(root, "Effects: Basic shadows");

            spawn_demo_item(
                root,
                "No shadow",
                Vec2::new(220.0, 68.0),
                Surface::rounded_rect_fill(14.0, colors.surface_elevated)
                    .uniform_border(1.0, Paint::solid(colors.border)),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Subtle / Medium / Strong",
                Vec2::new(220.0, 68.0),
                Surface::rounded_rect_fill(14.0, colors.surface_elevated)
                    .uniform_border(1.0, Paint::solid(colors.border))
                    .outer_shadow(colors.default_outer_shadow(theme.current.shadows)),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_section_header(root, "Effects: Shadow offset");

            for (label, offset) in [
                ("Top", Vec2::new(0.0, -8.0)),
                ("Bottom", Vec2::new(0.0, 8.0)),
                ("Left", Vec2::new(-10.0, 0.0)),
                ("Right", Vec2::new(10.0, 0.0)),
                ("Diagonal", Vec2::new(10.0, 8.0)),
            ] {
                spawn_demo_item(
                    root,
                    &format!("Offset {label}"),
                    Vec2::new(200.0, 64.0),
                    Surface::rounded_rect_fill(12.0, colors.surface)
                        .uniform_border(1.0, Paint::solid(colors.border))
                        .outer_shadow(
                            OuterShadow::new(Color::BLACK)
                                .with_offset(offset)
                                .with_blur(14.0)
                                .with_opacity(shadow_tokens.medium_alpha),
                        ),
                    None,
                    None,
                    light_bg,
                    dark_bg,
                );
            }

            spawn_demo_section_header(root, "Effects: Shadow blur + spread");

            for (label, blur, spread) in [
                ("Sharp", 0.5, 2.0),
                ("Soft", 8.0, 2.0),
                ("Very soft", 26.0, 2.0),
                ("Negative spread", 12.0, -4.0),
                ("Large spread", 18.0, 20.0),
            ] {
                spawn_demo_item(
                    root,
                    &format!("{label} (blur {blur:.1}, spread {spread:.1})"),
                    Vec2::new(220.0, 68.0),
                    Surface::rounded_rect_fill(14.0, colors.surface_elevated)
                        .uniform_border(1.0, Paint::solid(colors.border))
                        .outer_shadow(
                            OuterShadow::new(Color::BLACK)
                                .with_offset(Vec2::new(6.0, 8.0))
                                .with_blur(blur)
                                .with_spread(spread)
                                .with_opacity(shadow_tokens.medium_alpha),
                        ),
                    None,
                    None,
                    light_bg,
                    dark_bg,
                );
            }

            spawn_demo_section_header(root, "Effects: Falloff profiles");

            for (label, falloff) in [
                ("Linear", ShadowFalloff::Linear),
                ("Smooth", ShadowFalloff::Smooth),
                ("Gaussian", ShadowFalloff::Gaussian),
            ] {
                spawn_demo_item(
                    root,
                    &format!("Shadow falloff {label}"),
                    Vec2::new(220.0, 68.0),
                    Surface::rounded_rect_fill(14.0, colors.surface_elevated)
                        .uniform_border(1.0, Paint::solid(colors.border))
                        .outer_shadow(
                            OuterShadow::new(Color::BLACK)
                                .with_offset(Vec2::new(0.0, 8.0))
                                .with_blur(18.0)
                                .with_spread(2.0)
                                .with_opacity(shadow_tokens.high_alpha)
                                .with_falloff(falloff),
                        ),
                    None,
                    None,
                    light_bg,
                    dark_bg,
                );
            }

            spawn_demo_section_header(root, "Effects: Corner radii + combined render");

            spawn_demo_item(
                root,
                "Square / Rounded / Pill / Asymmetric shadow follow",
                Vec2::new(300.0, 80.0),
                Surface::new(
                    Shape::rounded_rect_corners(4.0, 16.0, 24.0, 8.0),
                    Paint::linear(LinearGradient::angle_degrees(
                        28.0,
                        vec![
                            GradientStop::new(0.0, colors.surface),
                            GradientStop::new(1.0, colors.surface_elevated),
                        ],
                    )),
                )
                .border(Border::per_side(
                    BorderWidths::sides(2.0, 1.0, 3.0, 1.5),
                    Paint::solid(colors.border_strong),
                ))
                .outer_shadow(
                    OuterShadow::new(Color::BLACK)
                        .with_offset(Vec2::new(8.0, 10.0))
                        .with_blur(24.0)
                        .with_spread(6.0)
                        .with_opacity(shadow_tokens.high_alpha),
                ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_section_header(root, "Effects: Outer glow");

            for (label, opacity, blur, spread) in [
                ("Glow subtle", 0.15, 10.0, 1.0),
                ("Glow medium", 0.28, 18.0, 4.0),
                ("Glow strong", 0.45, 30.0, 8.0),
            ] {
                spawn_demo_item(
                    root,
                    label,
                    Vec2::new(220.0, 68.0),
                    Surface::rounded_rect_fill(16.0, colors.primary.with_alpha(0.82))
                        .uniform_border(1.0, Paint::solid(colors.primary_active))
                        .outer_glow(
                            OuterGlow::new(colors.focus)
                                .with_blur(blur)
                                .with_spread(spread)
                                .with_opacity(opacity)
                                .with_falloff(ShadowFalloff::Gaussian),
                        ),
                    None,
                    None,
                    light_bg,
                    dark_bg,
                );
            }

            spawn_demo_item(
                root,
                "Shadow + Glow together",
                Vec2::new(240.0, 72.0),
                Surface::rounded_rect_fill(
                    18.0,
                    Paint::linear(LinearGradient::horizontal(vec![
                        GradientStop::new(0.0, colors.primary_hover),
                        GradientStop::new(1.0, colors.primary),
                    ])),
                )
                .uniform_border(1.0, Paint::solid(colors.border.with_alpha(0.65)))
                .outer_shadow(
                    OuterShadow::new(Color::BLACK)
                        .with_offset(Vec2::new(0.0, 6.0))
                        .with_blur(18.0)
                        .with_spread(2.0)
                        .with_opacity(shadow_tokens.medium_alpha),
                )
                .outer_glow(
                    OuterGlow::new(colors.focus)
                        .with_blur(14.0)
                        .with_spread(2.0)
                        .with_opacity(0.20),
                ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_section_header(root, "Effects: Inner shadow");

            spawn_demo_item(
                root,
                "Inner shadow subtle",
                Vec2::new(240.0, 72.0),
                Surface::rounded_rect_fill(16.0, colors.surface_elevated)
                    .uniform_border(1.0, Paint::solid(colors.border))
                    .inner_shadow(
                        InnerShadow::new(Color::BLACK)
                            .with_offset(Vec2::new(0.0, 1.5))
                            .with_blur(10.0)
                            .with_spread(1.0)
                            .with_opacity(0.22)
                            .with_falloff(ShadowFalloff::Smooth),
                    ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            for (label, offset) in [
                ("Inner offset top", Vec2::new(0.0, -5.0)),
                ("Inner offset bottom", Vec2::new(0.0, 5.0)),
                ("Inner offset left", Vec2::new(-6.0, 0.0)),
                ("Inner offset right", Vec2::new(6.0, 0.0)),
            ] {
                spawn_demo_item(
                    root,
                    label,
                    Vec2::new(240.0, 72.0),
                    Surface::rounded_rect_fill(16.0, colors.surface_elevated)
                        .uniform_border(1.0, Paint::solid(colors.border))
                        .inner_shadow(
                            InnerShadow::new(Color::BLACK)
                                .with_offset(offset)
                                .with_blur(10.0)
                                .with_spread(1.5)
                                .with_opacity(0.24)
                                .with_falloff(ShadowFalloff::Smooth),
                        ),
                    None,
                    None,
                    light_bg,
                    dark_bg,
                );
            }

            for (label, blur) in [
                ("Inner blur sharp", 1.0),
                ("Inner blur medium", 8.0),
                ("Inner blur soft", 20.0),
            ] {
                spawn_demo_item(
                    root,
                    label,
                    Vec2::new(240.0, 72.0),
                    Surface::rounded_rect_fill(16.0, colors.surface_elevated)
                        .uniform_border(1.0, Paint::solid(colors.border))
                        .inner_shadow(
                            InnerShadow::new(Color::BLACK)
                                .with_offset(Vec2::new(0.0, 3.0))
                                .with_blur(blur)
                                .with_spread(1.0)
                                .with_opacity(0.24)
                                .with_falloff(ShadowFalloff::Smooth),
                        ),
                    None,
                    None,
                    light_bg,
                    dark_bg,
                );
            }

            for (label, spread) in [
                ("Inner spread negative", -4.0),
                ("Inner spread zero", 0.0),
                ("Inner spread positive", 4.0),
            ] {
                spawn_demo_item(
                    root,
                    label,
                    Vec2::new(240.0, 72.0),
                    Surface::rounded_rect_fill(16.0, colors.surface_elevated)
                        .uniform_border(1.0, Paint::solid(colors.border))
                        .inner_shadow(
                            InnerShadow::new(Color::BLACK)
                                .with_offset(Vec2::new(0.0, 3.0))
                                .with_blur(10.0)
                                .with_spread(spread)
                                .with_opacity(0.24)
                                .with_falloff(ShadowFalloff::Smooth),
                        ),
                    None,
                    None,
                    light_bg,
                    dark_bg,
                );
            }

            spawn_demo_item(
                root,
                "Inner follows asymmetric corners",
                Vec2::new(260.0, 84.0),
                Surface::new(
                    Shape::rounded_rect_corners(4.0, 20.0, 8.0, 32.0),
                    Paint::solid(colors.surface_elevated),
                )
                .uniform_border(2.0, Paint::solid(colors.border_strong))
                .inner_shadow(
                    InnerShadow::new(Color::BLACK)
                        .with_offset(Vec2::new(0.0, 4.0))
                        .with_blur(11.0)
                        .with_spread(1.0)
                        .with_opacity(0.28)
                        .with_falloff(ShadowFalloff::Smooth),
                ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Gradient fill + inner shadow",
                Vec2::new(260.0, 80.0),
                Surface::rounded_rect_fill(
                    18.0,
                    Paint::linear(LinearGradient::angle_degrees(
                        42.0,
                        vec![
                            GradientStop::new(0.0, colors.primary),
                            GradientStop::new(1.0, colors.primary_hover),
                        ],
                    )),
                )
                .inner_shadow(
                    InnerShadow::new(Color::BLACK)
                        .with_offset(Vec2::new(0.0, 3.0))
                        .with_blur(10.0)
                        .with_spread(2.0)
                        .with_opacity(0.25)
                        .with_falloff(ShadowFalloff::Smooth),
                ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Inner + Outer shadow",
                Vec2::new(240.0, 72.0),
                Surface::rounded_rect_fill(16.0, colors.surface_elevated)
                    .uniform_border(1.0, Paint::solid(colors.border))
                    .outer_shadow(
                        OuterShadow::new(Color::BLACK)
                            .with_offset(Vec2::new(0.0, 6.0))
                            .with_blur(18.0)
                            .with_spread(2.0)
                            .with_opacity(shadow_tokens.low_alpha),
                    )
                    .inner_shadow(
                        InnerShadow::new(Color::BLACK)
                            .with_offset(Vec2::new(0.0, 2.0))
                            .with_blur(8.0)
                            .with_spread(2.0)
                            .with_opacity(0.24)
                            .with_falloff(ShadowFalloff::Smooth),
                    ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Gradient + border + outer + inner",
                Vec2::new(280.0, 84.0),
                Surface::new(
                    Shape::rounded_rect_corners(10.0, 26.0, 12.0, 24.0),
                    Paint::linear(LinearGradient::angle_degrees(
                        130.0,
                        vec![
                            GradientStop::new(0.0, colors.surface),
                            GradientStop::new(1.0, colors.surface_elevated),
                        ],
                    )),
                )
                .border(Border::per_side(
                    BorderWidths::sides(1.5, 2.5, 1.5, 2.5),
                    Paint::solid(colors.border_strong),
                ))
                .outer_shadow(
                    OuterShadow::new(Color::BLACK)
                        .with_offset(Vec2::new(0.0, 8.0))
                        .with_blur(20.0)
                        .with_spread(3.0)
                        .with_opacity(shadow_tokens.medium_alpha),
                )
                .inner_shadow(
                    InnerShadow::new(Color::BLACK)
                        .with_offset(Vec2::new(0.0, 3.0))
                        .with_blur(11.0)
                        .with_spread(1.5)
                        .with_opacity(0.28),
                ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Small shape stress (blur ~= size)",
                Vec2::new(26.0, 16.0),
                Surface::rounded_rect_fill(6.0, colors.surface_elevated)
                    .uniform_border(1.0, Paint::solid(colors.border))
                    .inner_shadow(
                        InnerShadow::new(Color::BLACK)
                            .with_offset(Vec2::new(0.0, 2.0))
                            .with_blur(10.0)
                            .with_spread(6.0)
                            .with_opacity(0.3),
                    ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_section_header(root, "Backdrop: Glass surfaces");

            for (label, blur, tint, tint_opacity, brightness, saturation) in [
                (
                    "Glass low blur",
                    4.0,
                    Color::srgba(0.96, 0.98, 1.0, 1.0),
                    0.10,
                    1.00,
                    1.00,
                ),
                (
                    "Glass medium blur",
                    12.0,
                    Color::srgba(0.92, 0.96, 1.0, 1.0),
                    0.13,
                    0.96,
                    0.90,
                ),
                (
                    "Glass high blur",
                    24.0,
                    Color::srgba(0.84, 0.92, 1.0, 1.0),
                    0.18,
                    0.92,
                    0.82,
                ),
            ] {
                spawn_demo_item(
                    root,
                    label,
                    Vec2::new(280.0, 86.0),
                    Surface::rounded_rect_fill(18.0, Color::srgba(1.0, 1.0, 1.0, 0.06))
                        .with_backdrop(
                            Backdrop::new()
                                .with_blur(blur)
                                .with_tint(tint)
                                .with_tint_opacity(tint_opacity)
                                .with_brightness(brightness)
                                .with_saturation(saturation)
                                .with_contrast(1.1)
                                .with_quality(BackdropQuality::High),
                        )
                        .uniform_border(1.0, Paint::solid(Color::srgba(1.0, 1.0, 1.0, 0.24)))
                        .outer_shadow(
                            OuterShadow::new(Color::BLACK)
                                .with_offset(Vec2::new(0.0, 10.0))
                                .with_blur(22.0)
                                .with_spread(2.0)
                                .with_opacity(0.25),
                        )
                        .inner_shadow(
                            InnerShadow::new(Color::WHITE)
                                .with_offset(Vec2::new(0.0, -2.0))
                                .with_blur(6.0)
                                .with_spread(0.0)
                                .with_opacity(0.20),
                        ),
                    None,
                    None,
                    light_bg,
                    dark_bg,
                );
            }

            spawn_demo_section_header(root, "Milestone #10: Procedural noise / grain overlay");

            spawn_demo_item(
                root,
                "Basic: solid surface + subtle grain",
                Vec2::new(240.0, 72.0),
                Surface::rounded_rect_fill(16.0, Color::srgba(0.96, 0.97, 0.99, 1.0))
                    .with_noise(Noise::grain(18.0, 0.02).with_seed(1.0)),
                None,
                None,
                light_bg,
                dark_bg,
            );

            for (label, strength) in [
                ("Noise strength 0.00", 0.00),
                ("Noise strength 0.01", 0.01),
                ("Noise strength 0.02", 0.02),
                ("Noise strength 0.04", 0.04),
            ] {
                spawn_demo_item(
                    root,
                    label,
                    Vec2::new(240.0, 70.0),
                    Surface::rounded_rect_fill(14.0, Color::srgba(0.12, 0.16, 0.28, 0.96))
                        .with_noise(Noise::grain(16.0, strength).with_seed(2.0)),
                    None,
                    None,
                    light_bg,
                    dark_bg,
                );
            }

            for (label, scale) in [
                ("Noise scale fine", 8.0),
                ("Noise scale medium", 20.0),
                ("Noise scale coarse", 44.0),
            ] {
                spawn_demo_item(
                    root,
                    label,
                    Vec2::new(240.0, 70.0),
                    Surface::rounded_rect_fill(14.0, Color::srgba(0.94, 0.95, 0.98, 1.0))
                        .with_noise(Noise::grain(scale, 0.028).with_seed(3.0)),
                    None,
                    None,
                    light_bg,
                    dark_bg,
                );
            }

            spawn_demo_item(
                root,
                "Gradient fill + noise",
                Vec2::new(280.0, 84.0),
                Surface::new(
                    Shape::rounded_rect(16.0),
                    Paint::linear(LinearGradient::angle_degrees(
                        145.0,
                        vec![
                            GradientStop::new(0.0, Color::srgba(0.08, 0.18, 0.44, 1.0)),
                            GradientStop::new(1.0, Color::srgba(0.26, 0.52, 0.94, 0.94)),
                        ],
                    )),
                )
                .with_noise(Noise::grain(18.0, 0.022).with_seed(4.0)),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Gradient border + border-only noise",
                Vec2::new(280.0, 84.0),
                Surface::rounded_rect_fill(18.0, Color::srgba(1.0, 1.0, 1.0, 0.08))
                    .border(Border::new(
                        3.0,
                        Paint::linear(LinearGradient::angle_degrees(
                            24.0,
                            vec![
                                GradientStop::new(0.0, Color::srgba(0.18, 0.68, 1.0, 1.0)),
                                GradientStop::new(0.5, Color::srgba(0.68, 0.35, 1.0, 1.0)),
                                GradientStop::new(1.0, Color::srgba(1.0, 0.42, 0.78, 1.0)),
                            ],
                        )),
                    ))
                    .with_noise(
                        Noise::grain(14.0, 0.03)
                            .with_seed(5.0)
                            .with_target(NoiseTarget::Border),
                    ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Glass + backdrop + subtle grain",
                Vec2::new(300.0, 92.0),
                Surface::rounded_rect_fill(20.0, Color::srgba(1.0, 1.0, 1.0, 0.06))
                    .with_backdrop(
                        Backdrop::new()
                            .with_blur(18.0)
                            .with_tint(Color::srgba(0.90, 0.95, 1.0, 1.0))
                            .with_tint_opacity(0.14)
                            .with_brightness(0.93)
                            .with_saturation(0.86)
                            .with_contrast(1.06)
                            .with_quality(BackdropQuality::High),
                    )
                    .border(Border::new(2.0, border_linear.clone()))
                    .with_noise(Noise::grain(24.0, 0.017).with_seed(6.0))
                    .outer_shadow(
                        OuterShadow::new(Color::BLACK)
                            .with_offset(Vec2::new(0.0, 12.0))
                            .with_blur(22.0)
                            .with_spread(2.0)
                            .with_opacity(0.22),
                    ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Noise + outer shadow + inner shadow",
                Vec2::new(280.0, 88.0),
                Surface::rounded_rect_fill(20.0, Color::srgba(0.10, 0.13, 0.20, 0.95))
                    .with_noise(Noise::grain(18.0, 0.022).with_seed(7.0))
                    .outer_shadow(
                        OuterShadow::new(Color::BLACK)
                            .with_offset(Vec2::new(0.0, 10.0))
                            .with_blur(20.0)
                            .with_spread(1.0)
                            .with_opacity(0.24),
                    )
                    .inner_shadow(
                        InnerShadow::new(Color::BLACK)
                            .with_offset(Vec2::new(0.0, 2.0))
                            .with_blur(8.0)
                            .with_spread(1.0)
                            .with_opacity(0.24),
                    ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Geometry: square noise",
                Vec2::new(120.0, 64.0),
                Surface::rounded_rect_fill(0.0, Color::srgba(0.92, 0.95, 1.0, 1.0))
                    .with_noise(Noise::grain(14.0, 0.026).with_seed(8.0)),
                None,
                None,
                light_bg,
                dark_bg,
            );
            spawn_demo_item(
                root,
                "Geometry: rounded noise",
                Vec2::new(120.0, 64.0),
                Surface::rounded_rect_fill(18.0, Color::srgba(0.92, 0.95, 1.0, 1.0))
                    .with_noise(Noise::grain(14.0, 0.026).with_seed(9.0)),
                None,
                None,
                light_bg,
                dark_bg,
            );
            spawn_demo_item(
                root,
                "Geometry: asymmetric corners noise",
                Vec2::new(200.0, 72.0),
                Surface::new(
                    Shape::rounded_rect_corners(4.0, 28.0, 12.0, 36.0),
                    Paint::solid(Color::srgba(0.14, 0.18, 0.28, 0.98)),
                )
                .with_noise(Noise::grain(18.0, 0.024).with_seed(10.0)),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Animation: static grain (speed 0)",
                Vec2::new(240.0, 72.0),
                Surface::rounded_rect_fill(16.0, Color::srgba(0.92, 0.94, 0.98, 1.0))
                    .with_noise(
                        Noise::grain(18.0, 0.025)
                            .with_seed(11.0)
                            .with_animated(true)
                            .with_speed(0.0),
                    ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Animation: slow grain",
                Vec2::new(240.0, 72.0),
                Surface::rounded_rect_fill(16.0, Color::srgba(0.92, 0.94, 0.98, 1.0))
                    .with_noise(
                        Noise::grain(18.0, 0.025)
                            .with_seed(12.0)
                            .with_animated(true)
                            .with_speed(0.35),
                    ),
                None,
                None,
                light_bg,
                dark_bg,
            );

            spawn_demo_item(
                root,
                "Preset: Matte",
                Vec2::new(220.0, 70.0),
                Surface::rounded_rect_fill(14.0, Color::srgba(0.95, 0.95, 0.97, 1.0))
                    .with_noise(Noise::grain(16.0, 0.012).with_seed(13.0)),
                None,
                None,
                light_bg,
                dark_bg,
            );
            spawn_demo_item(
                root,
                "Preset: Paper",
                Vec2::new(220.0, 70.0),
                Surface::rounded_rect_fill(14.0, Color::srgba(0.98, 0.96, 0.90, 1.0))
                    .with_noise(Noise::grain(30.0, 0.016).with_seed(14.0)),
                None,
                None,
                light_bg,
                dark_bg,
            );
            spawn_demo_item(
                root,
                "Preset: Premium dark",
                Vec2::new(260.0, 84.0),
                Surface::new(
                    Shape::rounded_rect(18.0),
                    Paint::linear(LinearGradient::vertical(vec![
                        GradientStop::new(0.0, Color::srgba(0.06, 0.08, 0.12, 1.0)),
                        GradientStop::new(1.0, Color::srgba(0.10, 0.12, 0.18, 1.0)),
                    ])),
                )
                .border(Border::new(2.0, border_linear.clone()))
                .with_noise(Noise::grain(18.0, 0.02).with_seed(15.0))
                .outer_shadow(
                    OuterShadow::new(Color::BLACK)
                        .with_offset(Vec2::new(0.0, 10.0))
                        .with_blur(18.0)
                        .with_spread(1.0)
                        .with_opacity(0.24),
                ),
                None,
                None,
                light_bg,
                dark_bg,
            );
        });
}

fn spawn_demo_section_header(parent: &mut ChildSpawnerCommands, title: &str) {
    parent.spawn((
        Text::new(title),
        TextColor(Color::srgba(0.80, 0.90, 1.0, 0.96)),
        TextFont {
            font_size: FontSize::Px(14.0),
            ..default()
        },
    ));
}

fn spawn_demo_item(
    parent: &mut ChildSpawnerCommands,
    title: &str,
    shape_size: Vec2,
    surface: Surface,
    animated_radius: Option<AnimatedRadius>,
    animated_border: Option<AnimatedBorderWidth>,
    light_bg: Color,
    dark_bg: Color,
) {
    let animate_gradient = title.contains("Animated gradient border");

    parent
        .spawn((
            Node {
                width: percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: px(6.0),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(title),
                TextColor(Color::WHITE),
                TextFont {
                    font_size: FontSize::Px(13.0),
                    ..default()
                },
            ));

            row.spawn((
                Node {
                    width: percent(100.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: px(8.0),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|samples| {
                spawn_surface_sample(
                    samples,
                    "Light",
                    light_bg,
                    shape_size,
                    surface.clone(),
                    animated_radius,
                    animated_border,
                    animate_gradient,
                );
                spawn_surface_sample(
                    samples,
                    "Dark",
                    dark_bg,
                    shape_size,
                    surface,
                    animated_radius,
                    animated_border,
                    animate_gradient,
                );
            });
        });
}

fn spawn_surface_sample(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    background: Color,
    shape_size: Vec2,
    surface: Surface,
    animated_radius: Option<AnimatedRadius>,
    animated_border: Option<AnimatedBorderWidth>,
    animate_gradient: bool,
) {
    parent
        .spawn((
            Node {
                flex_grow: 1.0,
                min_height: px(90.0),
                padding: UiRect::all(px(8.0)),
                border_radius: BorderRadius::all(px(8.0)),
                flex_direction: FlexDirection::Column,
                row_gap: px(8.0),
                ..default()
            },
            BackgroundColor(background),
            Pickable::IGNORE,
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new(label),
                TextColor(Color::srgba(0.9, 0.9, 0.95, 0.9)),
                TextFont {
                    font_size: FontSize::Px(11.0),
                    ..default()
                },
            ));

            let mut shape_entity = panel.spawn((
                Node {
                    width: px(shape_size.x),
                    height: px(shape_size.y),
                    ..default()
                },
                surface,
                Pickable::IGNORE,
            ));

            if let Some(animated_radius) = animated_radius {
                shape_entity.insert(animated_radius);
            }

            if let Some(animated_border) = animated_border {
                shape_entity.insert(animated_border);
            }

            if animate_gradient {
                shape_entity.insert(AnimatedGradientRotation { speed: 1.2 });
            }
        });
}

pub(super) fn animate_demo_radius(
    time: Res<Time>,
    mut surfaces: Query<(&AnimatedRadius, &mut Surface)>,
) {
    let t = time.elapsed_secs();

    for (anim, mut surface) in &mut surfaces {
        let x = (t * anim.speed).sin() * 0.5 + 0.5;
        let radius = anim.min + (anim.max - anim.min) * x;

        let Shape::RoundedRect(rect) = &mut surface.shape;
        rect.radii = super::shape::CornerRadii::new(radius);
    }
}

pub(super) fn animate_demo_border_width(
    time: Res<Time>,
    mut surfaces: Query<(&AnimatedBorderWidth, &mut Surface)>,
) {
    let t = time.elapsed_secs();

    for (anim, mut surface) in &mut surfaces {
        let x = (t * anim.speed).sin() * 0.5 + 0.5;
        let width = anim.min + (anim.max - anim.min) * x;

        if let Some(border) = &mut surface.border {
            border.width = BorderWidths::all(width);
        }
    }
}

pub(super) fn animate_demo_gradient_rotation(
    time: Res<Time>,
    mut surfaces: Query<(&AnimatedGradientRotation, &mut Surface)>,
) {
    let angle = time.elapsed_secs();

    for (anim, mut surface) in &mut surfaces {
        let Some(border) = &mut surface.border else {
            continue;
        };

        let Paint::AngularGradient(gradient) = &mut border.paint else {
            continue;
        };

        gradient.angle_radians = angle * anim.speed;
    }
}
