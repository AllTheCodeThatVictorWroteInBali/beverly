use bevy::prelude::*;

use super::{ProgressBar, ProgressBarFill};
use crate::rendering::{Paint, Surface};
use crate::theme::{ThemeMode, ThemeResource};

pub fn update_progress_bars(
    time: Res<Time>,
    theme: Res<ThemeResource>,
    mut bars: Query<
        (&ProgressBar, &Children, &mut Surface),
        (With<ProgressBar>, Without<ProgressBarFill>),
    >,
    mut fills: Query<
        (&mut Node, &mut ProgressBarFill, &mut Surface),
        (With<ProgressBarFill>, Without<ProgressBar>),
    >,
) {
    // Same neutral palette as the default alert, button and avatar.
    let (track, fill_color) = match theme.current.mode {
        ThemeMode::Light => (Color::srgb_u8(229, 229, 229), Color::srgb_u8(23, 23, 23)),
        ThemeMode::Dark => (Color::srgb_u8(51, 51, 51), Color::WHITE),
    };

    for (bar, children, mut track_surface) in &mut bars {
        for child in children.iter() {
            let Ok((mut node, mut fill, mut fill_surface)) = fills.get_mut(child) else {
                continue;
            };

            // Smoothly animate toward the target progress.
            let difference = bar.progress - fill.current;

            fill.current += difference * (1.0 - (-bar.animation_speed * time.delta_secs()).exp());

            fill.current = fill.current.clamp(0.0, 1.0);

            node.width = Val::Percent(fill.current * 100.0);

            if bar.use_theme_colors {
                fill_surface.fill = Paint::solid(fill_color);
            }
        }

        if bar.use_theme_colors {
            track_surface.fill = Paint::solid(track);
        }
    }
}
