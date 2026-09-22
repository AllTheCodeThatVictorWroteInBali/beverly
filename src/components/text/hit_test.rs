use bevy::prelude::*;

use super::layout::TextLayoutBlock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextHit {
    pub line_index: usize,
    pub cluster_index: usize,
}

pub fn hit_test(layout: &TextLayoutBlock, local_position: Vec2) -> Option<TextHit> {
    let mut best: Option<TextHit> = None;
    let mut best_distance = f32::MAX;

    for (line_index, line) in layout.lines.iter().enumerate() {
        let y_min = line.baseline - line.ascent;
        let y_max = line.baseline + line.descent;
        if local_position.y < y_min || local_position.y > y_max {
            continue;
        }

        let mut glyph_index = 0usize;
        for run in &layout.runs {
            for glyph in &run.glyphs {
                let x_min = glyph.position.x;
                let x_max = glyph.position.x + glyph.advance;
                let distance = if local_position.x < x_min {
                    x_min - local_position.x
                } else if local_position.x > x_max {
                    local_position.x - x_max
                } else {
                    0.0
                };

                if distance < best_distance {
                    best_distance = distance;
                    best = Some(TextHit {
                        line_index,
                        cluster_index: glyph.cluster_index,
                    });
                }
                glyph_index = glyph_index.saturating_add(1);
            }
        }

        if glyph_index == 0 {
            best = Some(TextHit {
                line_index,
                cluster_index: 0,
            });
        }
    }

    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::text::layout::{GlyphPlacement, GlyphRun, TextLine, TextMetrics};

    #[test]
    fn hit_test_picks_nearest_cluster() {
        let layout = TextLayoutBlock {
            metrics: TextMetrics {
                ascent: 8.0,
                descent: 2.0,
                line_gap: 0.0,
                line_height: 10.0,
                cap_height: 6.0,
                x_height: 4.0,
            },
            runs: vec![GlyphRun {
                font_family: "SFNS".to_string(),
                direction: crate::components::text::layout::TextDirection::Ltr,
                script: Some("latin".to_string()),
                language: Some("en".to_string()),
                glyphs: vec![
                    GlyphPlacement {
                        glyph_id: 72,
                        advance: 10.0,
                        offset: Vec2::ZERO,
                        position: Vec2::new(0.0, 8.0),
                        cluster_index: 0,
                    },
                    GlyphPlacement {
                        glyph_id: 105,
                        advance: 8.0,
                        offset: Vec2::ZERO,
                        position: Vec2::new(10.0, 8.0),
                        cluster_index: 1,
                    },
                ],
            }],
            lines: vec![TextLine {
                baseline: 8.0,
                ascent: 8.0,
                descent: 2.0,
                line_width: 18.0,
                start_glyph: 0,
                end_glyph: 2,
            }],
            layout_bounds: Rect::from_corners(Vec2::ZERO, Vec2::new(18.0, 10.0)),
            visual_bounds: Rect::from_corners(Vec2::ZERO, Vec2::new(18.0, 10.0)),
        };

        let hit = hit_test(&layout, Vec2::new(11.0, 8.0));
        assert_eq!(hit, Some(TextHit { line_index: 0, cluster_index: 1 }));
    }
}
