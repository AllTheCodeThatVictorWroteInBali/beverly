//! Border shader demo: the biscuit tin's boundary department.
//! Run with `-- dark` to start in dark mode.

use beverly::components::button::ButtonMotionDisabled;
use beverly::components::slider::{Slider, SliderStyle, spawn_slider};
use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::prelude::*;
use beverly::primitives::interaction::{InteractionAction, InteractionActionEvent};
use beverly::primitives::semantic::{SemanticNode, SemanticRole};
use beverly::rendering::{Border, BorderWidths, GradientStop, LinearGradient, Paint, Surface};
use bevy::prelude::*;

const COLORS: [(&str, Color); 4] = [
    ("Mint", Color::srgb(0.12, 0.68, 0.48)),
    ("Coral", Color::srgb(0.94, 0.30, 0.34)),
    ("Sky", Color::srgb(0.12, 0.55, 0.90)),
    ("Grey", Color::srgb(0.5, 0.5, 0.5)),
];

#[derive(Resource)]
struct BorderSettings {
    selected: usize,
    thickness: f32,
}

impl Default for BorderSettings {
    fn default() -> Self {
        Self {
            selected: 0,
            thickness: 4.0,
        }
    }
}

#[derive(Component)]
struct BorderSwatch(usize);

#[derive(Component)]
struct ThicknessControl;

#[derive(Component)]
struct BorderReadout;

#[derive(Component, Clone, Copy)]
enum BorderPreview {
    Uniform,
    Bottom,
    Asymmetric,
    Gradient,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .init_resource::<BorderSettings>()
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
                Text::new("Biscuit Boundary Department"),
            ));
            content.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("Keeping the biscuits inside the lines since second breakfast."),
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
                            .with_children(|swatch| {
                                let mut semantic =
                                    SemanticNode::new(SemanticRole::Button).label(label);
                                semantic.state.pressed = Some(index == 0);
                                swatch.spawn((
                                    Button,
                                    ButtonMotionDisabled,
                                    BorderSwatch(index),
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
                                swatch
                                    .spawn((ThemedText::new(TextRole::Caption), Text::new(label)));
                            });
                    }
                });
            content.spawn((
                ThemedText::new(TextRole::Label),
                Text::new("Border thickness"),
            ));
            let slider = spawn_slider(
                content,
                Slider::new(0.0, 16.0)
                    .value(4.0)
                    .step(1.0)
                    .label("Border thickness in pixels"),
                SliderStyle {
                    width: 320.0,
                    track_color: theme.current.colors.border,
                    fill_color: theme.current.colors.text,
                    thumb_color: theme.current.colors.surface_elevated,
                    ..default()
                },
                &theme,
            );
            content.commands().entity(slider).insert(ThicknessControl);
            content.spawn((
                BorderReadout,
                ThemedText::new(TextRole::Body),
                Text::new("Mint - 4 px"),
            ));
            content
                .spawn(Node {
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(20.0),
                    row_gap: Val::Px(20.0),
                    ..default()
                })
                .with_children(|previews| {
                    for (kind, label, message) in [
                        (BorderPreview::Uniform, "Uniform", "Biscuits contained"),
                        (
                            BorderPreview::Bottom,
                            "Bottom only",
                            "The official bottom line",
                        ),
                        (
                            BorderPreview::Asymmetric,
                            "Individual sides",
                            "Extra security on the left",
                        ),
                        (
                            BorderPreview::Gradient,
                            "Gradient border",
                            "A colorful jurisdiction",
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
                                    .spawn((
                                        kind,
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Px(120.0),
                                            padding: UiRect::all(Val::Px(20.0)),
                                            align_items: AlignItems::Center,
                                            justify_content: JustifyContent::Center,
                                            ..default()
                                        },
                                        Surface::rounded_rect_fill(
                                            8.0,
                                            Paint::solid(theme.current.colors.surface),
                                        )
                                        .border(preview_border(kind, 4.0, COLORS[0].1)),
                                    ))
                                    .with_children(|panel| {
                                        panel.spawn((
                                            ThemedText::new(TextRole::Body),
                                            Text::new(message),
                                        ));
                                    });
                            });
                    }
                });
        });
    });
}

fn preview_border(kind: BorderPreview, thickness: f32, color: Color) -> Border {
    let widths = match kind {
        BorderPreview::Uniform | BorderPreview::Gradient => BorderWidths::all(thickness),
        BorderPreview::Bottom => BorderWidths::sides(0.0, 0.0, thickness, 0.0),
        BorderPreview::Asymmetric => {
            BorderWidths::sides(thickness * 0.5, thickness, thickness * 1.5, thickness * 2.0)
        }
    };
    let paint = match kind {
        BorderPreview::Gradient => Paint::linear(LinearGradient::horizontal(vec![
            GradientStop::new(0.0, color),
            GradientStop::new(1.0, COLORS[1].1),
        ])),
        _ => Paint::solid(color),
    };
    Border::per_side(widths, paint)
}

fn update_settings(
    mut actions: MessageReader<InteractionActionEvent>,
    swatches: Query<&BorderSwatch>,
    sliders: Query<&Slider, With<ThicknessControl>>,
    mut settings: ResMut<BorderSettings>,
) {
    for action in actions.read() {
        if action.action == InteractionAction::Activate {
            if let Ok(swatch) = swatches.get(action.target) {
                settings.selected = swatch.0;
            }
        }
    }
    if let Ok(slider) = sliders.single() {
        settings.thickness = slider.value;
    }
}

fn update_previews(
    settings: Res<BorderSettings>,
    theme: Res<ThemeResource>,
    mut previews: Query<(&BorderPreview, &mut Surface), Without<BorderSwatch>>,
    mut swatches: Query<(&BorderSwatch, &mut Surface, &mut SemanticNode), Without<BorderPreview>>,
    mut readouts: Query<&mut Text, With<BorderReadout>>,
) {
    let (name, color) = COLORS[settings.selected];
    for (kind, mut surface) in &mut previews {
        let border = Some(preview_border(*kind, settings.thickness, color));
        let fill = Paint::solid(theme.current.colors.surface);
        if surface.border != border {
            surface.border = border;
        }
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
    let label = format!("{name} - {:.0} px", settings.thickness);
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
    fn border_previews_preserve_side_widths_and_gradient_paint() {
        for thickness in [0.0, 4.0, 16.0] {
            let color = COLORS[2].1;
            assert_eq!(
                preview_border(BorderPreview::Uniform, thickness, color).width,
                BorderWidths::all(thickness)
            );
            assert_eq!(
                preview_border(BorderPreview::Bottom, thickness, color).width,
                BorderWidths::sides(0.0, 0.0, thickness, 0.0)
            );
            assert_eq!(
                preview_border(BorderPreview::Asymmetric, thickness, color).width,
                BorderWidths::sides(thickness * 0.5, thickness, thickness * 1.5, thickness * 2.0)
            );
            let gradient = preview_border(BorderPreview::Gradient, thickness, color);
            assert_eq!(gradient.width, BorderWidths::all(thickness));
            assert_eq!(
                gradient.paint,
                Paint::linear(LinearGradient::horizontal(vec![
                    GradientStop::new(0.0, color),
                    GradientStop::new(1.0, COLORS[1].1),
                ]))
            );
        }
    }
}
