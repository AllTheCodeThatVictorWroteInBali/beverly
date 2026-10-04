use bevy::{color::Mix, prelude::*};

use crate::icons::{Icon, IconCommands, IconNode};
use crate::primitives::a11y;
use crate::primitives::composition::UiElement;
use crate::primitives::interaction::DisabledInteraction;
use crate::primitives::interaction::{InteractionEventType, UiPointerEvent};
use crate::primitives::semantic::{SemanticNode, SemanticRole};
use crate::rendering::{Paint, Surface};
use crate::theme::{ThemeColors, ThemeResource};

/// Application callback invoked when a button is activated.
pub type ButtonCommand = fn(&mut Commands, Entity);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonEventType {
    Click,
    ClickDown,
    ClickUp,
    PointerEnter,
    PointerLeave,
    PointerMove,
    DoubleClick,
    DragStart,
    DragMove,
    DragEnd,
    DragCancel,
    LongPress,
    Scroll,
    Load,
    Unload,
}

impl ButtonEventType {
    pub fn parse(event: &str) -> Option<Self> {
        Some(match event {
            "click" => Self::Click,
            "clickdown" | "pointerdown" => Self::ClickDown,
            "clickup" | "pointerup" => Self::ClickUp,
            "mouseenter" | "pointerenter" => Self::PointerEnter,
            "mouseleave" | "pointerleave" => Self::PointerLeave,
            "mousemove" | "pointermove" => Self::PointerMove,
            "doubleclick" => Self::DoubleClick,
            "dragstart" => Self::DragStart,
            "dragmove" => Self::DragMove,
            "dragend" => Self::DragEnd,
            "dragcancel" => Self::DragCancel,
            "longpress" => Self::LongPress,
            "scroll" => Self::Scroll,
            "load" => Self::Load,
            "unload" => Self::Unload,
            _ => return None,
        })
    }

    fn matches(self, event: InteractionEventType) -> bool {
        matches!(
            (self, event),
            (Self::Click, InteractionEventType::Click)
                | (Self::ClickDown, InteractionEventType::PointerDown)
                | (Self::ClickUp, InteractionEventType::PointerUp)
                | (Self::PointerEnter, InteractionEventType::PointerEnter)
                | (Self::PointerLeave, InteractionEventType::PointerLeave)
                | (Self::PointerMove, InteractionEventType::PointerMove)
                | (Self::DoubleClick, InteractionEventType::DoubleClick)
                | (Self::DragStart, InteractionEventType::DragStart)
                | (Self::DragMove, InteractionEventType::DragMove)
                | (Self::DragEnd, InteractionEventType::DragEnd)
                | (Self::DragCancel, InteractionEventType::DragCancel)
                | (Self::LongPress, InteractionEventType::LongPress)
                | (Self::Scroll, InteractionEventType::Scroll)
        )
    }
}

/// Custom child setup invoked while a button's content is spawned.
pub type ButtonChildSetup = fn(&mut ChildSpawnerCommands, Color);

/// A child definition used by [`BeverlyButton::children`].
#[derive(Clone, Debug)]
pub enum ButtonChild {
    Text(String),
    Icon(&'static str),
    Custom(ButtonChildSetup),
}

/// Starts a primary button builder.
pub fn button() -> UiElement {
    UiElement::button()
}

impl ButtonChild {
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    pub fn icon(value: &'static str) -> Self {
        Self::Icon(value)
    }

    pub fn custom(setup: ButtonChildSetup) -> Self {
        Self::Custom(setup)
    }
}

/// Semantic color role for a [`BeverlyButton`], matching the common
/// primary/secondary/success/danger/warning/info/light/dark palette used by
/// most design systems, plus a text-only appearance with no fill or border.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonColor {
    Primary,
    Secondary,
    Success,
    Danger,
    Warning,
    Info,
    Light,
    Dark,
    /// No fill or border; renders as themed text only, like a hyperlink.
    Text,
}

impl ButtonColor {
    /// Resting-state color this variant is built around. `Text` has no fill
    /// or border, so callers handle it separately rather than relying on this.
    fn base_color(self, colors: ThemeColors) -> Color {
        match self {
            Self::Primary => colors.primary,
            Self::Secondary => colors.secondary,
            Self::Success => colors.success,
            Self::Danger => colors.error,
            Self::Warning => colors.warning,
            Self::Info => colors.info,
            Self::Light => colors.light_surface,
            Self::Dark => colors.dark_surface,
            Self::Text => Color::NONE,
        }
    }
}

/// A themed, accessible push button.
///
/// Insert on any entity and `ButtonPlugin` builds the label/icon children,
/// wires up pointer and keyboard interaction (via the shared
/// `InteractionPlugin`/`A11yPlugin` primitives), and keeps AccessKit informed
/// for screen readers.
///
/// ```
/// # use bevy::prelude::*;
/// # use beverly::components::button::BeverlyButton;
/// fn spawn(mut commands: Commands) {
///     commands.spawn((
///         Node::default(),
///         BeverlyButton::primary("Deploy").on("click", deploy),
///     ));
/// # fn deploy(_commands: &mut Commands, _button: Entity) {}
/// }
/// ```
#[derive(Component, Clone, Debug)]
pub struct BeverlyButton {
    pub label: String,
    pub icon: Option<&'static str>,
    pub children: Vec<ButtonChild>,
    pub handlers: Vec<(ButtonEventType, ButtonCommand)>,
    pub color: ButtonColor,
    /// Transparent fill with a colored border/label instead of a solid fill.
    pub outline: bool,
    /// Blocks pointer/keyboard activation and announces "disabled" to
    /// assistive technology instead of only dimming the button visually.
    pub disabled: bool,
    /// Stretches to the full width of its parent, like Bootstrap's `.btn-block`.
    pub block: bool,
}

impl BeverlyButton {
    pub fn new(color: ButtonColor, label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            children: Vec::new(),
            handlers: Vec::new(),
            color,
            outline: false,
            disabled: false,
            block: false,
        }
    }

    pub fn primary(label: impl Into<String>) -> Self {
        Self::new(ButtonColor::Primary, label)
    }

    pub fn secondary(label: impl Into<String>) -> Self {
        Self::new(ButtonColor::Secondary, label)
    }

    pub fn success(label: impl Into<String>) -> Self {
        Self::new(ButtonColor::Success, label)
    }

    pub fn danger(label: impl Into<String>) -> Self {
        Self::new(ButtonColor::Danger, label)
    }

    pub fn warning(label: impl Into<String>) -> Self {
        Self::new(ButtonColor::Warning, label)
    }

    pub fn info(label: impl Into<String>) -> Self {
        Self::new(ButtonColor::Info, label)
    }

    pub fn light(label: impl Into<String>) -> Self {
        Self::new(ButtonColor::Light, label)
    }

    pub fn dark(label: impl Into<String>) -> Self {
        Self::new(ButtonColor::Dark, label)
    }

    /// Text-only appearance: no fill or border, just themed text (like a link).
    pub fn text_button(label: impl Into<String>) -> Self {
        Self::new(ButtonColor::Text, label)
    }

    pub fn icon(mut self, icon: &'static str) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn text(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self.children.clear();
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = ButtonChild>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn on(mut self, event: &'static str, command: ButtonCommand) -> Self {
        let event = ButtonEventType::parse(event)
            .unwrap_or_else(|| panic!("unsupported button event type: {event}"));
        self.handlers.push((event, command));
        self
    }

    pub fn outline(mut self, outline: bool) -> Self {
        self.outline = outline;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn block(mut self, block: bool) -> Self {
        self.block = block;
        self
    }
}

#[derive(Component)]
struct ButtonLabel {
    owner: Entity,
}

#[derive(Component)]
struct ButtonIcon {
    owner: Entity,
}

#[derive(Component, Clone)]
struct ButtonEventBindings(Vec<(ButtonEventType, ButtonCommand)>);

pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<crate::primitives::semantic::SemanticAccessibilityPlugin>() {
            app.add_plugins(crate::primitives::semantic::SemanticAccessibilityPlugin);
        }
        app.add_systems(
            Update,
            (
                spawn_button_ui,
                button_a11y_system,
                button_visual_system,
                button_lifecycle_system,
                button_pointer_event_system,
            )
                .chain(),
        );
    }
}

fn invoke_handlers(
    commands: &mut Commands,
    button: &BeverlyButton,
    entity: Entity,
    event: ButtonEventType,
) {
    for (bound_event, handler) in &button.handlers {
        if *bound_event == event {
            handler(commands, entity);
        }
    }
}

fn button_lifecycle_system(
    mut commands: Commands,
    added: Query<(Entity, &BeverlyButton), Added<BeverlyButton>>,
    mut removed: RemovedComponents<BeverlyButton>,
    bindings: Query<&ButtonEventBindings>,
) {
    for (entity, button) in &added {
        invoke_handlers(&mut commands, button, entity, ButtonEventType::Load);
    }

    for entity in removed.read() {
        let Ok(bindings) = bindings.get(entity) else {
            continue;
        };
        for (event, handler) in &bindings.0 {
            if *event == ButtonEventType::Unload {
                handler(&mut commands, entity);
            }
        }
    }
}

fn button_pointer_event_system(
    mut commands: Commands,
    events: Option<MessageReader<UiPointerEvent>>,
    buttons: Query<&BeverlyButton>,
) {
    let Some(mut events) = events else {
        return;
    };
    for event in events.read() {
        let Ok(button) = buttons.get(event.context.target) else {
            continue;
        };
        for candidate in [
            ButtonEventType::Click,
            ButtonEventType::ClickDown,
            ButtonEventType::ClickUp,
            ButtonEventType::PointerEnter,
            ButtonEventType::PointerLeave,
            ButtonEventType::PointerMove,
            ButtonEventType::DoubleClick,
            ButtonEventType::DragStart,
            ButtonEventType::DragMove,
            ButtonEventType::DragEnd,
            ButtonEventType::DragCancel,
            ButtonEventType::LongPress,
            ButtonEventType::Scroll,
        ] {
            if candidate.matches(event.event_type) {
                invoke_handlers(&mut commands, button, event.context.target, candidate);
            }
        }
    }
}

/// Creates the visual representation of newly-added buttons.
fn spawn_button_ui(
    mut commands: Commands,
    buttons: Query<(Entity, &BeverlyButton), Added<BeverlyButton>>,
    theme: Res<ThemeResource>,
) {
    let colors = theme.current.colors;

    for (entity, button) in &buttons {
        let (fill, border, foreground) = resolve_button_colors(button, colors, Interaction::None);

        let accessible_label = button
            .children
            .iter()
            .find_map(|child| match child {
                ButtonChild::Text(value) => Some(value.clone()),
                ButtonChild::Icon(_) => None,
                ButtonChild::Custom(_) => None,
            })
            .unwrap_or_else(|| button.label.clone());
        let mut semantic = SemanticNode::new(SemanticRole::Button).label(accessible_label);
        semantic.state.disabled = button.disabled;

        let mut entity_commands = commands.entity(entity);
        entity_commands.insert((
            Button,
            Node {
                display: Display::Flex,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(8.0),
                padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                width: if button.block {
                    Val::Percent(100.0)
                } else {
                    Val::Auto
                },
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderColor::all(Color::NONE),
            Surface::rounded_rect_fill(8.0, Paint::solid(fill))
                .uniform_border(1.0, Paint::solid(border)),
            a11y::TabIndex(if button.disabled { -1 } else { 0 }),
            semantic,
            ButtonEventBindings(button.handlers.clone()),
        ));

        if button.disabled {
            entity_commands.insert(DisabledInteraction);
        }

        entity_commands.with_children(|parent| {
            let children = if button.children.is_empty() {
                let mut defaults = Vec::new();
                if let Some(icon) = button.icon {
                    defaults.push(ButtonChild::Icon(icon));
                }
                defaults.push(ButtonChild::Text(button.label.clone()));
                defaults
            } else {
                button.children.clone()
            };

            for child in children {
                match child {
                    ButtonChild::Icon(icon) => {
                        let icon_entity =
                            parent.spawn_icon_colored(Icon::feather(icon), 16.0, foreground);
                        parent
                            .commands()
                            .entity(icon_entity)
                            .insert(ButtonIcon { owner: entity });
                    }
                    ButtonChild::Text(value) => {
                        parent.spawn((
                            ButtonLabel { owner: entity },
                            Text::new(value),
                            TextFont {
                                font_size: FontSize::Px(15.0),
                                ..default()
                            },
                            TextColor(foreground),
                        ));
                    }
                    ButtonChild::Custom(setup) => setup(parent, foreground),
                }
            }
        });
    }
}

/// Keeps semantic state and `DisabledInteraction`/`TabIndex` gating in sync.
fn button_a11y_system(
    mut commands: Commands,
    mut buttons: Query<
        (
            Entity,
            &BeverlyButton,
            &mut a11y::TabIndex,
            Has<DisabledInteraction>,
        ),
        Changed<BeverlyButton>,
    >,
) {
    for (entity, button, mut tab_index, has_disabled_marker) in &mut buttons {
        tab_index.0 = if button.disabled { -1 } else { 0 };

        if button.disabled && !has_disabled_marker {
            commands.entity(entity).insert(DisabledInteraction);
        } else if !button.disabled && has_disabled_marker {
            commands.entity(entity).remove::<DisabledInteraction>();
        }
    }
}

/// Keeps fill/border/label/icon colors and the `block` width in sync with the
/// button's color, outline, disabled, and pointer-interaction state.
fn button_visual_system(
    theme: Res<ThemeResource>,
    mut buttons: Query<(&BeverlyButton, &Interaction, &mut Surface, &mut Node)>,
    mut labels: Query<(&ButtonLabel, &mut TextColor)>,
    mut icons: Query<(&ButtonIcon, &mut IconNode)>,
) {
    let colors = theme.current.colors;

    for (button, interaction, mut surface, mut node) in &mut buttons {
        let (fill, border, _) = resolve_button_colors(button, colors, *interaction);
        surface.fill = Paint::solid(fill);
        if let Some(border_style) = surface.border.as_mut() {
            border_style.paint = Paint::solid(border);
        }
        node.width = if button.block {
            Val::Percent(100.0)
        } else {
            Val::Auto
        };
    }

    for (label, mut text_color) in &mut labels {
        if let Ok((button, interaction, _, _)) = buttons.get(label.owner) {
            let (_, _, foreground) = resolve_button_colors(button, colors, *interaction);
            text_color.0 = foreground;
        }
    }

    for (icon, mut icon_node) in &mut icons {
        if let Ok((button, interaction, _, _)) = buttons.get(icon.owner) {
            let (_, _, foreground) = resolve_button_colors(button, colors, *interaction);
            icon_node.color = foreground;
        }
    }
}

/// Resolves `(fill, border, foreground)` colors for the current variant,
/// outline/disabled flags, and pointer-interaction state.
fn resolve_button_colors(
    button: &BeverlyButton,
    colors: ThemeColors,
    interaction: Interaction,
) -> (Color, Color, Color) {
    if button.disabled {
        let foreground = colors.text_disabled;
        return if button.outline || matches!(button.color, ButtonColor::Text) {
            (Color::NONE, colors.border, foreground)
        } else {
            (colors.border, colors.border, foreground)
        };
    }

    if matches!(button.color, ButtonColor::Text) {
        let foreground = match interaction {
            Interaction::Pressed => colors.primary_active,
            Interaction::Hovered => colors.primary_hover,
            Interaction::None => colors.primary,
        };
        return (Color::NONE, Color::NONE, foreground);
    }

    let base = button.color.base_color(colors);

    if button.outline {
        let fill = match interaction {
            Interaction::None => Color::NONE,
            Interaction::Hovered => base.with_alpha(0.12),
            Interaction::Pressed => base.with_alpha(0.22),
        };
        return (fill, base, base);
    }

    let fill = match (button.color, interaction) {
        (ButtonColor::Primary, Interaction::None) => colors.primary,
        (ButtonColor::Primary, Interaction::Hovered) => colors.primary_hover,
        (ButtonColor::Primary, Interaction::Pressed) => colors.primary_active,
        (_, Interaction::None) => base,
        (_, Interaction::Hovered) => tint_for_state(base, 0.08),
        (_, Interaction::Pressed) => tint_for_state(base, 0.16),
    };

    (fill, fill, readable_foreground(fill))
}

/// Nudges `color` toward black or white (whichever increases contrast) by
/// `amount`, used for generic hover/press feedback on colors that don't have
/// their own hand-tuned hover/active theme entries.
fn tint_for_state(color: Color, amount: f32) -> Color {
    let toward = if color.to_linear().luminance() < 0.5 {
        Color::WHITE
    } else {
        Color::BLACK
    };
    color.mix(&toward, amount)
}

/// Picks white or near-black foreground text/icon color for a solid `fill`,
/// independent of the active theme (a light theme's dark text would be
/// unreadable on a light theme's own light-neutral background, for example).
fn readable_foreground(fill: Color) -> Color {
    if fill.to_linear().luminance() < 0.5 {
        Color::WHITE
    } else {
        Color::srgb(0.059, 0.090, 0.165)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::light_theme;
    use bevy::a11y::AccessibilityNode;

    fn test_app() -> App {
        let mut app = App::new();
        app.insert_resource(ThemeResource {
            current: light_theme(),
        })
        .add_plugins(ButtonPlugin);
        app
    }

    #[test]
    fn primary_button_spawns_surface_and_accessible_button_node() {
        let mut app = test_app();
        let entity = app.world_mut().spawn(BeverlyButton::primary("Deploy")).id();
        app.update();

        let world = app.world();
        assert!(world.get::<Surface>(entity).is_some());
        assert_eq!(world.get::<a11y::TabIndex>(entity).unwrap().0, 0);
        assert_eq!(
            world.get::<AccessibilityNode>(entity).unwrap().0.label(),
            Some("Deploy")
        );
        assert!(world.get::<DisabledInteraction>(entity).is_none());
    }

    #[test]
    fn disabled_button_blocks_interaction_and_is_announced_disabled() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn(BeverlyButton::danger("Delete").disabled(true))
            .id();
        app.update();

        let world = app.world();
        assert_eq!(world.get::<a11y::TabIndex>(entity).unwrap().0, -1);
        assert!(
            world
                .get::<AccessibilityNode>(entity)
                .unwrap()
                .0
                .is_disabled()
        );
        assert!(world.get::<DisabledInteraction>(entity).is_some());
    }

    #[test]
    fn toggling_disabled_at_runtime_updates_a11y_state() {
        let mut app = test_app();
        let entity = app.world_mut().spawn(BeverlyButton::primary("Save")).id();
        app.update();
        assert!(app.world().get::<DisabledInteraction>(entity).is_none());

        app.world_mut()
            .get_mut::<BeverlyButton>(entity)
            .unwrap()
            .disabled = true;
        app.update();
        assert!(app.world().get::<DisabledInteraction>(entity).is_some());
        assert_eq!(app.world().get::<a11y::TabIndex>(entity).unwrap().0, -1);

        app.world_mut()
            .get_mut::<BeverlyButton>(entity)
            .unwrap()
            .disabled = false;
        app.update();
        assert!(app.world().get::<DisabledInteraction>(entity).is_none());
        assert_eq!(app.world().get::<a11y::TabIndex>(entity).unwrap().0, 0);
    }

    #[test]
    fn block_button_stretches_to_full_width() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn(BeverlyButton::secondary("Continue").block(true))
            .id();
        app.update();
        assert_eq!(
            app.world().get::<Node>(entity).unwrap().width,
            Val::Percent(100.0)
        );
    }

    #[test]
    fn non_block_button_keeps_auto_width() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn(BeverlyButton::secondary("Continue"))
            .id();
        app.update();
        assert_eq!(app.world().get::<Node>(entity).unwrap().width, Val::Auto);
    }

    #[test]
    fn text_variant_has_no_fill_or_border() {
        let colors = light_theme().colors;
        let button = BeverlyButton::text_button("Learn more");
        let (fill, border, _) = resolve_button_colors(&button, colors, Interaction::None);
        assert_eq!(fill, Color::NONE);
        assert_eq!(border, Color::NONE);
    }

    #[test]
    fn outline_variant_is_transparent_at_rest_and_tinted_on_hover() {
        let colors = light_theme().colors;
        let button = BeverlyButton::danger("Delete").outline(true);
        let (rest_fill, border, foreground) =
            resolve_button_colors(&button, colors, Interaction::None);
        assert_eq!(rest_fill, Color::NONE);
        assert_eq!(border, colors.error);
        assert_eq!(foreground, colors.error);

        let (hover_fill, _, _) = resolve_button_colors(&button, colors, Interaction::Hovered);
        assert_ne!(hover_fill, Color::NONE);
    }

    #[test]
    fn solid_variants_pick_readable_foreground_for_every_color() {
        let colors = light_theme().colors;
        for color in [
            ButtonColor::Primary,
            ButtonColor::Secondary,
            ButtonColor::Success,
            ButtonColor::Danger,
            ButtonColor::Warning,
            ButtonColor::Info,
            ButtonColor::Light,
            ButtonColor::Dark,
        ] {
            let button = BeverlyButton::new(color, "").text("Label");
            let (fill, _, foreground) = resolve_button_colors(&button, colors, Interaction::None);
            let fill_is_dark = fill.to_linear().luminance() < 0.5;
            let foreground_is_white = foreground == Color::WHITE;
            assert_eq!(
                fill_is_dark, foreground_is_white,
                "{color:?} should pick a contrasting foreground for its fill"
            );
        }
    }

    #[test]
    fn disabled_buttons_use_neutral_muted_colors_regardless_of_variant() {
        let colors = light_theme().colors;
        let button = BeverlyButton::success("Save").disabled(true);
        let (fill, border, foreground) =
            resolve_button_colors(&button, colors, Interaction::Hovered);
        assert_eq!(fill, colors.border);
        assert_eq!(border, colors.border);
        assert_eq!(foreground, colors.text_disabled);
    }
}
