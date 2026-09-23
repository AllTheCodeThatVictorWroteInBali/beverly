use bevy::prelude::*;

use crate::components::text::{HighlightableText, Typography};
use crate::theme::{ThemeColors, ThemeResource, ThemeTypography};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleLevel {
    Display,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

#[derive(Component, Clone, Copy, Debug)]
#[require(HighlightableText)]
pub struct ThemedTitle {
    pub level: TitleLevel,
    pub size_override: Option<f32>,
    pub color_override: Option<Color>,
}

impl ThemedTitle {
    pub fn new(level: TitleLevel) -> Self {
        Self {
            level,
            size_override: None,
            color_override: None,
        }
    }

    pub fn size(mut self, px: f32) -> Self {
        self.size_override = Some(px);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color_override = Some(color);
        self
    }
}

fn themed_title_system(
    theme: Res<ThemeResource>,
    mut query: Query<(&ThemedTitle, &mut TextFont, &mut TextColor, Option<&Typography>)>,
) {
    let colors = theme.current.colors;
    let typography = theme.current.typography;

    for (title, mut font, mut color, custom_typography) in &mut query {
        // Text always carries a Typography component (required component); only defer to it
        // once the caller has explicitly opted in via a `with_*` builder (sync_to_bevy = true).
        if custom_typography.is_some_and(|t| t.sync_to_bevy) {
            continue;
        }

        let new_size = FontSize::Px(
            title
                .size_override
                .unwrap_or(level_size(title.level, typography)),
        );
        let new_color = title
            .color_override
            .unwrap_or(level_color(title.level, colors));

        // Only write on actual change: an unconditional write here marks TextFont/TextColor
        // Changed every frame, which triggers a full text relayout and tears down the
        // in-progress selection highlight overlay 60 times a second.
        if font.font_size != new_size {
            font.font_size = new_size;
        }
        if color.0 != new_color {
            color.0 = new_color;
        }
    }
}

fn level_size(level: TitleLevel, typography: ThemeTypography) -> f32 {
    let base = typography.font_size_title;

    match level {
        TitleLevel::Display => base * 1.75,
        TitleLevel::H1 => base * 1.45,
        TitleLevel::H2 => base * 1.20,
        TitleLevel::H3 => base,
        TitleLevel::H4 => base * 0.88,
        TitleLevel::H5 => base * 0.78,
        TitleLevel::H6 => base * 0.70,
    }
}

fn level_color(level: TitleLevel, colors: ThemeColors) -> Color {
    match level {
        TitleLevel::Display | TitleLevel::H1 | TitleLevel::H2 | TitleLevel::H3 => colors.text,
        TitleLevel::H4 | TitleLevel::H5 => colors.text_muted,
        TitleLevel::H6 => colors.text_disabled,
    }
}

pub struct TitlePlugin;

impl Plugin for TitlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, themed_title_system);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::text::{ThemedTextPlugin, Typography};
    use crate::theme::ThemeResource;
    use bevy::asset::AssetPlugin;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default(), ThemedTextPlugin, TitlePlugin));
        app.init_resource::<ThemeResource>();
        app
    }

    #[test]
    fn title_level_drives_font_size_by_default() {
        let mut app = app();
        let entity = app
            .world_mut()
            .spawn((Text::new("Workspace"), ThemedTitle::new(TitleLevel::H1)))
            .id();

        app.update();

        let font = app.world().get::<TextFont>(entity).unwrap();
        let expected = level_size(TitleLevel::H1, app.world().resource::<ThemeResource>().current.typography);
        assert_eq!(font.font_size, FontSize::Px(expected));
    }

    #[test]
    fn explicit_typography_override_takes_over_size_and_family() {
        let mut app = app();
        let entity = app
            .world_mut()
            .spawn((
                Text::new("Workspace"),
                ThemedTitle::new(TitleLevel::H1),
                Typography::default().with_size(48.0).with_family("SFNS"),
            ))
            .id();

        app.update();
        app.update();

        let font = app.world().get::<TextFont>(entity).unwrap();
        assert_eq!(font.font_size, FontSize::Px(48.0));
    }
}
