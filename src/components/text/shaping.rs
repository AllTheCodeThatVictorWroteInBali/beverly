use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use bevy::prelude::*;
use unicode_bidi::BidiInfo;
use unicode_script::{Script, UnicodeScript};

use super::font::{ResolvedFontFace, TypographyFontManager};
use super::layout::TextDirection;
use super::typography::Typography;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GlyphCluster {
    pub cluster_index: usize,
    pub byte_start: usize,
    pub byte_end: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShapedGlyph {
    pub glyph_id: u32,
    pub advance: f32,
    pub offset: Vec2,
    pub cluster: GlyphCluster,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShapedText {
    pub direction: TextDirection,
    pub script: Option<String>,
    pub language: Option<String>,
    pub glyphs: Vec<ShapedGlyph>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TextShapingKey {
    pub content_hash: u64,
    pub family: String,
    pub font_size_bits: u32,
    pub weight: u16,
    pub style: u8,
    pub letter_spacing_bits: u32,
    pub word_spacing_bits: u32,
    pub direction: TextDirection,
    pub language: Option<String>,
}

#[derive(Resource, Default)]
pub struct TextShapingCache {
    pub runs: HashMap<TextShapingKey, ShapedText>,
    pub hits: u64,
    pub misses: u64,
}

#[derive(Component, Clone, Debug, Default)]
pub struct TextShapingResult(pub Option<ShapedText>);

impl TextShapingKey {
    pub fn from_text(content: &str, typography: &Typography) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        content.hash(&mut hasher);

        let style = match typography.style {
            super::font::FontStyle::Normal => 0,
            super::font::FontStyle::Italic => 1,
            super::font::FontStyle::Oblique => 2,
        };

        Self {
            content_hash: hasher.finish(),
            family: typography.family.0.clone(),
            font_size_bits: typography.font_size.to_bits(),
            weight: typography.weight.0,
            style,
            letter_spacing_bits: typography.letter_spacing.to_bits(),
            word_spacing_bits: typography.word_spacing.to_bits(),
            direction: typography.direction,
            language: typography.language.clone(),
        }
    }
}

fn content_from_text(text: &Text, typography: &Typography) -> String {
    typography
        .content
        .clone()
        .unwrap_or_else(|| text.0.clone())
}

fn script_hint(content: &str) -> Option<String> {
    for ch in content.chars() {
        let script = ch.script();
        if !matches!(script, Script::Common | Script::Inherited | Script::Unknown) {
            return Some(format!("{script:?}").to_lowercase());
        }
    }

    None
}

fn resolve_direction(content: &str, requested: TextDirection) -> TextDirection {
    match requested {
        TextDirection::Auto => {
            let bidi = BidiInfo::new(content, None);
            if bidi
                .paragraphs
                .first()
                .map(|paragraph| paragraph.level.is_rtl())
                .unwrap_or(false)
            {
                TextDirection::Rtl
            } else {
                TextDirection::Ltr
            }
        }
        other => other,
    }
}

fn rustybuzz_direction(direction: TextDirection) -> rustybuzz::Direction {
    match direction {
        TextDirection::Rtl => rustybuzz::Direction::RightToLeft,
        TextDirection::Ltr | TextDirection::Auto => rustybuzz::Direction::LeftToRight,
    }
}

fn cluster_index_for_byte(content: &str, byte_start: usize) -> usize {
    content[..byte_start.min(content.len())].chars().count()
}

fn fallback_shape(content: &str, typography: &Typography, direction: TextDirection) -> ShapedText {
    let mut glyphs = Vec::with_capacity(content.chars().count());

    for (index, (byte_index, ch)) in content.char_indices().enumerate() {
        let cluster = GlyphCluster {
            cluster_index: index,
            byte_start: byte_index,
            byte_end: byte_index + ch.len_utf8(),
        };

        let mut advance = typography.font_size * 0.56;
        if ch == ' ' {
            advance += typography.word_spacing;
        }
        advance += typography.letter_spacing;

        glyphs.push(ShapedGlyph {
            glyph_id: ch as u32,
            advance,
            offset: Vec2::ZERO,
            cluster,
        });
    }

    ShapedText {
        direction,
        script: script_hint(content),
        language: typography.language.clone(),
        glyphs,
    }
}

fn shape_with_rustybuzz(
    content: &str,
    typography: &Typography,
    bytes: &[u8],
    direction: TextDirection,
) -> Option<ShapedText> {
    let face = rustybuzz::Face::from_slice(bytes, 0)?;
    let upem = face.units_per_em() as f32;
    let scale = typography.font_size.max(1.0) / upem.max(1.0);

    let mut buffer = rustybuzz::UnicodeBuffer::new();
    buffer.push_str(content);
    buffer.set_direction(rustybuzz_direction(direction));

    let glyph_buffer = rustybuzz::shape(&face, &[], buffer);
    let infos = glyph_buffer.glyph_infos();
    let positions = glyph_buffer.glyph_positions();

    let mut glyphs = Vec::with_capacity(infos.len());
    for (index, info) in infos.iter().enumerate() {
        let position = positions[index];
        let byte_start = (info.cluster as usize).min(content.len());
        let byte_end = infos
            .get(index + 1)
            .map(|next| (next.cluster as usize).min(content.len()))
            .unwrap_or(content.len())
            .max(byte_start);
        let cluster_index = cluster_index_for_byte(content, byte_start);
        let cluster_text = &content[byte_start..byte_end];

        let mut advance = position.x_advance as f32 * scale;
        if cluster_text.chars().all(|ch| ch.is_whitespace()) {
            advance += typography.word_spacing;
        } else if advance > 0.0 {
            advance += typography.letter_spacing;
        }

        glyphs.push(ShapedGlyph {
            glyph_id: info.glyph_id,
            advance,
            offset: Vec2::new(position.x_offset as f32 * scale, -(position.y_offset as f32 * scale)),
            cluster: GlyphCluster {
                cluster_index,
                byte_start,
                byte_end,
            },
        });
    }

    Some(ShapedText {
        direction,
        script: script_hint(content),
        language: typography.language.clone(),
        glyphs,
    })
}

fn ensure_shaping_result_component(world: &mut World) {
    let entities: Vec<Entity> = {
        let mut query = world.query_filtered::<Entity, (With<Typography>, Without<TextShapingResult>)>();
        query.iter(world).collect()
    };

    for entity in entities {
        if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
            entity_mut.insert(TextShapingResult::default());
        }
    }
}

fn shape_text_system(
    mut cache: ResMut<TextShapingCache>,
    font_manager: Res<TypographyFontManager>,
    query: Query<(&Text, &Typography, Option<&ResolvedFontFace>, &mut TextShapingResult)>,
) {
    for (text, typography, resolved_face, mut result) in query {
        let content = content_from_text(text, typography);
        let direction = resolve_direction(&content, typography.direction);
        let mut key = TextShapingKey::from_text(&content, typography);
        key.direction = direction;

        if let Some(face) = resolved_face {
            key.family = face.family.clone();
        }

        let shaped = if let Some(cached) = cache.runs.get(&key).cloned() {
            cache.hits += 1;
            cached
        } else {
            cache.misses += 1;
            let shaped = resolved_face
                .and_then(|resolved| font_manager.face_by_asset_path(&resolved.asset_path))
                .and_then(|face| {
                    if face.bytes.is_empty() {
                        None
                    } else {
                        shape_with_rustybuzz(&content, typography, face.bytes.as_slice(), direction)
                    }
                })
                .unwrap_or_else(|| fallback_shape(&content, typography, direction));
            cache.runs.insert(key, shaped.clone());
            shaped
        };

        if result.0.as_ref() != Some(&shaped) {
            result.0 = Some(shaped);
        }
    }
}

pub struct TypographyShapingPlugin;

impl Plugin for TypographyShapingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TextShapingCache>().add_systems(
            Update,
            (ensure_shaping_result_component, shape_text_system).chain(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::text::font::{FontStyle, FontWeight};
    use crate::components::text::layout::{TextAlignment, TextDirection, TextWrapping};
    use crate::components::text::typography::{FontFamily, LineHeight, Typography};

    fn typography() -> Typography {
        Typography {
            content: None,
            family: FontFamily::new("SFNS"),
            font_size: 16.0,
            weight: FontWeight::NORMAL,
            style: FontStyle::Normal,
            color: Color::WHITE,
            alignment: TextAlignment::Start,
            line_height: LineHeight::Normal,
            letter_spacing: 0.5,
            word_spacing: 1.25,
            wrapping: TextWrapping::WordWrap,
            direction: TextDirection::Auto,
            language: Some("en".to_string()),
            sync_to_bevy: false,
        }
    }

    #[test]
    fn shape_keeps_unicode_clusters() {
        let style = typography();
        let shaped = fallback_shape("Bonjour le monde こんにちは مرحبا", &style, TextDirection::Ltr);

        assert!(shaped.glyphs.len() >= 10);
        assert_eq!(shaped.language.as_deref(), Some("en"));
        assert!(shaped.script.is_some());
    }

    #[test]
    fn word_spacing_affects_space_advance() {
        let style = typography();
        let shaped = fallback_shape("A A", &style, TextDirection::Ltr);

        let advances: Vec<f32> = shaped.glyphs.iter().map(|glyph| glyph.advance).collect();
        assert!(advances[1] > advances[0]);
    }
}
