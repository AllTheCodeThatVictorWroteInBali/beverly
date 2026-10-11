//! Clip and mask demo: the snack desk's cookie cutter.
//! Run with `-- dark` to start in dark mode.

use beverly::components::slider::{Slider, SliderStyle, spawn_slider};
use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::prelude::*;
use beverly::rendering::{
    Clip, GradientStop, LinearGradient, Mask, OuterShadow, Paint, Shape, Surface,
};
use bevy::prelude::*;

const CANVAS: Vec2 = Vec2::new(170.0, 170.0);
const SURFACE: Vec2 = Vec2::new(110.0, 100.0);
const DEFAULTS: [f32; 3] = [28.0, 28.0, 60.0];

#[derive(Component)]
struct CutControl(usize);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum Preview {
    Reference,
    Clip,
    Mask,
    ClipWithShadow,
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
                Text::new("The Cookie Cutter Department"),
            ));
            content.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("Trimming the biscuit without touching its shadow."),
            ));
            content
                .spawn(Node {
                    column_gap: Val::Px(24.0),
                    row_gap: Val::Px(10.0),
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                })
                .with_children(|controls| {
                    for (index, (label, max, value)) in [
                        ("Clip radius (px)", 80.0, DEFAULTS[0]),
                        ("Mask radius (px)", 80.0, DEFAULTS[1]),
                        ("Mask opacity (%)", 100.0, DEFAULTS[2]),
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
                                    Slider::new(0.0, max).value(value).step(1.0).label(label),
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
                                row.commands().entity(slider).insert(CutControl(index));
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
                        (Preview::Reference, "No clip or mask"),
                        (Preview::Clip, "Clip"),
                        (Preview::Mask, "Mask"),
                        (Preview::ClipWithShadow, "Clip + shadow"),
                    ] {
                        previews
                            .spawn(Node {
                                width: Val::Px(CANVAS.x),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(8.0),
                                ..default()
                            })
                            .with_children(|sample| {
                                sample.spawn((ThemedText::new(TextRole::Label), Text::new(label)));
                                sample
                                    .spawn(Node {
                                        width: Val::Px(CANVAS.x),
                                        height: Val::Px(CANVAS.y),
                                        position_type: PositionType::Relative,
                                        overflow: Overflow::clip(),
                                        ..default()
                                    })
                                    .with_children(|canvas| {
                                        checkerboard(canvas);
                                        canvas.spawn((
                                            kind,
                                            Node {
                                                position_type: PositionType::Absolute,
                                                left: Val::Px((CANVAS.x - SURFACE.x) * 0.5),
                                                top: Val::Px((CANVAS.y - SURFACE.y) * 0.5),
                                                width: Val::Px(SURFACE.x),
                                                height: Val::Px(SURFACE.y),
                                                ..default()
                                            },
                                            preview_surface(kind, DEFAULTS),
                                        ));
                                    });
                            });
                    }
                });
        });
    });
}

fn checkerboard(canvas: &mut ChildSpawnerCommands) {
    let cell = 20.0;
    for row in 0..(CANVAS.y / cell).ceil() as usize {
        for column in 0..(CANVAS.x / cell).ceil() as usize {
            let grey = if (row + column) % 2 == 0 { 0.92 } else { 0.75 };
            canvas.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(column as f32 * cell),
                    top: Val::Px(row as f32 * cell),
                    width: Val::Px(cell),
                    height: Val::Px(cell),
                    ..default()
                },
                Surface::rounded_rect_fill(0.0, Paint::solid(Color::srgb(grey, grey, grey))),
            ));
        }
    }
}

/// `values` is clip radius, mask radius, and mask opacity in percent.
fn preview_surface(kind: Preview, values: [f32; 3]) -> Surface {
    let stops = vec![
        GradientStop::new(0.0, Color::srgb(0.98, 0.76, 0.18)),
        GradientStop::new(1.0, Color::srgb(0.94, 0.26, 0.32)),
    ];
    let base = Surface::rounded_rect_fill(
        0.0,
        Paint::linear(LinearGradient::angle_degrees(45.0, stops)),
    )
    .uniform_border(4.0, Paint::solid(Color::srgb(0.10, 0.10, 0.12)));
    match kind {
        Preview::Reference => base,
        Preview::Clip => base.with_clip(Clip::rounded_rect(values[0])),
        Preview::Mask => base
            .with_mask(Mask::new(Shape::rounded_rect(values[1])).with_opacity(values[2] / 100.0)),
        Preview::ClipWithShadow => base
            .outer_shadow(
                OuterShadow::new(Color::BLACK)
                    .with_offset(Vec2::new(0.0, 8.0))
                    .with_blur(16.0)
                    .with_opacity(0.6),
            )
            .with_clip(Clip::rounded_rect(values[0])),
    }
}

fn update_previews(
    controls: Query<(&CutControl, &Slider)>,
    mut previews: Query<(&Preview, &mut Surface)>,
) {
    let mut values = DEFAULTS;
    for (control, slider) in &controls {
        values[control.0] = slider.value;
    }
    for (kind, mut surface) in &mut previews {
        let next = preview_surface(*kind, values);
        if *surface != next {
            *surface = next;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn previews_apply_only_their_own_treatment() {
        let values = [40.0, 12.0, 25.0];
        let reference = preview_surface(Preview::Reference, values);
        assert!(reference.clip.is_none() && reference.mask.is_none());
        assert!(reference.effects.outer_shadow.is_none());

        let clip = preview_surface(Preview::Clip, values);
        assert_eq!(clip.clip, Some(Clip::rounded_rect(40.0)));
        assert!(clip.mask.is_none());

        let mask = preview_surface(Preview::Mask, values);
        assert_eq!(
            mask.mask,
            Some(Mask::new(Shape::rounded_rect(12.0)).with_opacity(0.25))
        );
        assert!(mask.clip.is_none());

        let shadowed = preview_surface(Preview::ClipWithShadow, values);
        assert_eq!(shadowed.clip, Some(Clip::rounded_rect(40.0)));
        assert!(shadowed.effects.outer_shadow.is_some());
    }

    #[test]
    fn slider_changes_replace_the_surface_once_and_leave_the_reference_alone() {
        let mut app = App::new();
        app.add_systems(Update, update_previews);
        for index in 0..3 {
            app.world_mut()
                .spawn((CutControl(index), Slider::new(0.0, 100.0).value(10.0)));
        }
        let clip = app
            .world_mut()
            .spawn((Preview::Clip, preview_surface(Preview::Clip, DEFAULTS)))
            .id();
        let reference = app
            .world_mut()
            .spawn((
                Preview::Reference,
                preview_surface(Preview::Reference, DEFAULTS),
            ))
            .id();
        app.update();
        assert_eq!(
            app.world().get::<Surface>(clip).unwrap().clip,
            Some(Clip::rounded_rect(10.0))
        );
        assert_eq!(
            *app.world().get::<Surface>(reference).unwrap(),
            preview_surface(Preview::Reference, DEFAULTS)
        );
    }
}
