use bevy::prelude::*;

use crate::rendering::{Paint, Surface};

/// Named badge sizes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BadgeSize {
    Xs,
    S,
    M,
    L,
    Xl,
}

impl BadgeSize {
    /// (font size, horizontal padding, vertical padding)
    fn metrics(self) -> (f32, f32, f32) {
        match self {
            Self::Xs => (10.0, 6.0, 2.0),
            Self::S => (12.0, 8.0, 3.0),
            Self::M => (14.0, 10.0, 4.0),
            Self::L => (16.0, 12.0, 5.0),
            Self::Xl => (20.0, 14.0, 6.0),
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct Badge {
    pub text: String,
    pub size: f32,
    pub text_color: Color,
    pub background_color: Color,
    pub border_color: Option<Color>,
    pub padding_x: f32,
    pub padding_y: f32,
    pub border_radius: f32,
}

impl Badge {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            size: 14.0,
            text_color: Color::WHITE,
            background_color: Color::srgb(0.20, 0.20, 0.25),
            border_color: None,
            padding_x: 10.0,
            padding_y: 4.0,
            border_radius: 999.0,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn sized(mut self, size: BadgeSize) -> Self {
        let (font, x, y) = size.metrics();
        self.size = font;
        self.padding_x = x;
        self.padding_y = y;
        self
    }

    pub fn border(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

    /// White badge with black text and a light grey border.
    pub fn light(self) -> Self {
        self.background_color(Color::WHITE)
            .text_color(Color::BLACK)
            .border(Color::srgb(0.831, 0.831, 0.831))
    }

    /// `#171717` badge with white text and a `#262626` border.
    pub fn dark(self) -> Self {
        self.background_color(Color::srgb(0.090, 0.090, 0.090))
            .text_color(Color::WHITE)
            .border(Color::srgb(0.149, 0.149, 0.149))
    }

    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    pub fn background_color(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }

    pub fn padding(mut self, x: f32, y: f32) -> Self {
        self.padding_x = x;
        self.padding_y = y;
        self
    }

    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }
}

#[derive(Component)]
pub struct BadgeRoot;

#[derive(Component)]
pub struct BadgeLabel;

pub fn spawn_badge(parent: &mut ChildSpawnerCommands, badge: Badge, font: Handle<Font>) -> Entity {
    let mut surface =
        Surface::rounded_rect_fill(badge.border_radius, Paint::solid(badge.background_color));
    if let Some(border) = badge.border_color {
        surface = surface.uniform_border(1.0, Paint::solid(border));
    }

    parent
        .spawn((
            BadgeRoot,
            badge.clone(),
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::axes(px(badge.padding_x), px(badge.padding_y)),
                border_radius: BorderRadius::all(px(badge.border_radius)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            surface,
        ))
        .with_children(|container| {
            container.spawn((
                BadgeLabel,
                Text::new(badge.text),
                TextFont {
                    font: FontSource::Handle(font),
                    font_size: FontSize::Px(badge.size),
                    ..default()
                },
                TextColor(badge.text_color),
            ));
        })
        .id()
}
