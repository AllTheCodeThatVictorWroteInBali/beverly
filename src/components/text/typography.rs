use bevy::prelude::*;

use super::font::{FontStyle, FontWeight};
use super::layout::{TextAlignment, TextDirection, TextWrapping};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FontFamily(pub String);

impl FontFamily {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

impl Default for FontFamily {
    fn default() -> Self {
        Self("SFNS".to_string())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LineHeight {
    Normal,
    Px(f32),
    Multiplier(f32),
}

impl Default for LineHeight {
    fn default() -> Self {
        Self::Normal
    }
}

impl LineHeight {
    pub fn resolve(self, font_size: f32) -> f32 {
        match self {
            Self::Normal => font_size * 1.2,
            Self::Px(px) => px.max(0.0),
            Self::Multiplier(multiplier) => (font_size * multiplier).max(0.0),
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct Typography {
    /// Optional override for the rendered text content.
    ///
    /// When None, the current `Text` component remains the source of truth.
    pub content: Option<String>,
    pub family: FontFamily,
    pub font_size: f32,
    pub weight: FontWeight,
    pub style: FontStyle,
    pub color: Color,
    pub alignment: TextAlignment,
    pub line_height: LineHeight,
    pub letter_spacing: f32,
    pub word_spacing: f32,
    pub wrapping: TextWrapping,
    pub direction: TextDirection,
    pub language: Option<String>,
    /// When true, `Typography` drives `Text`/`TextFont`/`TextColor` directly.
    /// When false, legacy Bevy text components remain source-of-truth and
    /// typography metadata mirrors them for layout/shaping caches.
    pub sync_to_bevy: bool,
}

impl Default for Typography {
    fn default() -> Self {
        Self {
            content: None,
            family: FontFamily::default(),
            font_size: 16.0,
            weight: FontWeight::NORMAL,
            style: FontStyle::Normal,
            color: Color::WHITE,
            alignment: TextAlignment::Start,
            line_height: LineHeight::Normal,
            letter_spacing: 0.0,
            word_spacing: 0.0,
            wrapping: TextWrapping::WordWrap,
            direction: TextDirection::Auto,
            language: None,
            sync_to_bevy: false,
        }
    }
}

impl Typography {
    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self.sync_to_bevy = true;
        self
    }

    pub fn with_family(mut self, family: impl Into<String>) -> Self {
        self.family = FontFamily::new(family);
        self.sync_to_bevy = true;
        self
    }

    pub fn with_size(mut self, size: f32) -> Self {
        self.font_size = size.max(1.0);
        self.sync_to_bevy = true;
        self
    }

    pub fn with_weight(mut self, weight: FontWeight) -> Self {
        self.weight = weight;
        self.sync_to_bevy = true;
        self
    }

    pub fn with_style(mut self, style: FontStyle) -> Self {
        self.style = style;
        self.sync_to_bevy = true;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self.sync_to_bevy = true;
        self
    }

    pub fn with_alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self.sync_to_bevy = true;
        self
    }

    pub fn with_line_height(mut self, line_height: LineHeight) -> Self {
        self.line_height = line_height;
        self.sync_to_bevy = true;
        self
    }

    pub fn with_letter_spacing(mut self, letter_spacing: f32) -> Self {
        self.letter_spacing = letter_spacing;
        self.sync_to_bevy = true;
        self
    }

    pub fn with_word_spacing(mut self, word_spacing: f32) -> Self {
        self.word_spacing = word_spacing;
        self.sync_to_bevy = true;
        self
    }

    pub fn with_wrapping(mut self, wrapping: TextWrapping) -> Self {
        self.wrapping = wrapping;
        self.sync_to_bevy = true;
        self
    }

    pub fn with_direction(mut self, direction: TextDirection) -> Self {
        self.direction = direction;
        self.sync_to_bevy = true;
        self
    }

    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self.sync_to_bevy = true;
        self
    }
}

pub struct TypographyPlugin;

impl Plugin for TypographyPlugin {
    fn build(&self, app: &mut App) {
        app.register_required_components::<Text, Typography>();
    }
}
