use bevy::prelude::*;

use super::atlas::GlyphAtlasCache;
use super::layout::{TextLayoutBlock, TextLayoutCache};
use super::shaping::TextShapingCache;
use super::typography::Typography;

#[derive(Resource, Clone, Debug)]
pub struct TypographyDebugSettings {
    pub enabled: bool,
    pub print_every_seconds: f32,
    pub show_overlay: bool,
}

impl Default for TypographyDebugSettings {
    fn default() -> Self {
        let enabled = matches!(
            std::env::var("UI_TYPOGRAPHY_DEBUG").ok().as_deref(),
            Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("on")
        );
        Self {
            enabled,
            print_every_seconds: 2.0,
            show_overlay: enabled,
        }
    }
}

#[derive(Resource, Default)]
struct TypographyDebugTicker {
    timer: Option<Timer>,
}

#[derive(Component, Default)]
struct TypographyDebugOverlayEntities(Vec<Entity>);

fn initialize_ticker(
    settings: Res<TypographyDebugSettings>,
    mut ticker: ResMut<TypographyDebugTicker>,
) {
    if ticker.timer.is_none() {
        ticker.timer = Some(Timer::from_seconds(
            settings.print_every_seconds.max(0.25),
            TimerMode::Repeating,
        ));
    }
}

fn report_typography_stats(
    time: Res<Time>,
    settings: Res<TypographyDebugSettings>,
    mut ticker: ResMut<TypographyDebugTicker>,
    shaping: Res<TextShapingCache>,
    layout: Res<TextLayoutCache>,
    atlas: Res<GlyphAtlasCache>,
) {
    if !settings.enabled {
        return;
    }

    let Some(timer) = ticker.timer.as_mut() else {
        return;
    };

    if !timer.tick(time.delta()).just_finished() {
        return;
    }

    info!(
        "typography stats | shape cache: {} (hits {}, misses {}) | layout cache: {} (hits {}, misses {}) | atlas: glyphs {} pages {} (hits {}, misses {}, evictions {})",
        shaping.runs.len(),
        shaping.hits,
        shaping.misses,
        layout.layouts.len(),
        layout.hits,
        layout.misses,
        atlas.map.len(),
        atlas.page_count,
        atlas.cache_hits,
        atlas.cache_misses,
        atlas.evictions,
    );
}

fn ensure_overlay_component(world: &mut World) {
    let entities: Vec<Entity> = {
        let mut query = world
            .query_filtered::<Entity, (With<Typography>, Without<TypographyDebugOverlayEntities>)>();
        query.iter(world).collect()
    };

    for entity in entities {
        if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
            entity_mut.insert(TypographyDebugOverlayEntities::default());
        }
    }
}

fn clear_overlay_entities(commands: &mut Commands, overlays: &mut TypographyDebugOverlayEntities) {
    for entity in overlays.0.drain(..) {
        commands.entity(entity).despawn();
    }
}

fn spawn_overlay_for_text(
    commands: &mut Commands,
    owner: Entity,
    layout: &TextLayoutBlock,
    overlays: &mut TypographyDebugOverlayEntities,
) {
    commands.entity(owner).with_children(|parent| {
        for line in &layout.lines {
            let line_height = (line.ascent + line.descent).max(1.0);
            let top = line.baseline - line.ascent;

            let line_rect = parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0.0),
                        top: px(top),
                        width: px(line.line_width.max(1.0)),
                        height: px(line_height),
                        border: UiRect::all(px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.5, 1.0, 0.06)),
                    BorderColor::all(Color::srgba(0.1, 0.7, 1.0, 0.45)),
                    Pickable::IGNORE,
                    ZIndex(20),
                ))
                .id();
            overlays.0.push(line_rect);

            let baseline = parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0.0),
                        top: px(line.baseline - 0.5),
                        width: px(line.line_width.max(1.0)),
                        height: px(1.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.95, 0.85, 0.05, 0.95)),
                    Pickable::IGNORE,
                    ZIndex(21),
                ))
                .id();
            overlays.0.push(baseline);

            let ascent = parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0.0),
                        top: px(line.baseline - line.ascent),
                        width: px(line.line_width.max(1.0)),
                        height: px(1.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.9, 0.8, 0.85)),
                    Pickable::IGNORE,
                    ZIndex(21),
                ))
                .id();
            overlays.0.push(ascent);

            let descent = parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0.0),
                        top: px(line.baseline + line.descent),
                        width: px(line.line_width.max(1.0)),
                        height: px(1.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(1.0, 0.3, 0.45, 0.85)),
                    Pickable::IGNORE,
                    ZIndex(21),
                ))
                .id();
            overlays.0.push(descent);
        }

        for run in &layout.runs {
            for glyph in &run.glyphs {
                let glyph_box = parent
                    .spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(glyph.position.x),
                            top: px(glyph.position.y - layout.metrics.ascent),
                            width: px(glyph.advance.max(1.0)),
                            height: px((layout.metrics.ascent + layout.metrics.descent).max(1.0)),
                            border: UiRect::all(px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.7, 0.2, 1.0, 0.04)),
                        BorderColor::all(Color::srgba(0.8, 0.4, 1.0, 0.35)),
                        Pickable::IGNORE,
                        ZIndex(22),
                    ))
                    .id();
                overlays.0.push(glyph_box);
            }
        }
    });
}

fn render_typography_overlay(
    mut commands: Commands,
    settings: Res<TypographyDebugSettings>,
    mut query: Query<(Entity, &TextLayoutBlock, &mut TypographyDebugOverlayEntities), Or<(Changed<TextLayoutBlock>, Changed<TypographyDebugSettings>)>>,
) {
    for (entity, layout, mut overlays) in &mut query {
        clear_overlay_entities(&mut commands, &mut overlays);

        if !settings.show_overlay {
            continue;
        }

        spawn_overlay_for_text(&mut commands, entity, layout, &mut overlays);
    }
}

pub struct TypographyDebugPlugin;

impl Plugin for TypographyDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TypographyDebugSettings>()
            .init_resource::<TypographyDebugTicker>()
            .add_systems(
                Update,
                (
                    initialize_ticker,
                    ensure_overlay_component,
                    render_typography_overlay,
                    report_typography_stats,
                ),
            );
    }
}
