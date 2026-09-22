use bevy::a11y::AccessibilityNode;
use bevy::{
    asset::Asset, prelude::*, reflect::TypePath, render::render_resource::AsBindGroup,
    shader::ShaderRef,
};

use crate::rendering::{InnerShadow, Paint, Surface};
use crate::primitives::a11y;
use crate::icons::{Icon, IconNode};
use crate::components::text::{TextRole, ThemedText};
use crate::theme::{AccessibilityVisualPolicyResource, ThemeResource};

// ============================================================
// TOGGLE
// ============================================================

#[derive(Component)]
pub struct Toggle {
    pub checked: bool,
    pub disabled: bool,
    pub press_consumed: bool,
}

// ============================================================
// VISUAL PARTS
// ============================================================

#[derive(Component)]
pub struct ToggleTrack;

#[derive(Component)]
pub struct ToggleThumb;

#[derive(Component)]
pub struct ToggleThumbShadow;

#[derive(Component)]
pub struct ToggleThumbIcon;

#[derive(Component)]
pub struct ToggleLabel;

#[derive(Component, Clone, Copy)]
struct ToggleTrackPart {
    owner: Entity,
}

#[derive(Component, Clone, Copy)]
struct ToggleThumbPart {
    owner: Entity,
}

#[derive(Component, Clone, Copy)]
struct ToggleThumbShadowPart {
    owner: Entity,
    alpha: f32,
}

#[derive(Component, Clone, Copy)]
struct ToggleThumbIconPart {
    owner: Entity,
    kind: ToggleThumbIconKind,
}

#[derive(Component, Clone, Copy)]
struct ToggleLabelPart {
    owner: Entity,
}

#[derive(Clone, Copy)]
enum ToggleThumbIconKind {
    Off,
    On,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ToggleShadowMaterial {
    #[uniform(0)]
    color: Vec4,
    #[uniform(0)]
    radius: f32,
    #[uniform(0)]
    softness: f32,
}

impl UiMaterial for ToggleShadowMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://beverly/components/toggle/toggle_shadow.wgsl".into()
    }
}

// ============================================================
// EVENTS
// ============================================================

#[derive(Message, Debug, Clone)]
pub enum ToggleEvent {
    Changed { entity: Entity, checked: bool },
}

// ============================================================
// CONFIGURATION
// ============================================================

#[derive(Clone)]
pub struct ToggleConfig {
    pub label: Option<String>,
    pub checked: bool,
    pub disabled: bool,
    pub thumb_icons: Option<ToggleThumbIcons>,
}

#[derive(Clone)]
pub struct ToggleThumbIcons {
    pub off: Icon,
    pub on: Icon,
}

impl Default for ToggleConfig {
    fn default() -> Self {
        Self {
            label: None,
            checked: false,
            disabled: false,
            thumb_icons: None,
        }
    }
}

impl ToggleConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn thumb_icons(mut self, off: Icon, on: Icon) -> Self {
        self.thumb_icons = Some(ToggleThumbIcons { off, on });
        self
    }
}

// ============================================================
// ANIMATION
// ============================================================

#[derive(Component)]
pub struct ToggleAnimation {
    pub progress: f32,
    pub target: f32,
    pub velocity: f32,
}

// ============================================================
// CONSTANTS
// ============================================================

const TRACK_WIDTH: f32 = 68.0;
const TRACK_HEIGHT: f32 = 38.0;

const THUMB_SIZE: f32 = 28.0;
const THUMB_ICON_SIZE: f32 = 12.0;

const THUMB_OFF_X: f32 = 5.0;
const THUMB_ON_X: f32 = TRACK_WIDTH - THUMB_SIZE - 5.0;

const THUMB_ICON_EPSILON: f32 = 0.01;

// Faster, snappier
const SPRING_STIFFNESS: f32 = 260.0;
const SPRING_DAMPING: f32 = 26.0;

// ============================================================
// SHADOW CONSTANTS
// ============================================================

const SHADOW_OFFSET_X: f32 = -3.0;
const SHADOW_OFFSET_Y: f32 = 4.0;
const SHADOW_WIDTH: f32 = 46.0;
const SHADOW_HEIGHT: f32 = 32.0;
const SHADOW_ALPHA: f32 = 0.22;

// ============================================================
// SPAWN
// ============================================================

pub fn spawn_toggle(
    parent: &mut ChildSpawnerCommands,
    config: ToggleConfig,
    theme: &ThemeResource,
    materials: &mut Assets<ToggleShadowMaterial>,
) -> Entity {
    let checked = config.checked;
    let default_colors = theme.current.colors;
    let accessible_label = config.label.clone().unwrap_or_else(|| "Toggle".to_string());

    let mut toggle_entity = parent.spawn((
        Name::new("toggle"),
        Toggle {
            checked,
            disabled: config.disabled,
            press_consumed: false,
        },
        Button,
        a11y::TabIndex(if config.disabled { -1 } else { 0 }),
        a11y::switch_node(accessible_label, checked),
        Node {
            width: percent(100),
            min_height: px(48.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: px(14.0),
            overflow: Overflow::visible(),
            ..default()
        },
        ToggleAnimation {
            progress: if checked { 1.0 } else { 0.0 },
            target: if checked { 1.0 } else { 0.0 },
            velocity: 0.0,
        },
    ));

    let toggle_id = toggle_entity.id();

    toggle_entity.with_children(|root| {
        if let Some(label) = config.label {
            root.spawn((
                ToggleLabel,
                ToggleLabelPart { owner: toggle_id },
                ThemedText::new(TextRole::Label),
                Text::new(label),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(label_color(default_colors, config.disabled)),
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
            ));
        }

        root.spawn((
            ToggleTrack,
            ToggleTrackPart { owner: toggle_id },
            Node {
                width: px(TRACK_WIDTH),
                height: px(TRACK_HEIGHT),
                position_type: PositionType::Relative,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(px(TRACK_HEIGHT * 0.5)),
                overflow: Overflow::visible(),
                ..default()
            },
            BackgroundColor(Color::NONE),
            Surface::rounded_rect_fill(
                TRACK_HEIGHT * 0.5,
                Paint::solid(track_color(
                    default_colors,
                    config.disabled,
                    Interaction::None,
                )),
            )
            .uniform_border(
                1.0,
                Paint::solid(track_inner_highlight_border(
                    config.disabled,
                    Interaction::None,
                )),
            )
            .inner_shadow(track_inner_shadow(config.disabled, Interaction::None)),
        ))
        .with_children(|track| {
            let initial_x = if checked { THUMB_ON_X } else { THUMB_OFF_X };
            let initial_top = (TRACK_HEIGHT - THUMB_SIZE) * 0.5;

            track
                .spawn((
                    ToggleThumb,
                    ToggleThumbPart { owner: toggle_id },
                    ZIndex(2),
                    Node {
                        width: px(THUMB_SIZE),
                        height: px(THUMB_SIZE),
                        position_type: PositionType::Absolute,
                        left: px(initial_x),
                        top: px(initial_top),
                        border_radius: BorderRadius::all(px(THUMB_SIZE * 0.5)),
                        overflow: Overflow::visible(),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    Surface::rounded_rect_fill(
                        THUMB_SIZE * 0.5,
                        Paint::solid(thumb_color(default_colors, config.disabled, Interaction::None)),
                    ),
                ))
                .with_children(|thumb| {
                    thumb.spawn((
                        ToggleThumbShadow,
                        ToggleThumbShadowPart {
                            owner: toggle_id,
                            alpha: SHADOW_ALPHA,
                        },
                        MaterialNode(materials.add(ToggleShadowMaterial {
                            color: Vec4::new(0.0, 0.0, 0.0, SHADOW_ALPHA),
                            radius: 0.42,
                            softness: 0.58,
                        })),
                        Node {
                            width: px(SHADOW_WIDTH),
                            height: px(SHADOW_HEIGHT),
                            position_type: PositionType::Absolute,
                            left: px(SHADOW_OFFSET_X),
                            top: px(SHADOW_OFFSET_Y),
                            border_radius: BorderRadius::all(px(9999.0)),
                            overflow: Overflow::visible(),
                            ..default()
                        },
                        ZIndex(0),
                    ));

                    thumb
                        .spawn((
                            Node {
                                width: px(THUMB_SIZE),
                                height: px(THUMB_SIZE),
                                position_type: PositionType::Absolute,
                                left: px(0.0),
                                top: px(0.0),
                                border_radius: BorderRadius::all(px(THUMB_SIZE * 0.5)),
                                overflow: Overflow::visible(),
                                ..default()
                            },
                            ZIndex(2),
                        ))
                        .with_children(|thumb_body| {
                            if let Some(thumb_icons) = config.thumb_icons.clone() {
                                thumb_body.spawn((
                                    ToggleThumbIcon,
                                    ToggleThumbIconPart {
                                        owner: toggle_id,
                                        kind: ToggleThumbIconKind::Off,
                                    },
                                    IconNode::new(thumb_icons.off).size(THUMB_ICON_SIZE).color(
                                        thumb_icon_color(
                                            default_colors,
                                            config.disabled,
                                            thumb_icon_progress(
                                                ToggleThumbIconKind::Off,
                                                if checked { 1.0 } else { 0.0 },
                                            ),
                                        ),
                                    ),
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: px((THUMB_SIZE - THUMB_ICON_SIZE) * 0.5),
                                        top: px((THUMB_SIZE - THUMB_ICON_SIZE) * 0.5),
                                        width: px(THUMB_ICON_SIZE),
                                        height: px(THUMB_ICON_SIZE),
                                        ..default()
                                    },
                                    Visibility::Visible,
                                ));

                                thumb_body.spawn((
                                    ToggleThumbIcon,
                                    ToggleThumbIconPart {
                                        owner: toggle_id,
                                        kind: ToggleThumbIconKind::On,
                                    },
                                    IconNode::new(thumb_icons.on).size(THUMB_ICON_SIZE).color(
                                        thumb_icon_color(
                                            default_colors,
                                            config.disabled,
                                            thumb_icon_progress(
                                                ToggleThumbIconKind::On,
                                                if checked { 1.0 } else { 0.0 },
                                            ),
                                        ),
                                    ),
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: px((THUMB_SIZE - THUMB_ICON_SIZE) * 0.5),
                                        top: px((THUMB_SIZE - THUMB_ICON_SIZE) * 0.5),
                                        width: px(THUMB_ICON_SIZE),
                                        height: px(THUMB_ICON_SIZE),
                                        ..default()
                                    },
                                    Visibility::Visible,
                                ));
                            }
                        });
                });
        });
    });

    toggle_id
}

// ============================================================
// INPUT
// ============================================================

fn toggle_interaction_system(
    time: Res<Time>,
    policy: Res<AccessibilityVisualPolicyResource>,
    mut query: Query<
        (Entity, &Interaction, &mut Toggle, &mut ToggleAnimation),
        (With<Button>, With<Toggle>),
    >,
    mut events: MessageWriter<ToggleEvent>,
) {
    let dt = time.delta_secs();

    for (entity, interaction, mut toggle, mut animation) in &mut query {
        let is_pressed = *interaction == Interaction::Pressed;

        if is_pressed && !toggle.disabled && !toggle.press_consumed {
            toggle.checked = !toggle.checked;
            animation.target = if toggle.checked { 1.0 } else { 0.0 };
            toggle.press_consumed = true;

            events.write(ToggleEvent::Changed {
                entity,
                checked: toggle.checked,
            });
        }

        if !is_pressed {
            toggle.press_consumed = false;
        }

        animation.target = if toggle.checked { 1.0 } else { 0.0 };
        if policy.current.reduced_motion {
            animation.progress = animation.target;
            animation.velocity = 0.0;
        } else {
            spring_step(&mut animation, dt);
        }
        animation.progress = animation.progress.clamp(0.0, 1.0);
    }
}

/// Keeps the AccessKit toggled/disabled state in sync for screen readers.
fn toggle_a11y_system(mut query: Query<(&Toggle, &mut AccessibilityNode), Changed<Toggle>>) {
    for (toggle, mut node) in &mut query {
        node.0.set_toggled(if toggle.checked {
            accesskit::Toggled::True
        } else {
            accesskit::Toggled::False
        });
        a11y::set_disabled(&mut node, toggle.disabled);
    }
}

// ============================================================
// SPRING ANIMATION
// ============================================================

fn spring_step(animation: &mut ToggleAnimation, dt: f32) {
    crate::animation::animation::spring::spring_step(
        &mut animation.progress,
        &mut animation.velocity,
        animation.target,
        dt,
        SPRING_STIFFNESS,
        SPRING_DAMPING,
    );
}

// ============================================================
// VISUAL STATE
// ============================================================

fn toggle_visual_system(
    theme: Res<ThemeResource>,
    toggle_query: Query<(&Toggle, &Interaction, &ToggleAnimation)>,
    mut surface_queries: ParamSet<(
        Query<
            (&ToggleTrackPart, &mut Surface),
            (
                With<ToggleTrack>,
                Without<ToggleThumb>,
                Without<ToggleLabel>,
            ),
        >,
        Query<
            (&ToggleThumbPart, &mut Node, &mut Surface),
            (
                With<ToggleThumb>,
                Without<ToggleTrack>,
                Without<ToggleLabel>,
            ),
        >,
    )>,
    mut materials: ResMut<Assets<ToggleShadowMaterial>>,
    mut thumb_shadow_query: Query<
        (&ToggleThumbShadowPart, &MaterialNode<ToggleShadowMaterial>),
        (
            With<ToggleThumbShadow>,
            Without<ToggleTrack>,
            Without<ToggleThumb>,
            Without<ToggleLabel>,
        ),
    >,
    mut thumb_icon_query: Query<
        (
            &ToggleThumbIconPart,
            &mut IconNode,
            &mut Node,
            &mut Visibility,
        ),
        (
            With<ToggleThumbIcon>,
            Without<ToggleTrack>,
            Without<ToggleThumb>,
            Without<ToggleLabel>,
        ),
    >,
    mut label_query: Query<
        (&ToggleLabelPart, &mut TextColor),
        (
            With<ToggleLabel>,
            Without<ToggleTrack>,
            Without<ToggleThumb>,
        ),
    >,
) {
    let palette = theme.current.colors;

    for (track, mut surface) in &mut surface_queries.p0() {
        let Ok((toggle, interaction, _)) = toggle_query.get(track.owner) else {
            continue;
        };

        surface.fill = Paint::solid(track_color(palette, toggle.disabled, *interaction));
        if let Some(border) = surface.border.as_mut() {
            border.paint = Paint::solid(track_inner_highlight_border(toggle.disabled, *interaction));
        }
        surface.effects.inner_shadow = Some(track_inner_shadow(toggle.disabled, *interaction));
    }

    for (thumb, mut node, mut surface) in &mut surface_queries.p1() {
        let Ok((toggle, interaction, animation)) = toggle_query.get(thumb.owner) else {
            continue;
        };

        let pressed = !toggle.disabled && *interaction == Interaction::Pressed;
        let progress = animation.progress.clamp(0.0, 1.0);

        let pressed_offset = if pressed {
            if toggle.checked { -1.2 } else { 1.2 }
        } else {
            0.0
        };

        let travel = THUMB_ON_X - THUMB_OFF_X;
        let left_pos = THUMB_OFF_X + travel * progress + pressed_offset;
        let top_pos = (TRACK_HEIGHT - THUMB_SIZE) * 0.5;

        node.width = px(THUMB_SIZE);
        node.height = px(THUMB_SIZE);
        node.left = px(left_pos);
        node.top = px(top_pos);
        node.border_radius = BorderRadius::all(px(THUMB_SIZE * 0.5));

        surface.fill = Paint::solid(thumb_color(palette, toggle.disabled, *interaction));
    }

    for (shadow, material_node) in &mut thumb_shadow_query {
        let Ok((toggle, _, _)) = toggle_query.get(shadow.owner) else {
            continue;
        };

        let alpha = shadow.alpha * if toggle.disabled { 0.55 } else { 1.0 };
        if let Some(mut material) = materials.get_mut(material_node.id()) {
            material.color = Vec4::new(0.0, 0.0, 0.0, alpha.clamp(0.0, 1.0));
        }
    }

    for (thumb_icon, mut icon, mut node, mut visibility) in &mut thumb_icon_query {
        let Ok((toggle, _, animation)) = toggle_query.get(thumb_icon.owner) else {
            continue;
        };

        let progress = animation.progress.clamp(0.0, 1.0);
        let icon_progress = thumb_icon_progress(thumb_icon.kind, progress);

        if icon_progress < THUMB_ICON_EPSILON {
            *visibility = Visibility::Hidden;
            continue;
        }

        *visibility = Visibility::Visible;

        let size = THUMB_ICON_SIZE * (0.86 + icon_progress * 0.18);
        let offset = (1.0 - icon_progress) * 1.5;
        icon.size = size;
        icon.color = thumb_icon_color(palette, toggle.disabled, icon_progress);

        node.width = px(size);
        node.height = px(size);
        node.left = px((THUMB_SIZE - size) * 0.5 + offset);
        node.top = px((THUMB_SIZE - size) * 0.5 - offset * 0.35);
    }

    for (label, mut color) in &mut label_query {
        let Ok((toggle, _, _)) = toggle_query.get(label.owner) else {
            continue;
        };

        color.0 = label_color(palette, toggle.disabled);
    }
}

fn track_color(
    colors: crate::theme::ThemeColors,
    disabled: bool,
    interaction: Interaction,
) -> Color {
    if disabled {
        match interaction {
            Interaction::Hovered => colors.border.with_alpha(0.75),
            Interaction::Pressed => colors.border_strong,
            Interaction::None => colors.border.with_alpha(0.75),
        }
    } else {
        match interaction {
            Interaction::Pressed => colors.border_strong,
            Interaction::Hovered | Interaction::None => colors.border,
        }
    }
}

fn track_inner_shadow(disabled: bool, interaction: Interaction) -> InnerShadow {
    let opacity = if disabled {
        0.55
    } else {
        match interaction {
            Interaction::Pressed => 0.88,
            Interaction::Hovered | Interaction::None => 0.78,
        }
    };

    let color = Color::srgb(0.76, 0.83, 0.89);

    InnerShadow::small(color).with_opacity(opacity)
}

fn track_inner_highlight_border(disabled: bool, interaction: Interaction) -> Color {
    let alpha = if disabled {
        0.24
    } else {
        match interaction {
            Interaction::Pressed => 0.36,
            Interaction::Hovered | Interaction::None => 0.42,
        }
    };

    Color::srgba(1.0, 1.0, 1.0, alpha)
}

fn thumb_color(
    colors: crate::theme::ThemeColors,
    disabled: bool,
    interaction: Interaction,
) -> Color {
    if disabled {
        colors.surface.with_alpha(0.82)
    } else {
        match interaction {
            Interaction::Hovered => blend_color(colors.surface_elevated, colors.background, 0.24),
            Interaction::Pressed => blend_color(colors.surface_elevated, colors.background, 0.38),
            Interaction::None => colors.surface_elevated,
        }
    }
}

fn blend_color(from: Color, to: Color, t: f32) -> Color {
    let from = from.to_linear();
    let to = to.to_linear();

    Color::linear_rgba(
        from.red + (to.red - from.red) * t,
        from.green + (to.green - from.green) * t,
        from.blue + (to.blue - from.blue) * t,
        from.alpha + (to.alpha - from.alpha) * t,
    )
}

fn thumb_icon_progress(kind: ToggleThumbIconKind, progress: f32) -> f32 {
    let p = progress.clamp(0.0, 1.0);
    match kind {
        ToggleThumbIconKind::Off => 1.0 - p,
        ToggleThumbIconKind::On => p,
    }
}

fn thumb_icon_color(colors: crate::theme::ThemeColors, disabled: bool, progress: f32) -> Color {
    let alpha = progress.clamp(0.0, 1.0);
    if disabled {
        colors.text_disabled.with_alpha(alpha)
    } else {
        colors.text.with_alpha(alpha)
    }
}

fn label_color(colors: crate::theme::ThemeColors, disabled: bool) -> Color {
    if disabled {
        colors.text_disabled
    } else {
        colors.text
    }
}

fn px(value: f32) -> Val {
    Val::Px(value)
}

fn percent(value: i32) -> Val {
    Val::Percent(value as f32)
}

pub struct TogglePlugin;

impl Plugin for TogglePlugin {
    fn build(&self, app: &mut App) {
        bevy::asset::embedded_asset!(app, "toggle_shadow.wgsl");
        app.add_plugins(UiMaterialPlugin::<ToggleShadowMaterial>::default())
            .init_resource::<AccessibilityVisualPolicyResource>()
            .add_message::<ToggleEvent>()
            .add_systems(Update, (toggle_interaction_system, toggle_a11y_system))
            .add_systems(PostUpdate, toggle_visual_system);
    }
}
