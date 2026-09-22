use bevy::prelude::*;

use crate::rendering::{Paint, Surface};

/// Configuration for a photo component.
#[derive(Component, Clone)]
pub struct Photo {
    pub image: Handle<Image>,
    pub width: Val,
    pub height: Val,
    pub border_radius: Val,
    pub background_color: Color,
    pub border_color: Color,
}

impl Photo {
    pub fn new(image: Handle<Image>) -> Self {
        Self {
            image,
            width: Val::Px(220.0),
            height: Val::Px(160.0),
            border_radius: Val::Px(24.0),
            background_color: Color::srgba(0.12, 0.15, 0.20, 1.0),
            border_color: Color::srgba(1.0, 1.0, 1.0, 0.14),
        }
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = Val::Px(width);
        self.height = Val::Px(height);
        self
    }

    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = Val::Px(radius);
        self
    }

    pub fn background_color(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }
}

/// Marker for the photo's image node.
#[derive(Component)]
pub struct PhotoImage;

/// Plugin for the photo component.
pub struct PhotoPlugin;

impl Plugin for PhotoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_photos);
    }
}

/// Spawn a photo component.
pub fn spawn_photo(parent: &mut ChildSpawnerCommands, photo: Photo) {
    parent
        .spawn((
            Node {
                width: photo.width,
                height: photo.height,
                overflow: Overflow::clip(),
                padding: UiRect::all(Val::Px(6.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(photo.border_radius),
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderColor::all(Color::NONE),
            Surface::rounded_rect_fill(
                match photo.border_radius {
                    Val::Px(radius) => radius,
                    _ => 24.0,
                },
                Paint::solid(photo.background_color),
            )
            .uniform_border(1.0, Paint::solid(photo.border_color)),
            photo.clone(),
        ))
        .with_children(|parent| {
            parent.spawn((
                ImageNode::new(photo.image),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    border_radius: BorderRadius::all(Val::Px(18.0)),
                    ..default()
                },
                PhotoImage,
            ));
        });
}

fn update_photos(photos: Query<&Photo>, mut images: Query<&mut ImageNode, With<PhotoImage>>) {
    // Reserved for future functionality such as:
    // - lazy loading
    // - image transitions
    // - crossfades
    // - zooming
    // - loading placeholders
    // - image effects
    let _ = (&photos, &mut images);
}
