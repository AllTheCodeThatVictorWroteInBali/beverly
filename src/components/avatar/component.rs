use bevy::prelude::*;

use crate::rendering::{Paint, Surface};
use crate::components::text::{TextRole, ThemedText};

/// Marker component for an avatar.
#[derive(Component)]
pub struct Avatar;

const DEFAULT_AVATAR_SIZE: f32 = 48.0;
const MIN_AVATAR_SIZE: f32 = 12.0;
const DEFAULT_INITIALS: &str = "??";

/// How the avatar should be rendered.
#[derive(Clone, Debug)]
pub enum AvatarContent {
    /// Image loaded from an asset path.
    Image(String),

    /// Fallback initials. Intended for 1–3 characters.
    Initials(String),
}

/// Configuration for an avatar.
#[derive(Clone, Debug)]
pub struct AvatarConfig {
    /// Diameter of the avatar in pixels.
    pub size: f32,

    /// Image or initials.
    pub content: AvatarContent,

    /// Background color used for initials.
    pub background_color: Color,

    /// Text color used for initials.
    pub text_color: Color,

    /// Optional font size used for initials.
    ///
    /// If not set, the value is derived from the avatar size.
    pub font_size: Option<f32>,
}

impl Default for AvatarConfig {
    fn default() -> Self {
        Self {
            size: DEFAULT_AVATAR_SIZE,
            content: AvatarContent::Initials(DEFAULT_INITIALS.to_string()),
            background_color: Color::srgb(0.25, 0.25, 0.30),
            text_color: Color::WHITE,
            font_size: None,
        }
    }
}

impl AvatarConfig {
    pub fn image(path: impl Into<String>) -> Self {
        Self {
            content: AvatarContent::Image(path.into()),
            ..default()
        }
    }

    pub fn initials(initials: impl Into<String>) -> Self {
        let initials = initials.into();

        Self {
            content: AvatarContent::Initials(normalize_initials(&initials)),
            ..default()
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }

    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = Some(size);
        self
    }
}

/// Spawn a reusable avatar with `Commands`.
///
/// Usage
///
/// Image avatar:
/// ```no_run
/// # use bevy::prelude::*;
/// # use frontend::ui::avatar::{spawn_avatar, AvatarConfig};
/// # fn demo(mut commands: Commands, asset_server: Res<AssetServer>) {
/// spawn_avatar(
///     &mut commands,
///     &asset_server,
///     AvatarConfig::image("avatars/victor.png").size(64.0),
/// );
/// # }
/// ```
///
/// Initials:
/// ```no_run
/// # use bevy::prelude::*;
/// # use frontend::ui::avatar::{spawn_avatar, AvatarConfig};
/// # fn demo(mut commands: Commands, asset_server: Res<AssetServer>) {
/// spawn_avatar(
///     &mut commands,
///     &asset_server,
///     AvatarConfig::initials("VV").size(48.0),
/// );
/// # }
/// ```
///
/// You can also customize the fallback:
/// ```no_run
/// # use bevy::prelude::*;
/// # use frontend::ui::avatar::{spawn_avatar, AvatarConfig};
/// # fn demo(mut commands: Commands, asset_server: Res<AssetServer>) {
/// spawn_avatar(
///     &mut commands,
///     &asset_server,
///     AvatarConfig::initials("ABC")
///         .size(56.0)
///         .background(Color::srgb(0.15, 0.45, 0.75))
///         .text_color(Color::WHITE)
///         .font_size(20.0),
/// );
/// # }
/// ```
pub fn spawn_avatar(
    commands: &mut Commands,
    asset_server: &AssetServer,
    config: AvatarConfig,
) -> Entity {
    let mut avatar = commands.spawn_empty();
    configure_avatar(&mut avatar, asset_server, config);
    avatar.id()
}

/// Spawn a reusable avatar as a child node.
pub fn spawn_avatar_in(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    config: AvatarConfig,
) -> Entity {
    let mut avatar = parent.spawn_empty();
    configure_avatar(&mut avatar, asset_server, config);
    avatar.id()
}

fn configure_avatar(avatar: &mut EntityCommands, asset_server: &AssetServer, config: AvatarConfig) {
    let size = config.size.max(MIN_AVATAR_SIZE);
    let font_size = config
        .font_size
        .unwrap_or((size * 0.38).round().clamp(12.0, 32.0));

    avatar.insert((
        Avatar,
        Node {
            width: Val::Px(size),
            height: Val::Px(size),
            min_width: Val::Px(size),
            min_height: Val::Px(size),
            max_width: Val::Px(size),
            max_height: Val::Px(size),
            border_radius: BorderRadius::all(px(size / 2.0)),

            // Center fallback initials.
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,

            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(Color::NONE),
        Surface::rounded_rect_fill(size / 2.0, Paint::solid(config.background_color)),
    ));

    match config.content {
        AvatarContent::Image(path) => {
            let image = asset_server.load(path);

            avatar.insert(ImageNode { image, ..default() });
        }

        AvatarContent::Initials(initials) => {
            avatar.with_children(|parent| {
                parent.spawn((
                    ThemedText::new(TextRole::Label),
                    Text::new(normalize_initials(&initials)),
                    TextFont {
                        font_size: FontSize::Px(font_size),
                        ..default()
                    },
                    TextColor(config.text_color),
                ));
            });
        }
    }
}

/// Keep avatars within the intended 1–3 character range.
fn normalize_initials(initials: &str) -> String {
    initials
        .chars()
        .filter(|c| !c.is_whitespace())
        .take(3)
        .collect::<String>()
        .to_uppercase()
}
