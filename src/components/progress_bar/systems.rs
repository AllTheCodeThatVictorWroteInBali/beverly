use bevy::prelude::*;

use crate::rendering::{Paint, Surface};
use super::{ProgressBar, ProgressBarFill};
use crate::theme::ThemeResource;

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
    let colors = theme.current.colors;

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
                fill_surface.fill = Paint::solid(colors.primary);
            }
        }

        if bar.use_theme_colors {
            track_surface.fill = Paint::solid(colors.surface_elevated);
        }
    }
}
