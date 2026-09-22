use bevy::prelude::*;

use crate::rendering::{Paint, Surface};

#[derive(Component, Clone, Debug)]
pub struct Badge {
    pub text: String,
    pub size: f32,
    pub text_color: Color,
    pub background_color: Color,
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
            padding_x: 10.0,
            padding_y: 4.0,
            border_radius: 999.0,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
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
            Surface::rounded_rect_fill(badge.border_radius, Paint::solid(badge.background_color)),
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
