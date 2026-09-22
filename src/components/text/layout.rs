use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use bevy::prelude::*;

use super::shaping::{ShapedText, TextShapingResult};
use super::typography::{LineHeight, Typography};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum TextDirection {
    Ltr,
    Rtl,
    #[default]
    Auto,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Justify,
    #[default]
    Start,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum TextWrapping {
    NoWrap,
    #[default]
    WordWrap,
    CharacterWrap,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextMetrics {
    pub ascent: f32,
    pub descent: f32,
    pub line_gap: f32,
    pub line_height: f32,
    pub cap_height: f32,
    pub x_height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphPlacement {
    pub glyph_id: u32,
    pub advance: f32,
    pub offset: Vec2,
    pub position: Vec2,
    pub cluster_index: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GlyphRun {
    pub font_family: String,
    pub direction: TextDirection,
    pub script: Option<String>,
    pub language: Option<String>,
    pub glyphs: Vec<GlyphPlacement>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextLine {
    pub baseline: f32,
    pub ascent: f32,
    pub descent: f32,
    pub line_width: f32,
    pub start_glyph: usize,
    pub end_glyph: usize,
}

#[derive(Component, Clone, Debug, PartialEq)]
pub struct TextLayoutBlock {
    pub metrics: TextMetrics,
    pub runs: Vec<GlyphRun>,
    pub lines: Vec<TextLine>,
    pub layout_bounds: Rect,
    pub visual_bounds: Rect,
}

impl Default for TextLayoutBlock {
    fn default() -> Self {
        Self {
            metrics: TextMetrics::default(),
            runs: Vec::new(),
            lines: Vec::new(),
            layout_bounds: Rect::from_corners(Vec2::ZERO, Vec2::ZERO),
            visual_bounds: Rect::from_corners(Vec2::ZERO, Vec2::ZERO),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TextLayoutKey {
    pub content_hash: u64,
    pub width_bits: u32,
    pub font_size_bits: u32,
    pub line_height_bits: u32,
    pub wrapping: TextWrapping,
    pub alignment: TextAlignment,
    pub direction: TextDirection,
}

#[derive(Resource, Default)]
pub struct TextLayoutCache {
    pub layouts: HashMap<TextLayoutKey, TextLayoutBlock>,
    pub hits: u64,
    pub misses: u64,
}

fn glyph_text_hash(runs: &[GlyphRun]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for run in runs {
        run.font_family.hash(&mut hasher);
        run.direction.hash(&mut hasher);
        for glyph in &run.glyphs {
            glyph.glyph_id.hash(&mut hasher);
            glyph.cluster_index.hash(&mut hasher);
            glyph.advance.to_bits().hash(&mut hasher);
        }
    }
    hasher.finish()
}

fn layout_key(
    block_width: f32,
    typography: &Typography,
    line_height: LineHeight,
    runs: &[GlyphRun],
) -> TextLayoutKey {
    TextLayoutKey {
        content_hash: glyph_text_hash(runs),
        width_bits: block_width.to_bits(),
        font_size_bits: typography.font_size.to_bits(),
        line_height_bits: line_height.resolve(typography.font_size).to_bits(),
        wrapping: typography.wrapping,
        alignment: typography.alignment,
        direction: typography.direction,
    }
}

fn derive_metrics(typography: &Typography) -> TextMetrics {
    let line_height = typography.line_height.resolve(typography.font_size);
    let ascent = typography.font_size * 0.8;
    let descent = typography.font_size * 0.2;
    let line_gap = (line_height - (ascent + descent)).max(0.0);

    TextMetrics {
        ascent,
        descent,
        line_gap,
        line_height,
        cap_height: ascent * 0.88,
        x_height: ascent * 0.58,
    }
}

fn to_runs(shaped: &ShapedText, family: String) -> Vec<GlyphRun> {
    let mut x = 0.0;
    let glyphs = shaped
        .glyphs
        .iter()
        .map(|glyph| {
            let placement = GlyphPlacement {
                glyph_id: glyph.glyph_id,
                advance: glyph.advance,
                offset: glyph.offset,
                position: Vec2::new(x, 0.0),
                cluster_index: glyph.cluster.cluster_index,
            };
            x += glyph.advance;
            placement
        })
        .collect();

    vec![GlyphRun {
        font_family: family,
        direction: shaped.direction,
        script: shaped.script.clone(),
        language: shaped.language.clone(),
        glyphs,
    }]
}

fn apply_alignment(line_width: f32, block_width: f32, alignment: TextAlignment, rtl: bool) -> f32 {
    match alignment {
        TextAlignment::Left => 0.0,
        TextAlignment::Center => ((block_width - line_width) * 0.5).max(0.0),
        TextAlignment::Right => (block_width - line_width).max(0.0),
        TextAlignment::Justify => 0.0,
        TextAlignment::Start => {
            if rtl {
                (block_width - line_width).max(0.0)
            } else {
                0.0
            }
        }
        TextAlignment::End => {
            if rtl {
                0.0
            } else {
                (block_width - line_width).max(0.0)
            }
        }
    }
}

fn break_lines(
    runs: &mut [GlyphRun],
    metrics: TextMetrics,
    block_width: f32,
    wrapping: TextWrapping,
    alignment: TextAlignment,
    direction: TextDirection,
) -> Vec<TextLine> {
    let mut lines = Vec::new();

    let mut cursor_y = metrics.ascent;
    let mut global_index = 0usize;

    let rtl = matches!(direction, TextDirection::Rtl);

    for run in runs {
        let mut line_start = 0usize;
        let mut line_width = 0.0;
        let glyph_count = run.glyphs.len();

        for index in 0..glyph_count {
            let next_width = line_width + run.glyphs[index].advance;
            let needs_break = match wrapping {
                TextWrapping::NoWrap => false,
                TextWrapping::WordWrap | TextWrapping::CharacterWrap => {
                    block_width.is_finite() && block_width > 0.0 && next_width > block_width
                }
            };

            if needs_break && index > line_start {
                let offset_x = apply_alignment(line_width, block_width, alignment, rtl);
                for glyph in &mut run.glyphs[line_start..index] {
                    glyph.position.x += offset_x;
                    glyph.position.y = cursor_y;
                }

                lines.push(TextLine {
                    baseline: cursor_y,
                    ascent: metrics.ascent,
                    descent: metrics.descent,
                    line_width,
                    start_glyph: global_index + line_start,
                    end_glyph: global_index + index,
                });

                cursor_y += metrics.line_height;
                line_start = index;
                line_width = 0.0;
            }

            run.glyphs[index].position.x = line_width;
            run.glyphs[index].position.y = cursor_y;
            line_width += run.glyphs[index].advance;
        }

        let offset_x = apply_alignment(line_width, block_width, alignment, rtl);
        for glyph in &mut run.glyphs[line_start..] {
            glyph.position.x += offset_x;
            glyph.position.y = cursor_y;
        }

        lines.push(TextLine {
            baseline: cursor_y,
            ascent: metrics.ascent,
            descent: metrics.descent,
            line_width,
            start_glyph: global_index + line_start,
            end_glyph: global_index + run.glyphs.len(),
        });

        global_index += run.glyphs.len();
        cursor_y += metrics.line_height;
    }

    lines
}

fn measure_bounds(runs: &[GlyphRun], lines: &[TextLine], metrics: TextMetrics) -> (Rect, Rect) {
    let width = lines
        .iter()
        .map(|line| line.line_width)
        .fold(0.0_f32, f32::max);

    let height = if lines.is_empty() {
        0.0
    } else {
        lines.len() as f32 * metrics.line_height
    };

    let layout = Rect::from_corners(Vec2::ZERO, Vec2::new(width.max(0.0), height.max(0.0)));

    let mut min = Vec2::new(f32::MAX, f32::MAX);
    let mut max = Vec2::new(f32::MIN, f32::MIN);

    for run in runs {
        for glyph in &run.glyphs {
            let glyph_min = Vec2::new(glyph.position.x, glyph.position.y - metrics.ascent);
            let glyph_max = Vec2::new(
                glyph.position.x + glyph.advance,
                glyph.position.y + metrics.descent,
            );
            min = min.min(glyph_min);
            max = max.max(glyph_max);
        }
    }

    let visual = if min.x == f32::MAX {
        Rect::from_corners(Vec2::ZERO, Vec2::ZERO)
    } else {
        Rect::from_corners(min, max)
    };

    (layout, visual)
}

fn ensure_layout_components(world: &mut World) {
    let entities: Vec<Entity> = {
        let mut query = world
            .query_filtered::<Entity, (With<Text>, With<Typography>, Without<TextLayoutBlock>)>();
        query.iter(world).collect()
    };

    for entity in entities {
        if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
            entity_mut.insert(TextLayoutBlock::default());
        }
    }
}

fn layout_text_system(
    mut cache: ResMut<TextLayoutCache>,
    query: Query<(
        &Typography,
        &TextShapingResult,
        Option<&ComputedNode>,
        Option<&Node>,
        &mut TextLayoutBlock,
    )>,
) {
    for (typography, shaped, computed_node, node, mut layout_block) in query {
        let Some(shaped) = shaped.0.as_ref() else {
            continue;
        };

        let block_width = if let Some(computed) = computed_node {
            computed.size().x * computed.inverse_scale_factor()
        } else if let Some(node) = node {
            match node.width {
                Val::Px(px) => px,
                _ => f32::INFINITY,
            }
        } else {
            f32::INFINITY
        };

        let mut runs = to_runs(shaped, typography.family.0.clone());
        let key = layout_key(block_width, typography, typography.line_height, &runs);

        if let Some(cached) = cache.layouts.get(&key).cloned() {
            cache.hits += 1;
            if *layout_block != cached {
                *layout_block = cached;
            }
            continue;
        }

        cache.misses += 1;
        let metrics = derive_metrics(typography);
        let lines = break_lines(
            &mut runs,
            metrics,
            block_width,
            typography.wrapping,
            typography.alignment,
            typography.direction,
        );
        let (layout_bounds, visual_bounds) = measure_bounds(&runs, &lines, metrics);

        let next = TextLayoutBlock {
            metrics,
            runs,
            lines,
            layout_bounds,
            visual_bounds,
        };

        cache.layouts.insert(key, next.clone());

        if *layout_block != next {
            *layout_block = next;
        }
    }
}

pub struct TypographyLayoutPlugin;

impl Plugin for TypographyLayoutPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TextLayoutCache>().add_systems(
            Update,
            (ensure_layout_components, layout_text_system).chain(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::text::shaping::{GlyphCluster, ShapedGlyph, ShapedText};
    use crate::components::text::typography::{FontFamily, LineHeight, Typography};
    use crate::components::text::{FontStyle, FontWeight};

    fn base_typography() -> Typography {
        Typography {
            content: None,
            family: FontFamily::new("SFNS"),
            font_size: 16.0,
            weight: FontWeight::NORMAL,
            style: FontStyle::Normal,
            color: Color::WHITE,
            alignment: TextAlignment::Start,
            line_height: LineHeight::Multiplier(1.5),
            letter_spacing: 0.0,
            word_spacing: 0.0,
            wrapping: TextWrapping::WordWrap,
            direction: TextDirection::Ltr,
            language: Some("en".to_string()),
            sync_to_bevy: false,
        }
    }

    fn shaped_word(word: &str, advance: f32) -> ShapedText {
        let glyphs = word
            .char_indices()
            .enumerate()
            .map(|(cluster_index, (byte_start, ch))| ShapedGlyph {
                glyph_id: ch as u32,
                advance,
                offset: Vec2::ZERO,
                cluster: GlyphCluster {
                    cluster_index,
                    byte_start,
                    byte_end: byte_start + ch.len_utf8(),
                },
            })
            .collect();

        ShapedText {
            direction: TextDirection::Ltr,
            script: Some("latin".to_string()),
            language: Some("en".to_string()),
            glyphs,
        }
    }

    #[test]
    fn wraps_when_width_constraint_is_smaller_than_run() {
        let typography = base_typography();
        let shaped = shaped_word("HELLO", 10.0);
        let mut runs = to_runs(&shaped, typography.family.0.clone());
        let metrics = derive_metrics(&typography);

        let lines = break_lines(
            &mut runs,
            metrics,
            25.0,
            TextWrapping::CharacterWrap,
            TextAlignment::Start,
            TextDirection::Ltr,
        );

        assert!(lines.len() >= 2);
    }

    #[test]
    fn logical_alignment_uses_direction() {
        let offset_ltr = apply_alignment(80.0, 120.0, TextAlignment::Start, false);
        let offset_rtl = apply_alignment(80.0, 120.0, TextAlignment::Start, true);

        assert_eq!(offset_ltr, 0.0);
        assert!(offset_rtl > 0.0);
    }
}
