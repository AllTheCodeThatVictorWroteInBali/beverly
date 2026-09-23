use bevy::picking::{
    events::{Drag, Pointer, Press},
    pointer::PointerButton,
};
use bevy::prelude::*;
use bevy::text::{ComputedTextBlock, FontCx};
use bevy::ui::{ui_surface::UiSurface, ComputedUiRenderTargetInfo, UiScale};
use parley::editing::Selection;

use super::{
    TextRenderItemPlugin,
    Typography,
    TypographyDebugPlugin,
    TypographyFontManagerPlugin,
    TypographyGlyphAtlasPlugin,
    TypographyLayoutPlugin,
    TypographyPlugin,
    TypographyShapingPlugin,
};
use crate::components::title::ThemedTitle;
use crate::theme::{ThemeColors, ThemeResource};

#[derive(Component, Clone, Debug, Default)]
#[require(TextSelectionOverlays)]
pub struct HighlightableText {
    selection: Option<Selection>,
}

#[derive(Component, Default)]
struct TextSelectionOverlays(Vec<Entity>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextRole {
    Heading,
    Body,
    Label,
    Caption,
    Muted,
    Accent,
    Disabled,
}

#[derive(Component, Clone, Copy, Debug)]
#[require(HighlightableText)]
pub struct ThemedText {
    pub role: TextRole,
    pub size_override: Option<f32>,
    pub color_override: Option<Color>,
}

impl ThemedText {
    pub fn new(role: TextRole) -> Self {
        Self {
            role,
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

fn themed_text_system(
    theme: Option<Res<ThemeResource>>,
    mut query: Query<(&ThemedText, &mut TextFont, &mut TextColor, Option<&Typography>)>,
) {
    let Some(theme) = theme else {
        return;
    };

    let colors = theme.current.colors;
    let typography = theme.current.typography;

    for (themed, mut font, mut color, custom_typography) in &mut query {
        // Text always carries a Typography component (required component); only defer to it
        // once the caller has explicitly opted in via a `with_*` builder (sync_to_bevy = true).
        if custom_typography.is_some_and(|t| t.sync_to_bevy) {
            continue;
        }

        let new_size = FontSize::Px(
            themed
                .size_override
                .unwrap_or(role_size(themed.role, typography)),
        );
        let new_color = themed
            .color_override
            .unwrap_or(role_color(themed.role, colors));

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

fn font_size_to_px(font_size: &FontSize) -> f32 {
    match font_size {
        FontSize::Px(px) => *px,
        _ => 16.0,
    }
}

fn bootstrap_typography_from_text_system(
    mut query: Query<
        (&mut Typography, Option<&TextFont>, Option<&TextColor>),
        (Added<Typography>, Without<ThemedText>, Without<ThemedTitle>),
    >,
) {
    for (mut typography, text_font, text_color) in &mut query {
        if typography.sync_to_bevy {
            continue;
        }

        if let Some(font) = text_font {
            typography.font_size = font_size_to_px(&font.font_size).max(1.0);
        }

        if let Some(color) = text_color {
            typography.color = color.0;
        }
    }
}

fn apply_typography_to_text_system(
    mut query: Query<
        (&Typography, &mut Text, &mut TextFont, &mut TextColor),
        Changed<Typography>,
    >,
) {
    for (typography, mut text, mut font, mut color) in &mut query {
        if !typography.sync_to_bevy {
            continue;
        }

        if let Some(content) = &typography.content {
            if text.0 != *content {
                *text = Text::new(content.clone());
            }
        }

        let next_size = FontSize::Px(typography.font_size.max(1.0));
        if font.font_size != next_size {
            font.font_size = next_size;
        }

        if color.0 != typography.color {
            color.0 = typography.color;
        }
    }
}

fn mirror_text_into_typography_system(
    mut query: Query<
        (&mut Typography, Option<&TextFont>, Option<&TextColor>, Option<&Text>),
        (
            Or<(Changed<TextFont>, Changed<TextColor>, Changed<Text>)>,
            Without<ThemedText>,
            Without<ThemedTitle>,
        ),
    >,
) {
    for (mut typography, text_font, text_color, text) in &mut query {
        if typography.sync_to_bevy {
            continue;
        }

        if let Some(font) = text_font {
            typography.font_size = font_size_to_px(&font.font_size).max(1.0);
        }

        if let Some(color) = text_color {
            typography.color = color.0;
        }

        if let Some(content) = text {
            typography.content = Some(content.0.clone());
        }
    }
}

fn text_local_position(
    pointer_position: Vec2,
    node: &ComputedNode,
    target: &ComputedUiRenderTargetInfo,
    transform: &UiGlobalTransform,
    ui_scale: &UiScale,
) -> Option<Vec2> {
    transform.try_inverse().map(|inverse| {
        inverse.transform_point2(pointer_position * target.scale_factor() / ui_scale.0)
            - node.content_box().min
    })
}

fn begin_text_selection(
    press: On<Pointer<Press>>,
    ui_scale: Res<UiScale>,
    mut query: Query<(
        Entity,
        &ComputedTextBlock,
        &ComputedNode,
        &ComputedUiRenderTargetInfo,
        &UiGlobalTransform,
        &mut HighlightableText,
    )>,
) {
    if press.button != PointerButton::Primary {
        return;
    }

    let target_entity = press.entity;
    for (entity, text, node, target, transform, mut highlightable) in &mut query {
        if entity != target_entity {
            if highlightable.selection.take().is_some() {
                highlightable.set_changed();
            }
            continue;
        }

        let Some(position) = text_local_position(
            press.pointer_location.position,
            node,
            target,
            transform,
            &ui_scale,
        ) else {
            continue;
        };

        let selection = if press.count == 2 {
            Selection::word_from_point(text.buffer(), position.x, position.y)
        } else {
            Selection::from_point(text.buffer(), position.x, position.y)
        };
        highlightable.selection = Some(selection);
    }
}

fn extend_text_selection(
    drag: On<Pointer<Drag>>,
    ui_scale: Res<UiScale>,
    mut query: Query<(
        &ComputedTextBlock,
        &ComputedNode,
        &ComputedUiRenderTargetInfo,
        &UiGlobalTransform,
        &mut HighlightableText,
    )>,
) {
    if drag.button != PointerButton::Primary {
        return;
    }

    let Ok((text, node, target, transform, mut highlightable)) = query.get_mut(drag.entity) else {
        return;
    };
    let Some(selection) = highlightable.selection else {
        return;
    };
    let Some(position) = text_local_position(
        drag.pointer_location.position,
        node,
        target,
        transform,
        &ui_scale,
    ) else {
        return;
    };

    highlightable.selection =
        Some(selection.extend_to_point(text.buffer(), position.x, position.y));
}

/// Bright, saturated highlighter yellow so the selection reads clearly on any theme/background.
const TEXT_SELECTION_COLOR: Color = Color::srgba(1.0, 0.85, 0.0, 0.55);

fn render_text_selection(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &ComputedTextBlock,
            &HighlightableText,
            &mut TextSelectionOverlays,
        ),
        Or<(Changed<HighlightableText>, Changed<ComputedTextBlock>)>,
    >,
) {
    for (entity, text, highlightable, mut overlays) in &mut query {
        for overlay in overlays.0.drain(..) {
            commands.entity(overlay).despawn();
        }

        let Some(selection) = highlightable.selection else {
            continue;
        };
        let rectangles = selection.geometry(text.buffer());
        let color = TEXT_SELECTION_COLOR;

        commands.entity(entity).with_children(|parent| {
            for (bounds, _) in rectangles {
                let overlay = parent
                    .spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(bounds.x0 as f32),
                            top: px(bounds.y0 as f32),
                            width: px(bounds.width() as f32),
                            height: px(bounds.height() as f32),
                            ..default()
                        },
                        BackgroundColor(color),
                        Pickable::IGNORE,
                        ZIndex(1),
                    ))
                    .id();
                overlays.0.push(overlay);
            }
        });
    }
}

fn role_size(role: TextRole, typography: crate::theme::ThemeTypography) -> f32 {
    match role {
        TextRole::Heading => typography.font_size_title,
        TextRole::Body => typography.font_size_body,
        TextRole::Label => typography.font_size_body - 2.0,
        TextRole::Caption => typography.font_size_caption,
        TextRole::Muted => typography.font_size_body,
        TextRole::Accent => typography.font_size_body - 2.0,
        TextRole::Disabled => typography.font_size_body,
    }
}

fn role_color(role: TextRole, colors: ThemeColors) -> Color {
    match role {
        TextRole::Heading | TextRole::Body | TextRole::Label => colors.text,
        TextRole::Caption | TextRole::Muted => colors.text_muted,
        TextRole::Accent => colors.primary,
        TextRole::Disabled => colors.text_disabled,
    }
}

pub struct ThemedTextPlugin;

impl Plugin for ThemedTextPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ThemeResource>()
            .init_resource::<Time>()
            .init_resource::<UiScale>()
            .init_resource::<UiSurface>()
            .init_resource::<FontCx>()
            .add_plugins((
                TypographyPlugin,
                TypographyFontManagerPlugin,
                TypographyShapingPlugin,
                TypographyLayoutPlugin,
                TypographyGlyphAtlasPlugin,
                TextRenderItemPlugin,
                TypographyDebugPlugin,
            ));

        // Every Text node gets selection support, not just ones wrapped in ThemedText/ThemedTitle.
        app.register_required_components::<Text, HighlightableText>();
        app.add_observer(begin_text_selection)
            .add_observer(extend_text_selection)
            .add_systems(
                Update,
                (
                    bootstrap_typography_from_text_system,
                    apply_typography_to_text_system,
                    themed_text_system,
                    mirror_text_into_typography_system,
                    render_text_selection,
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_entity_gets_highlightable_text() {
        let mut app = App::new();
        app.add_plugins((AssetPlugin::default(), ThemedTextPlugin));

        let entity = app.world_mut().spawn(Text::new("Videos")).id();

        assert!(app.world().get::<HighlightableText>(entity).is_some());
    }

    #[test]
    fn themed_text_entities_with_typography_do_not_conflict() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default(), ThemedTextPlugin));
        app.init_asset::<Font>();
        app.init_resource::<ThemeResource>();
        let entity = app
            .world_mut()
            .spawn((
                Text::new("Videos"),
                Typography::default(),
                ThemedText::new(TextRole::Body),
            ))
            .id();

        app.update();

        assert!(app.world().get::<TextFont>(entity).is_some());
    }
}
