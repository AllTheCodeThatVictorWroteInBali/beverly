use bevy::prelude::*;

use crate::rendering::{Paint, Surface};
use crate::theme::ThemeResource;

/// Event emitted when a [`Link`] is clicked.
#[derive(Message, Debug, Clone)]
pub struct LinkClicked {
    pub entity: Entity,
}

/// Marker component for a reusable link.
#[derive(Component, Debug, Clone)]
pub struct Link {
    pub text: String,
    pub icon: Option<String>,
    pub disabled: bool,
}

impl Link {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            icon: None,
            disabled: false,
        }
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Optional child marker for the text portion.
#[derive(Component)]
pub struct LinkText;

/// Optional child marker for the icon portion.
#[derive(Component)]
pub struct LinkIcon;

/// Plugin providing link behavior.
pub struct LinkPlugin;

impl Plugin for LinkPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<LinkClicked>()
            .add_systems(Update, (link_interaction_system, link_visual_system));
    }
}

fn link_interaction_system(
    mut interaction_query: Query<(Entity, &Interaction, &Link), (Changed<Interaction>, With<Link>)>,
    mut events: MessageWriter<LinkClicked>,
) {
    for (entity, interaction, link) in &mut interaction_query {
        if link.disabled {
            continue;
        }

        if *interaction == Interaction::Pressed {
            events.write(LinkClicked { entity });
        }
    }
}

fn link_visual_system(
    theme: Res<ThemeResource>,
    links: Query<(Entity, &Link, Option<&Interaction>)>,
    mut backgrounds: Query<&mut Surface, With<Link>>,
    mut text_colors: Query<(&ChildOf, &mut TextColor), With<LinkText>>,
) {
    let colors = theme.current.colors;

    for (entity, link, interaction) in &links {
        if let Ok(mut surface) = backgrounds.get_mut(entity) {
            surface.fill = if link.disabled {
                Paint::solid(Color::srgba(0.0, 0.0, 0.0, 0.0))
            } else {
                match interaction.copied().unwrap_or(Interaction::None) {
                    Interaction::Pressed => Paint::solid(Color::srgba(0.0, 0.0, 0.0, 0.20)),
                    Interaction::Hovered => Paint::solid(Color::srgba(0.0, 0.0, 0.0, 0.10)),
                    Interaction::None => Paint::solid(Color::srgba(0.0, 0.0, 0.0, 0.06)),
                }
            };
        }

        for (parent, mut text_color) in &mut text_colors {
            if parent.parent() != entity {
                continue;
            }

            text_color.0 = if link.disabled {
                colors.text_disabled
            } else if matches!(interaction.copied(), Some(Interaction::Hovered)) {
                colors.primary_hover
            } else if matches!(interaction.copied(), Some(Interaction::Pressed)) {
                colors.primary_active
            } else {
                colors.primary
            };
        }
    }
}
