use bevy::prelude::*;

use crate::components::text::{FontWeight, Typography};
use crate::rendering::{Paint, Surface};
use crate::theme::{ThemeMode, ThemeResource};

/// Marker component for an avatar.
#[derive(Component)]
pub struct Avatar;

/// Initials avatar whose unset colors follow the active theme mode.
#[derive(Component)]
struct AvatarThemed {
    radius: f32,
    background: Option<Color>,
    text: Option<Color>,
    border: Option<Color>,
}

#[derive(Component)]
struct AvatarInitials;

pub struct AvatarPlugin;

impl Plugin for AvatarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_avatar_theme);
    }
}

fn apply_avatar_theme(
    theme: Res<ThemeResource>,
    mut avatars: Query<(Ref<AvatarThemed>, &mut Surface, Option<&Children>)>,
    mut texts: Query<&mut Typography, With<AvatarInitials>>,
) {
    let theme_changed = theme.is_changed();
    // Same fill/border as the default alert.
    let (fill, text, border) = match theme.current.mode {
        ThemeMode::Light => LIGHT_COLORS,
        ThemeMode::Dark => DARK_COLORS,
    };

    for (themed, mut surface, children) in &mut avatars {
        if !theme_changed && !themed.is_added() {
            continue;
        }

        let fill = themed.background.unwrap_or(fill);
        let text = themed.text.unwrap_or(text);
        let border = themed.border.unwrap_or(border);
        *surface = Surface::rounded_rect_fill(themed.radius, Paint::solid(fill))
            .uniform_border(1.0, Paint::solid(border));

        for child in children.into_iter().flatten() {
            if let Ok(mut typography) = texts.get_mut(*child) {
                typography.color = text;
            }
        }
    }
}

const DEFAULT_AVATAR_SIZE: f32 = 48.0;
const MIN_AVATAR_SIZE: f32 = 12.0;
const DEFAULT_INITIALS: &str = "??";

/// (fill, text, border) for initials; same fill/border as the default alert.
type AvatarColors = (Color, Color, Color);
const LIGHT_COLORS: AvatarColors = (
    Color::WHITE,
    Color::BLACK,
    Color::srgb(0.831, 0.831, 0.831),
);
const DARK_COLORS: AvatarColors = (
    Color::srgb(0.090, 0.090, 0.090),
    Color::WHITE,
    Color::srgb(0.149, 0.149, 0.149),
);

/// Named avatar sizes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AvatarSize {
    Xs,
    S,
    M,
    L,
    Xl,
}

impl AvatarSize {
    pub fn px(self) -> f32 {
        match self {
            Self::Xs => 24.0,
            Self::S => 32.0,
            Self::M => DEFAULT_AVATAR_SIZE,
            Self::L => 64.0,
            Self::Xl => 96.0,
        }
    }
}

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

    /// Background color used for initials. `None` follows the theme mode.
    pub background_color: Option<Color>,

    /// Text color used for initials. `None` follows the theme mode.
    pub text_color: Option<Color>,

    /// Border color used for initials. `None` follows the theme mode.
    pub border_color: Option<Color>,

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
            background_color: None,
            text_color: None,
            border_color: None,
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

    pub fn sized(self, size: AvatarSize) -> Self {
        self.size(size.px())
    }

    /// Fixed light look (white, black text) regardless of theme mode.
    pub fn light(self) -> Self {
        let (fill, text, border) = LIGHT_COLORS;
        self.background(fill).text_color(text).border(border)
    }

    /// Fixed dark look (black, white text) regardless of theme mode.
    pub fn dark(self) -> Self {
        let (fill, text, border) = DARK_COLORS;
        self.background(fill).text_color(text).border(border)
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = Some(color);
        self
    }

    pub fn border(mut self, color: Color) -> Self {
        self.border_color = Some(color);
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
        .unwrap_or((size * 0.5).round().max(12.0));

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
        Surface::rounded_rect_fill(
            size / 2.0,
            Paint::solid(config.background_color.unwrap_or(Color::NONE)),
        ),
    ));

    match config.content {
        AvatarContent::Image(path) => {
            let image = asset_server.load(path);

            avatar.insert(ImageNode { image, ..default() });
        }

        AvatarContent::Initials(initials) => {
            avatar.insert(AvatarThemed {
                radius: size / 2.0,
                background: config.background_color,
                text: config.text_color,
                border: config.border_color,
            });
            avatar.with_children(|parent| {
                parent.spawn((
                    AvatarInitials,
                    Typography::default()
                        .with_size(font_size)
                        .with_weight(FontWeight::BOLD)
                        .with_color(config.text_color.unwrap_or(Color::BLACK)),
                    Text::new(normalize_initials(&initials)),
                    TextFont {
                        font_size: FontSize::Px(font_size),
                        ..default()
                    },
                    TextColor(config.text_color.unwrap_or(Color::BLACK)),
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
