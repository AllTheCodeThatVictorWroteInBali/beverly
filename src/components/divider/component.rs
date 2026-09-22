use bevy::prelude::*;

use crate::rendering::{Paint, Surface};
use crate::theme::ThemeResource;

pub const DIVIDER_DEFAULT: Color = Color::srgb_u8(226, 232, 240);

/// A reusable UI divider/separator.
///
/// Examples:
///
/// ```ignore
/// commands.spawn(Divider::horizontal().build());
/// commands.spawn(Divider::vertical().build());
/// ```
#[derive(Component, Debug, Clone)]
pub struct Divider {
    /// Direction of the divider.
    pub orientation: DividerOrientation,

    /// Thickness of the divider.
    pub thickness: f32,

    /// Length of the divider.
    ///
    /// For horizontal dividers this is the width.
    /// For vertical dividers this is the height.
    pub length: Val,

    /// Divider color override. If `None`, the divider follows the active theme.
    pub color: Option<Color>,

    /// Optional margin around the divider.
    pub margin: UiRect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DividerOrientation {
    Horizontal,
    Vertical,
}

impl Default for Divider {
    fn default() -> Self {
        Self {
            orientation: DividerOrientation::Horizontal,
            thickness: 1.0,
            length: Val::Percent(100.0),
            color: None,
            margin: UiRect::all(Val::Px(8.0)),
        }
    }
}

impl Divider {
    /// Create a divider that matches the active theme.
    pub fn themed(_theme: &ThemeResource) -> Self {
        Self {
            color: None,
            ..Default::default()
        }
    }

    /// Create a divider with the requested orientation.
    pub fn new(orientation: DividerOrientation) -> Self {
        Self {
            orientation,
            ..Default::default()
        }
    }

    /// Create a horizontal divider.
    pub fn horizontal() -> Self {
        Self::new(DividerOrientation::Horizontal)
    }

    /// Create a vertical divider.
    pub fn vertical() -> Self {
        Self::new(DividerOrientation::Vertical)
    }

    /// Set the divider orientation.
    pub fn orientation(mut self, orientation: DividerOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }

    pub fn length(mut self, length: Val) -> Self {
        self.length = length;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn margin(mut self, margin: UiRect) -> Self {
        self.margin = margin;
        self
    }

    /// Spawn the divider as a Bevy UI node.
    pub fn build(self) -> impl Bundle {
        let color = self.color.unwrap_or(DIVIDER_DEFAULT);

        let style = match self.orientation {
            DividerOrientation::Horizontal => Node {
                width: self.length,
                height: Val::Px(self.thickness),
                margin: self.margin,
                ..default()
            },

            DividerOrientation::Vertical => Node {
                width: Val::Px(self.thickness),
                height: self.length,
                margin: self.margin,
                ..default()
            },
        };

        (self, style, BackgroundColor(Color::NONE), Surface::rounded_rect_fill(0.0, Paint::solid(color)))
    }
}

fn divider_theme_system(
    theme: Res<ThemeResource>,
    mut query: Query<(&Divider, &mut Surface)>,
) {
    let theme_color = theme.current.colors.border;

    for (divider, mut surface) in &mut query {
        surface.fill = Paint::solid(divider.color.unwrap_or(theme_color));
    }
}

pub struct DividerPlugin;

impl Plugin for DividerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, divider_theme_system);
    }
}
