use bevy::prelude::*;

use crate::components::text::{TextRole, ThemedText};
use crate::icons::{Icon, IconNode};
use crate::primitives::a11y;
use crate::primitives::semantic::{SemanticNode, SemanticRole};
use crate::rendering::prelude::Border;
use crate::rendering::{Paint, Surface};
use crate::theme::{ThemeColors, ThemeMode, ThemeResource, light_theme};

// ============================================================
// CHECKBOX
// ============================================================

#[derive(Component)]
pub struct Checkbox;

#[derive(Component, Clone, Copy, Debug)]
pub struct CheckboxState {
    pub checked: bool,
    pub disabled: bool,
    pub indeterminate: bool,
}

// ============================================================
// VISUAL PARTS
// ============================================================

#[derive(Component)]
pub struct CheckboxBox;

#[derive(Component)]
pub struct CheckboxCheck;

#[derive(Component)]
pub struct CheckboxLabel;

#[derive(Component, Clone, Copy)]
struct CheckboxBoxPart {
    owner: Entity,
}

#[derive(Component, Clone, Copy)]
struct CheckboxCheckPart {
    owner: Entity,
}

#[derive(Component, Clone, Copy)]
struct CheckboxLabelPart {
    owner: Entity,
}

// ============================================================
// EVENTS
// ============================================================

#[derive(Message, Debug, Clone)]
pub enum CheckboxEvent {
    Changed {
        entity: Entity,
        checked: bool,
        indeterminate: bool,
    },
}

// ============================================================
// CONFIGURATION
// ============================================================

#[derive(Clone)]
pub struct CheckboxConfig {
    pub label: Option<String>,
    pub checked: bool,
    pub disabled: bool,
    pub indeterminate: bool,
}

impl Default for CheckboxConfig {
    fn default() -> Self {
        Self {
            label: None,
            checked: false,
            disabled: false,
            indeterminate: false,
        }
    }
}

impl CheckboxConfig {
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

    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }
}

// ============================================================
// ANIMATION
// ============================================================

#[derive(Component)]
struct CheckboxAnimation {
    check_progress: f32,
    check_target: f32,
    check_velocity: f32,
    indeterminate_progress: f32,
    indeterminate_target: f32,
    indeterminate_velocity: f32,
}

const SPRING_STIFFNESS: f32 = 300.0;
const SPRING_DAMPING: f32 = 24.0;
const MARK_HIDDEN_EPSILON: f32 = 0.01;

// ============================================================
// SPAWN
// ============================================================

pub fn spawn_checkbox(parent: &mut ChildSpawnerCommands, config: CheckboxConfig) -> Entity {
    let checked = config.checked;
    let indeterminate = config.indeterminate;
    // Placeholder until the visual system applies the live theme on the first frame.
    let colors = light_theme().colors;
    let mode = ThemeMode::Light;
    let accessible_label = config
        .label
        .clone()
        .unwrap_or_else(|| "Checkbox".to_string());
    let mut semantic = SemanticNode::new(SemanticRole::Checkbox).label(accessible_label);
    semantic.state.checked = Some(checked);
    semantic.state.indeterminate = indeterminate;
    semantic.state.disabled = config.disabled;

    let mut checkbox_entity = parent.spawn((
        Checkbox,
        CheckboxState {
            checked,
            disabled: config.disabled,
            indeterminate,
        },
        Button,
        a11y::TabIndex(if config.disabled { -1 } else { 0 }),
        semantic,
        Node {
            width: percent(100),
            min_height: px(42.0),
            align_items: AlignItems::Center,
            column_gap: px(12.0),
            ..default()
        },
        CheckboxAnimation {
            check_progress: if checked && !indeterminate { 1.0 } else { 0.0 },
            check_target: if checked && !indeterminate { 1.0 } else { 0.0 },
            check_velocity: 0.0,
            indeterminate_progress: if indeterminate { 1.0 } else { 0.0 },
            indeterminate_target: if indeterminate { 1.0 } else { 0.0 },
            indeterminate_velocity: 0.0,
        },
    ));

    let checkbox_id = checkbox_entity.id();

    checkbox_entity.with_children(|root| {
        let initial_mark =
            mark_icon(checked, indeterminate).unwrap_or_else(|| Icon::feather("check"));

        root.spawn((
            CheckboxBox,
            CheckboxBoxPart { owner: checkbox_id },
            Node {
                width: px(26.0),
                height: px(26.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(px(7.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            Surface::rounded_rect_fill(
                7.0,
                Paint::solid(box_color(
                    mode,
                    checked,
                    indeterminate,
                    config.disabled,
                    Interaction::None,
                )),
            )
            .uniform_border(
                1.0,
                Paint::solid(border_color(
                    mode,
                    checked,
                    indeterminate,
                    config.disabled,
                    Interaction::None,
                )),
            ),
        ))
        .with_children(|box_node| {
            box_node.spawn((
                CheckboxCheck,
                CheckboxCheckPart { owner: checkbox_id },
                IconNode::new(initial_mark).size(20.0).color(mark_color(
                    mode,
                    checked,
                    indeterminate,
                    config.disabled,
                    1.0,
                    1.0,
                )),
                Node {
                    width: px(20.0),
                    height: px(20.0),
                    ..default()
                },
                Visibility::Visible,
            ));
        });

        if let Some(label) = config.label {
            root.spawn((
                CheckboxLabel,
                CheckboxLabelPart { owner: checkbox_id },
                ThemedText::new(TextRole::Label),
                Text::new(label),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(label_color(colors, config.disabled)),
            ));
        }
    });

    checkbox_id
}

// ============================================================
// INTERACTION
// ============================================================

fn checkbox_interaction_system(
    mut query: Query<
        (
            Entity,
            &Interaction,
            &mut CheckboxState,
            &mut CheckboxAnimation,
        ),
        (Changed<Interaction>, With<Button>, With<Checkbox>),
    >,
    mut events: MessageWriter<CheckboxEvent>,
) {
    for (entity, interaction, mut state, mut animation) in &mut query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if state.disabled {
            continue;
        }

        if state.indeterminate {
            state.indeterminate = false;
            state.checked = true;
        } else {
            state.checked = !state.checked;
        }

        sync_animation_targets(&state, &mut animation);

        events.write(CheckboxEvent::Changed {
            entity,
            checked: state.checked,
            indeterminate: state.indeterminate,
        });
    }
}

// ============================================================
// ANIMATION
// ============================================================

fn checkbox_animation_system(
    time: Res<Time>,
    mut query: Query<(&CheckboxState, &mut CheckboxAnimation), With<Checkbox>>,
) {
    let dt = time.delta_secs();

    for (state, mut animation) in &mut query {
        sync_animation_targets(state, &mut animation);

        let mut check_progress = animation.check_progress;
        let mut check_velocity = animation.check_velocity;
        let check_target = animation.check_target;
        spring_step(&mut check_progress, &mut check_velocity, check_target, dt);
        animation.check_progress = check_progress;
        animation.check_velocity = check_velocity;

        let mut indeterminate_progress = animation.indeterminate_progress;
        let mut indeterminate_velocity = animation.indeterminate_velocity;
        let indeterminate_target = animation.indeterminate_target;
        spring_step(
            &mut indeterminate_progress,
            &mut indeterminate_velocity,
            indeterminate_target,
            dt,
        );
        animation.indeterminate_progress = indeterminate_progress;
        animation.indeterminate_velocity = indeterminate_velocity;
    }
}

fn sync_animation_targets(state: &CheckboxState, animation: &mut CheckboxAnimation) {
    animation.check_target = if state.checked && !state.indeterminate {
        1.0
    } else {
        0.0
    };
    animation.indeterminate_target = if state.indeterminate { 1.0 } else { 0.0 };
}

fn spring_step(progress: &mut f32, velocity: &mut f32, target: f32, dt: f32) {
    if dt <= 0.0 {
        return;
    }

    let displacement = target - *progress;
    *velocity += displacement * SPRING_STIFFNESS * dt;

    let damping_factor = (1.0 - SPRING_DAMPING * dt).clamp(0.0, 1.0);
    *velocity *= damping_factor;

    *progress += *velocity * dt;
    *progress = progress.clamp(0.0, 1.0);

    let near_target = (target - *progress).abs() < 0.001;
    let near_still = velocity.abs() < 0.001;
    if near_target && near_still {
        *progress = target;
        *velocity = 0.0;
    }
}

// ============================================================
// VISUAL STATE
// ============================================================

fn checkbox_visual_system(
    theme: Res<ThemeResource>,
    root_query: Query<(&Interaction, &CheckboxState, &CheckboxAnimation), With<Checkbox>>,
    mut box_query: Query<
        (&CheckboxBoxPart, &mut Surface),
        (
            With<CheckboxBox>,
            Without<CheckboxCheck>,
            Without<CheckboxLabel>,
        ),
    >,
    mut check_query: Query<
        (
            &CheckboxCheckPart,
            &mut IconNode,
            &mut Node,
            &mut Visibility,
        ),
        (
            With<CheckboxCheck>,
            Without<CheckboxBox>,
            Without<CheckboxLabel>,
        ),
    >,
    mut label_query: Query<
        (&CheckboxLabelPart, &mut TextColor),
        (
            With<CheckboxLabel>,
            Without<CheckboxBox>,
            Without<CheckboxCheck>,
        ),
    >,
) {
    let colors = theme.current.colors;
    let mode = theme.current.mode;

    for (box_part, mut surface) in &mut box_query {
        let Ok((interaction, state, _)) = root_query.get(box_part.owner) else {
            continue;
        };

        surface.fill = Paint::solid(box_color(
            mode,
            state.checked,
            state.indeterminate,
            state.disabled,
            *interaction,
        ));
        surface.border = Some(Border::new(
            1.0,
            Paint::solid(border_color(
                mode,
                state.checked,
                state.indeterminate,
                state.disabled,
                *interaction,
            )),
        ));
    }

    for (check_part, mut icon, mut node, mut visibility) in &mut check_query {
        let Ok((_, state, animation)) = root_query.get(check_part.owner) else {
            continue;
        };

        let checked_progress = animation.check_progress;
        let indeterminate_progress = animation.indeterminate_progress;
        let mark_progress = checked_progress.max(indeterminate_progress);

        if let Some(mark) = mark_icon(state.checked, state.indeterminate) {
            icon.icon = mark;
        }

        if mark_progress < MARK_HIDDEN_EPSILON {
            *visibility = Visibility::Hidden;
            continue;
        }

        *visibility = Visibility::Visible;

        let scale = if state.indeterminate {
            0.92 + indeterminate_progress * 0.12
        } else {
            0.84 + checked_progress * 0.20
        };
        let size = 20.0 * scale;
        icon.size = size;
        node.width = px(size);
        node.height = px(size);

        icon.color = mark_color(
            mode,
            state.checked,
            state.indeterminate,
            state.disabled,
            checked_progress,
            indeterminate_progress,
        );
    }

    for (label_part, mut color) in &mut label_query {
        let Ok((_, state, _)) = root_query.get(label_part.owner) else {
            continue;
        };

        color.0 = label_color(colors, state.disabled);
    }
}

/// Same neutral palette as the default alert, button and avatar.
fn box_color(
    mode: ThemeMode,
    checked: bool,
    indeterminate: bool,
    disabled: bool,
    interaction: Interaction,
) -> Color {
    let on = checked || indeterminate;
    let light = mode == ThemeMode::Light;
    let gray = |v: u8| Color::srgb_u8(v, v, v);

    match (on, disabled, interaction) {
        // Checked: ink-colored box (black in light mode, white in dark mode).
        (true, true, _) => gray(if light { 163 } else { 82 }),
        (true, false, Interaction::None) => gray(if light { 23 } else { 255 }),
        (true, false, Interaction::Hovered) => gray(if light { 38 } else { 229 }),
        (true, false, Interaction::Pressed) => gray(if light { 64 } else { 212 }),
        // Unchecked: paper-colored box (white in light mode, #171717 in dark mode).
        (false, true, _) => gray(if light { 245 } else { 38 }),
        (false, false, Interaction::None) => gray(if light { 255 } else { 23 }),
        (false, false, Interaction::Hovered) => gray(if light { 245 } else { 38 }),
        (false, false, Interaction::Pressed) => gray(if light { 229 } else { 51 }),
    }
}

fn border_color(
    mode: ThemeMode,
    checked: bool,
    indeterminate: bool,
    disabled: bool,
    interaction: Interaction,
) -> Color {
    if checked || indeterminate {
        // Checked boxes blend their border into the fill.
        return box_color(mode, checked, indeterminate, disabled, interaction);
    }

    match mode {
        ThemeMode::Light => Color::srgb_u8(212, 212, 212),
        ThemeMode::Dark => Color::srgb_u8(38, 38, 38),
    }
}

fn mark_icon(checked: bool, indeterminate: bool) -> Option<Icon> {
    if indeterminate {
        Some(Icon::feather("minus"))
    } else if checked {
        Some(Icon::feather("check"))
    } else {
        None
    }
}

fn mark_color(
    mode: ThemeMode,
    checked: bool,
    indeterminate: bool,
    disabled: bool,
    checked_progress: f32,
    indeterminate_progress: f32,
) -> Color {
    let alpha = if indeterminate {
        indeterminate_progress.clamp(0.0, 1.0)
    } else if checked {
        checked_progress.clamp(0.0, 1.0)
    } else {
        0.0
    };

    // The mark is the inverse of the checked fill.
    let ink = match mode {
        ThemeMode::Light => Color::WHITE,
        ThemeMode::Dark => Color::srgb_u8(23, 23, 23),
    };
    ink.with_alpha(if disabled { alpha * 0.90 } else { alpha })
}

fn label_color(colors: ThemeColors, disabled: bool) -> Color {
    if disabled {
        colors.text_disabled
    } else {
        colors.text
    }
}

// ============================================================
// PLUGIN
// ============================================================

pub struct CheckboxPlugin;

impl Plugin for CheckboxPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<crate::primitives::semantic::SemanticAccessibilityPlugin>() {
            app.add_plugins(crate::primitives::semantic::SemanticAccessibilityPlugin);
        }
        app.add_message::<CheckboxEvent>().add_systems(
            Update,
            (
                checkbox_interaction_system,
                checkbox_animation_system,
                checkbox_visual_system,
            )
                .chain(),
        );
    }
}
