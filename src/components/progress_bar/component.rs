use bevy::prelude::*;

use crate::rendering::{Paint, Surface};

/// Main progress bar component.
///
/// `progress` is always expected to be between 0.0 and 1.0.
#[derive(Component)]
pub struct ProgressBar {
    pub progress: f32,

    pub width: f32,
    pub height: f32,

    pub background_color: Color,
    pub fill_color: Color,
    pub use_theme_colors: bool,

    /// How quickly the visual fill catches up
    /// to the target progress.
    pub animation_speed: f32,
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self {
            progress: 0.0,

            width: 300.0,
            height: 8.0,

            background_color: Color::srgb(0.15, 0.15, 0.17),
            fill_color: Color::srgb(0.3, 0.7, 1.0),
            use_theme_colors: true,

            animation_speed: 8.0,
        }
    }
}

impl ProgressBar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn progress(mut self, value: f32) -> Self {
        self.progress = value.clamp(0.0, 1.0);
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn background_color(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }

    pub fn fill_color(mut self, color: Color) -> Self {
        self.fill_color = color;
        self.use_theme_colors = false;
        self
    }

    pub fn use_theme_colors(mut self, enabled: bool) -> Self {
        self.use_theme_colors = enabled;
        self
    }

    pub fn animation_speed(mut self, speed: f32) -> Self {
        self.animation_speed = speed;
        self
    }
}

/// Marks the actual visual fill portion of a progress bar.
#[derive(Component)]
pub struct ProgressBarFill {
    pub current: f32,
}

pub fn spawn_progress_bar(commands: &mut Commands, progress_bar: ProgressBar) -> Entity {
    let width = progress_bar.width;
    let height = progress_bar.height;

    let background_color = progress_bar.background_color;
    let fill_color = progress_bar.fill_color;
    let progress = progress_bar.progress;

    commands
        .spawn((
            progress_bar,
            crate::primitives::a11y::progress_node("Progress", progress),
            Node {
                width: Val::Px(width),
                height: Val::Px(height),
                border_radius: BorderRadius::all(Val::Px(height / 2.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            Surface::rounded_rect_fill(height / 2.0, Paint::solid(background_color)),
        ))
        .with_children(|parent| {
            parent.spawn((
                ProgressBarFill { current: 0.0 },
                Node {
                    width: Val::Px(0.0),
                    height: Val::Percent(100.0),
                    border_radius: BorderRadius::all(Val::Px(height / 2.0)),
                    ..default()
                },
                Surface::rounded_rect_fill(height / 2.0, Paint::solid(fill_color)),
            ));
        })
        .id()
}
