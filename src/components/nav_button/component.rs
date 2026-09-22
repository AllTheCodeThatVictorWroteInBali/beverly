use bevy::prelude::*;

use crate::icons::{Icon, IconCommands};
use crate::primitives::root::UiFonts;
use crate::rendering::{Paint, Surface};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageId {
    Music,
    Videos,
    Photos,
    Podcasts,
    Playlists,
    PlaylistDetail,
    Settings,
    Radio,
}

impl PageId {
    pub fn all() -> [Self; 7] {
        [
            Self::Music,
            Self::Videos,
            Self::Photos,
            Self::Podcasts,
            Self::Playlists,
            Self::Radio,
            Self::Settings,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Music => "Music",
            Self::Videos => "Videos",
            Self::Photos => "Photos",
            Self::Podcasts => "Podcasts",
            Self::Playlists => "Playlists",
            Self::PlaylistDetail => "Playlist Detail",
            Self::Settings => "Settings",
            Self::Radio => "Radio",
        }
    }

    pub fn body(self) -> &'static str {
        match self {
            Self::Music => "A dummy music workspace with a future playlist experience.",
            Self::Videos => "A dummy video view for browsing clips and playlists.",
            Self::Photos => "A dummy photo gallery for organizing recent captures.",
            Self::Podcasts => "A dummy podcast feed ready for episode browsing.",
            Self::Playlists => "A dummy playlist page for your next listening session.",
            Self::PlaylistDetail => "Songs in a selected playlist.",
            Self::Settings => "Personalize your workspace, playback, and account preferences.",
            Self::Radio => "Tune your favorite station presets and listening modes.",
        }
    }

    pub fn icon(self) -> Icon {
        match self {
            Self::Music => Icon::feather("music"),
            Self::Videos => Icon::feather("play"),
            Self::Photos => Icon::feather("image"),
            Self::Podcasts => Icon::feather("radio"),
            Self::Playlists => Icon::feather("list"),
            Self::PlaylistDetail => Icon::feather("disc"),
            Self::Settings => Icon::feather("settings"),
            Self::Radio => Icon::feather("radio"),
        }
    }

    pub fn accent(self) -> Color {
        match self {
            Self::Music => Color::srgb(0.16, 0.42, 0.78),
            Self::Videos => Color::srgb(0.78, 0.32, 0.18),
            Self::Photos => Color::srgb(0.22, 0.72, 0.36),
            Self::Podcasts => Color::srgb(0.68, 0.20, 0.72),
            Self::Playlists => Color::srgb(0.90, 0.62, 0.16),
            Self::PlaylistDetail => Color::srgb(0.84, 0.48, 0.22),
            Self::Settings => Color::srgb(0.40, 0.42, 0.46),
            Self::Radio => Color::srgb(0.10, 0.76, 0.68),
        }
    }
}

#[derive(Component)]
pub struct NavButton {
    pub page: PageId,
}

#[derive(Component)]
pub struct DrawerToggle;

#[derive(Component)]
pub struct DrawerLabel;

#[derive(Component, Clone, Copy)]
pub struct DrawerNavLabel {
    pub page: PageId,
}

#[derive(Component)]
pub struct DrawerToggleLabel;

#[derive(Component)]
pub struct DrawerIcon;

#[derive(Component, Clone, Copy)]
pub struct DrawerNavIcon {
    pub page: PageId,
}

#[derive(Component)]
pub struct DrawerButton;

pub fn drawer_toggle(parent: &mut ChildSpawnerCommands, ui_fonts: &UiFonts) {
    parent
        .spawn((
            Button,
            DrawerButton,
            DrawerToggle,
            Node {
                width: percent(100),
                height: px(58.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                padding: UiRect::horizontal(px(18.0)),
                border_radius: BorderRadius::all(px(8.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            Surface::rounded_rect_fill(8.0, Paint::solid(Color::NONE))
                .uniform_border(1.0, Paint::solid(Color::NONE)),
        ))
        .with_children(|button| {
            button
                .spawn((
                    DrawerIcon,
                    Node {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                ))
                .with_children(|icon| {
                    icon.spawn_icon_sized(Icon::feather("menu"), 22.0);
                });

            button.spawn((
                DrawerLabel,
                DrawerToggleLabel,
                Text::new("Menu"),
                TextFont {
                    font: FontSource::Handle(ui_fonts.text.clone()),
                    font_size: FontSize::Px(22.0),
                    ..default()
                },
                Node {
                    margin: UiRect::left(px(14)),
                    ..default()
                },
            ));
        });
}

pub fn nav_button(parent: &mut ChildSpawnerCommands, page: PageId, ui_fonts: &UiFonts) {
    parent
        .spawn((
            Button,
            DrawerButton,
            Node {
                width: percent(100),
                height: px(58.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                padding: UiRect::horizontal(px(18.0)),
                border_radius: BorderRadius::all(px(8.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            Surface::rounded_rect_fill(8.0, Paint::solid(Color::NONE))
                .uniform_border(1.0, Paint::solid(Color::NONE)),
            NavButton { page },
        ))
        .with_children(|button| {
            button
                .spawn((
                    DrawerIcon,
                    Node {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                ))
                .with_children(|icon| {
                    let icon_entity = icon.spawn_icon_colored(page.icon(), 22.0, Color::NONE);
                    icon.commands()
                        .entity(icon_entity)
                        .insert(DrawerNavIcon { page });
                });

            button.spawn((
                DrawerLabel,
                DrawerNavLabel { page },
                Text::new(page.label()),
                TextFont {
                    font: FontSource::Handle(ui_fonts.text.clone()),
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                Node {
                    margin: UiRect::left(px(14)),
                    ..default()
                },
            ));
        });
}

#[cfg(test)]
mod tests {
    use super::PageId;

    #[test]
    fn page_registry_includes_radio_scene() {
        let pages = PageId::all();
        assert!(pages.contains(&PageId::Settings));
        assert!(pages.iter().any(|page| page.label() == "Radio"));
    }
}
