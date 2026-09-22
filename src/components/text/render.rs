use bevy::prelude::*;

use crate::rendering::Paint;

use super::atlas::{GlyphAtlasCache, GlyphAtlasKey, GlyphRasterizationMode};
use super::layout::TextLayoutBlock;
use super::typography::Typography;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphInstance {
    pub atlas_page: u32,
    pub position: Vec2,
    pub advance: f32,
    pub uv_min: Vec2,
    pub uv_max: Vec2,
    pub color: LinearRgba,
}

#[derive(Component, Clone, Debug, Default, PartialEq)]
pub struct TextRenderItem {
    pub instances: Vec<GlyphInstance>,
    pub paint: Option<Paint>,
    pub layout_bounds: Rect,
    pub visual_bounds: Rect,
}

fn ensure_text_render_item(world: &mut World) {
    let entities: Vec<Entity> = {
        let mut query = world.query_filtered::<Entity, (With<Typography>, Without<TextRenderItem>)>();
        query.iter(world).collect()
    };

    for entity in entities {
        if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
            entity_mut.insert(TextRenderItem::default());
        }
    }
}

fn build_render_items(
    atlas: Res<GlyphAtlasCache>,
    query: Query<(&Typography, &TextLayoutBlock, &mut TextRenderItem)>,
) {
    for (typography, layout, mut item) in query {
        let mut instances = Vec::new();

        for run in &layout.runs {
            for glyph in &run.glyphs {
                let key = GlyphAtlasKey {
                    family: run.font_family.clone(),
                    glyph_id: glyph.glyph_id,
                    font_size_bits: typography.font_size.to_bits(),
                    mode: GlyphRasterizationMode::Bitmap,
                };

                let Some(region) = atlas.map.get(&key) else {
                    continue;
                };

                instances.push(GlyphInstance {
                    atlas_page: region.page,
                    position: glyph.position + glyph.offset,
                    advance: glyph.advance,
                    uv_min: region.uv_min,
                    uv_max: region.uv_max,
                    color: typography.color.to_linear(),
                });
            }
        }

        let next = TextRenderItem {
            instances,
            paint: Some(Paint::solid(typography.color)),
            layout_bounds: layout.layout_bounds,
            visual_bounds: layout.visual_bounds,
        };

        if *item != next {
            *item = next;
        }
    }
}

pub struct TextRenderItemPlugin;

impl Plugin for TextRenderItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (ensure_text_render_item, build_render_items).chain());
    }
}
