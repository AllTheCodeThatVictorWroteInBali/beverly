//! Shared presentation for actual playback controls. Scene markers require this
//! component; scene systems own playback and only publish `playing` here.

use bevy::{a11y::AccessibilityNode, prelude::*, ui::UiSystems};

use crate::rendering::{Backdrop, InnerShadow, LiquidGlass, OuterShadow, Surface};
use crate::animation::animation::spring::spring_step;
use crate::icons::{Icon, IconNode};
use crate::primitives::a11y;
use crate::primitives::interaction::DisabledInteraction;
use crate::primitives::semantic::{SemanticNode, SemanticRole};
use crate::theme::{AccessibilityContrastMode, AccessibilityVisualPolicyResource, ThemeResource};

/// Playback state is independent of pointer state. Disable a control with
/// `DisabledInteraction`; its business handler must also exclude that marker.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
#[require(Button, a11y::TabIndex(0), PlayButtonSpring)]
pub struct PlayButton {
    pub playing: bool,
}

#[derive(Component, Clone, Copy, Default, PartialEq)]
struct PlayButtonSpring {
    amount: f32,
    velocity: f32,
}

/// Attach to a [`PlayButton`] entity that should stay hidden until revealed,
/// e.g. a hover-only control in a list row. Set `visible` to show it; absence
/// of this component means the button is always fully visible.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct PlayButtonHoverReveal {
    pub visible: bool,
    amount: f32,
    velocity: f32,
}

#[derive(Component)]
struct PlayButtonContent;

/// Scene state bridges run before this set, after all normal Update writers.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayButtonSystems;

#[derive(Resource)]
struct PlayButtonEnvironment {
    backdrop_disabled: bool,
    reduced_effects: bool,
}

impl Default for PlayButtonEnvironment {
    fn default() -> Self {
        let enabled = |name| matches!(
            std::env::var(name).ok().as_deref(),
            Some("1" | "true" | "TRUE" | "yes" | "on")
        );
        Self {
            backdrop_disabled: enabled("UI_BACKDROP_DISABLED"),
            reduced_effects: enabled("UI_REDUCED_EFFECTS"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendering::{FocusRing, Paint};
    use crate::theme::{dark_theme, light_theme};
    use std::time::Duration;

    fn test_app() -> App {
        let mut app = App::new();
        app.init_resource::<Time>()
            .insert_resource(ThemeResource { current: light_theme() })
            .add_plugins(PlayButtonPlugin);
        // Tests are deterministic regardless of the launching process's flags.
        app.world_mut().insert_resource(PlayButtonEnvironment {
            backdrop_disabled: false,
            reduced_effects: false,
        });
        app.world_mut().resource_mut::<AccessibilityVisualPolicyResource>().current = default();
        app
    }

    fn spawn_control(app: &mut App, size: f32) -> (Entity, Entity, Entity) {
        let button = app.world_mut().spawn((
            PlayButton::default(),
            Node {
                height: px(36.0),
                border_radius: BorderRadius::all(px(18.0)),
                ..default()
            },
            BackgroundColor(Color::WHITE),
        )).id();
        let icon = app.world_mut().spawn((
            IconNode::new(Icon::feather("play")).size(size),
            ChildOf(button),
            Node { width: px(size), height: px(size), ..default() },
        )).id();
        let label = app.world_mut().spawn((Text::new("Play"), TextColor(Color::WHITE), ChildOf(button))).id();
        (button, icon, label)
    }

    #[test]
    fn play_button_bootstraps_glass_and_preserves_original_icon_sizes() {
        let mut app = test_app();
        for size in [13.0, 14.0, 17.0] {
            let (button, icon, _) = spawn_control(&mut app, size);
            app.update();
            let world = app.world();
            let surface = world.get::<Surface>(button).unwrap();
            let backdrop = surface.backdrop.unwrap();
            assert_eq!(backdrop.blur, 1.4);
            assert_eq!(backdrop.saturation, 1.06);
            assert_eq!(backdrop.tint_opacity, 0.10);
            assert_eq!(surface.fill, Paint::solid(Color::NONE));
            assert_eq!(world.get::<BackgroundColor>(button).unwrap().0, Color::NONE);
            assert_eq!(world.get::<a11y::TabIndex>(button).unwrap().0, 0);
            assert_eq!(world.get::<AccessibilityNode>(button).unwrap().0.label(), Some("Play"));
            assert_eq!(world.get::<IconNode>(icon).unwrap().size, size);
            assert_eq!(world.get::<Node>(icon).unwrap().width, px(size));
            assert_eq!(world.get::<IconNode>(icon).unwrap().color, light_theme().colors.text);
        }
    }

    #[test]
    fn play_button_syncs_pause_theme_and_disabled_semantics() {
        let mut app = test_app();
        let (button, icon, label) = spawn_control(&mut app, 14.0);
        app.update();
        app.world_mut().get_mut::<PlayButton>(button).unwrap().playing = true;
        app.world_mut().resource_mut::<ThemeResource>().current = dark_theme();
        app.update();
        assert_eq!(app.world().get::<Text>(label).unwrap().0, "Pause");
        assert_eq!(app.world().get::<AccessibilityNode>(button).unwrap().0.label(), Some("Pause"));
        assert_eq!(app.world().get::<SemanticNode>(button).unwrap().label.as_deref(), Some("Pause"));
        assert_eq!(app.world().get::<IconNode>(icon).unwrap().icon, Icon::feather("pause"));
        assert_eq!(app.world().get::<IconNode>(icon).unwrap().color, dark_theme().colors.text);
        app.world_mut().entity_mut(button).insert((DisabledInteraction, Interaction::Pressed));
        app.update();
        assert!(app.world().get::<AccessibilityNode>(button).unwrap().0.is_disabled());
        let state = &app.world().get::<SemanticNode>(button).unwrap().state;
        assert!(state.disabled);
        assert!(!state.pressed);
        assert_eq!(app.world().get::<a11y::TabIndex>(button).unwrap().0, -1);
        assert_eq!(app.world().get::<PlayButtonSpring>(button).unwrap().amount, 0.0);
        app.world_mut().entity_mut(button).remove::<DisabledInteraction>();
        app.update();
        assert!(!app.world().get::<AccessibilityNode>(button).unwrap().0.is_disabled());
        assert_eq!(app.world().get::<a11y::TabIndex>(button).unwrap().0, 0);
    }

    #[test]
    fn play_button_falls_back_for_every_visual_policy_and_environment_flag() {
        for variant in 0..5 {
            let mut app = test_app();
            let (button, _, _) = spawn_control(&mut app, 17.0);
            match variant {
                0 => app.world_mut().resource_mut::<AccessibilityVisualPolicyResource>().current.reduced_transparency = true,
                1 => app.world_mut().resource_mut::<AccessibilityVisualPolicyResource>().current.reduced_effects = true,
                2 => app.world_mut().resource_mut::<AccessibilityVisualPolicyResource>().current.contrast = AccessibilityContrastMode::High,
                3 => app.world_mut().resource_mut::<PlayButtonEnvironment>().backdrop_disabled = true,
                _ => app.world_mut().resource_mut::<PlayButtonEnvironment>().reduced_effects = true,
            }
            app.update();
            let surface = app.world().get::<Surface>(button).unwrap();
            assert!(surface.backdrop.is_none());
            assert_eq!(surface.fill, Paint::solid(light_theme().colors.surface.with_alpha(1.0)));
            if variant == 2 { assert!(surface.border.is_some()); }
        }
    }

    #[test]
    fn play_button_press_deforms_only_glass_and_reduced_motion_snaps() {
        let mut app = test_app();
        let (button, icon, _) = spawn_control(&mut app, 14.0);
        app.update();
        app.world_mut().resource_mut::<Time>().advance_by(Duration::from_secs_f32(1.0 / 60.0));
        *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::Pressed;
        app.update();
        let spring = app.world().get::<PlayButtonSpring>(button).unwrap();
        assert!(spring.amount > 0.0 && spring.amount < 1.0);
        assert_eq!(app.world().get::<IconNode>(icon).unwrap().size, 14.0);
        assert_eq!(app.world().get::<UiTransform>(button).unwrap().scale, Vec2::ONE);
        app.world_mut().resource_mut::<AccessibilityVisualPolicyResource>().current.reduced_motion = true;
        app.update();
        assert_eq!(app.world().get::<PlayButtonSpring>(button).unwrap().amount, 1.0);
        assert_eq!(app.world().get::<PlayButtonSpring>(button).unwrap().velocity, 0.0);
        *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::None;
        app.update();
        assert_eq!(app.world().get::<PlayButtonSpring>(button).unwrap().amount, 0.0);
    }

    #[test]
    fn play_button_hover_reveal_hides_by_default_and_fades_in_when_visible() {
        let mut app = test_app();
        let (button, icon, _) = spawn_control(&mut app, 14.0);
        app.world_mut().entity_mut(button).insert(PlayButtonHoverReveal::default());
        app.update();
        assert_eq!(app.world().get::<IconNode>(icon).unwrap().color.alpha(), 0.0);
        assert_eq!(app.world().get::<Surface>(button).unwrap().backdrop.unwrap().tint_opacity, 0.0);

        app.world_mut().get_mut::<PlayButtonHoverReveal>(button).unwrap().visible = true;
        app.world_mut().resource_mut::<Time>().advance_by(Duration::from_secs_f32(1.0 / 60.0));
        app.update();
        let mid_alpha = app.world().get::<IconNode>(icon).unwrap().color.alpha();
        assert!(mid_alpha > 0.0 && mid_alpha < 1.0);

        app.world_mut().resource_mut::<AccessibilityVisualPolicyResource>().current.reduced_motion = true;
        app.update();
        assert_eq!(app.world().get::<IconNode>(icon).unwrap().color.alpha(), 1.0);
        assert_eq!(app.world().get::<Surface>(button).unwrap().backdrop.unwrap().tint_opacity, 0.10);

        app.world_mut().resource_mut::<AccessibilityVisualPolicyResource>().current.reduced_transparency = true;
        app.world_mut().get_mut::<PlayButtonHoverReveal>(button).unwrap().visible = false;
        app.update();
        assert_eq!(app.world().get::<IconNode>(icon).unwrap().color.alpha(), 1.0);
    }

    #[derive(Resource, Default)]
    struct Changes(usize);

    fn count_changes(
        surfaces: Query<(), Changed<Surface>>,
        icons: Query<(), Changed<IconNode>>,
        labels: Query<(), Changed<Text>>,
        semantics: Query<(), Changed<SemanticNode>>,
        springs: Query<(), Changed<PlayButtonSpring>>,
        mut count: ResMut<Changes>,
    ) {
        count.0 = surfaces.iter().count() + icons.iter().count() + labels.iter().count()
            + semantics.iter().count() + springs.iter().count();
    }

    #[test]
    fn play_button_preserves_focus_decoration_and_does_not_dirty_static_frames() {
        let mut app = test_app();
        app.init_resource::<Changes>().add_systems(Last, count_changes);
        let (button, _, _) = spawn_control(&mut app, 14.0);
        app.world_mut().entity_mut(button).insert(
            Surface::rounded_rect_fill(18.0, Color::NONE).with_focus_ring(FocusRing::default()),
        );
        app.update();
        let decorations = app.world().get::<Surface>(button).unwrap().decorations.clone();
        app.update();
        assert_eq!(app.world().resource::<Changes>().0, 0);
        assert_eq!(app.world().get::<Surface>(button).unwrap().decorations, decorations);
    }
}

pub struct PlayButtonPlugin;

impl Plugin for PlayButtonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AccessibilityVisualPolicyResource>()
            .init_resource::<PlayButtonEnvironment>()
            .add_systems(
                PostUpdate,
                present_play_buttons
                    .in_set(PlayButtonSystems)
                    .before(UiSystems::Layout),
            );
    }
}

fn present_play_buttons(
    mut commands: Commands,
    time: Res<Time>,
    theme: Res<ThemeResource>,
    policy: Res<AccessibilityVisualPolicyResource>,
    environment: Res<PlayButtonEnvironment>,
    mut buttons: Query<(
        Entity,
        &PlayButton,
        &Node,
        &Interaction,
        Has<DisabledInteraction>,
        &mut PlayButtonSpring,
        Option<&mut PlayButtonHoverReveal>,
        Option<&mut Surface>,
        Option<&mut BackgroundColor>,
        Option<&mut SemanticNode>,
        Option<&mut AccessibilityNode>,
        &mut a11y::TabIndex,
    )>,
    children: Query<&Children>,
    mut content: Query<(
        Option<&mut Text>,
        Option<&mut TextColor>,
        Option<&mut IconNode>,
        Has<PlayButtonContent>,
    )>,
) {
    let colors = theme.current.colors;
    let high_contrast = policy.current.contrast == AccessibilityContrastMode::High;
    let fallback = high_contrast
        || policy.current.reduced_transparency
        || policy.current.reduced_effects
        || environment.backdrop_disabled
        || environment.reduced_effects;

    for (entity, button, node, interaction, disabled, mut spring, reveal, surface,
        background, semantic, accessible, mut tab_index) in &mut buttons
    {
        let pressed = !disabled && *interaction == Interaction::Pressed;
        let target = if pressed { 1.0 } else { 0.0 };
        let mut next_spring = *spring;
        if policy.current.reduced_motion || fallback || disabled {
            next_spring = PlayButtonSpring { amount: target, velocity: 0.0 };
        } else if spring.amount != target || spring.velocity != 0.0 {
            spring_step(
                &mut next_spring.amount,
                &mut next_spring.velocity,
                target,
                time.delta_secs(),
                260.0,
                26.0,
            );
        }
        spring.set_if_neq(next_spring);

        // Reduced transparency/high-contrast prioritize discoverability over
        // a hover-reveal effect, so always show the control in those modes.
        let force_visible = high_contrast || policy.current.reduced_transparency;
        let fade = if let Some(mut reveal) = reveal {
            if force_visible {
                reveal.set_if_neq(PlayButtonHoverReveal { visible: reveal.visible, amount: 1.0, velocity: 0.0 });
                1.0
            } else {
                let reveal_target = if reveal.visible { 1.0 } else { 0.0 };
                let mut next_reveal = *reveal;
                if policy.current.reduced_motion || fallback || disabled {
                    next_reveal.amount = reveal_target;
                    next_reveal.velocity = 0.0;
                } else if reveal.amount != reveal_target || reveal.velocity != 0.0 {
                    spring_step(
                        &mut next_reveal.amount,
                        &mut next_reveal.velocity,
                        reveal_target,
                        time.delta_secs(),
                        320.0,
                        30.0,
                    );
                }
                reveal.set_if_neq(next_reveal);
                next_reveal.amount.clamp(0.0, 1.0)
            }
        } else {
            1.0
        };

        let radius = match node.border_radius.top_left {
            Val::Px(value) if value > 0.0 => value,
            _ => match node.height {
                Val::Px(height) => height * 0.5,
                _ => 18.0,
            },
        };
        let mut next_surface = if fallback || disabled {
            Surface::rounded_rect_fill(radius, colors.surface.with_alpha(fade))
        } else {
            Surface::rounded_rect_fill(radius, Color::NONE)
                .with_backdrop(
                    Backdrop::default()
                        .with_blur(1.4)
                        .with_saturation(1.06)
                        .with_tint(colors.surface)
                        .with_tint_opacity(0.10 * fade)
                        .with_brightness(if *interaction == Interaction::Hovered { 1.04 } else { 1.0 })
                        .with_liquid_glass(LiquidGlass {
                            press_amount: if policy.current.reduced_motion { 0.0 } else { spring.amount.clamp(0.0, 1.0) },
                            specular_intensity: if *interaction == Interaction::Hovered { 0.34 } else { 0.28 },
                            ..Default::default()
                        }),
                )
                .outer_shadow(OuterShadow::small(Color::BLACK.with_alpha(fade)))
                .inner_shadow(InnerShadow::new(colors.border_strong)
                    .with_offset(Vec2::new(0.0, -0.75))
                    .with_blur(1.0)
                    .with_opacity(0.18 * fade))
        };
        if high_contrast {
            next_surface = next_surface.uniform_border(1.0, colors.text);
        }
        if let Some(mut surface) = surface {
            // Focus belongs to the existing accessibility/animation machinery.
            next_surface.decorations = surface.decorations.clone();
            surface.set_if_neq(next_surface);
        } else {
            commands.entity(entity).insert(next_surface);
        }
        if let Some(mut background) = background {
            background.set_if_neq(BackgroundColor(Color::NONE));
        } else {
            commands.entity(entity).insert(BackgroundColor(Color::NONE));
        }

        let label = if button.playing { "Pause" } else { "Play" };
        let index = if disabled { -1 } else { 0 };
        if tab_index.0 != index {
            tab_index.0 = index;
        }
        if let Some(mut semantic) = semantic {
            if semantic.label.as_deref() != Some(label) {
                semantic.label = Some(label.to_string());
            }
            if semantic.state.disabled != disabled {
                semantic.state.disabled = disabled;
            }
            if semantic.state.pressed != pressed {
                semantic.state.pressed = pressed;
            }
        } else {
            let mut semantic = SemanticNode::new(SemanticRole::Button).label(label);
            semantic.state.disabled = disabled;
            semantic.state.pressed = pressed;
            commands.entity(entity).insert(semantic);
        }
        if let Some(mut accessible) = accessible {
            if accessible.0.label() != Some(label) {
                accessible.0.set_label(label);
            }
            if accessible.0.is_disabled() != disabled {
                a11y::set_disabled(&mut accessible, disabled);
            }
        } else {
            let mut accessible = a11y::button_node(label);
            a11y::set_disabled(&mut accessible, disabled);
            commands.entity(entity).insert(accessible);
        }

        let base_foreground = if disabled && !high_contrast { colors.text_disabled } else { colors.text };
        let foreground = base_foreground.with_alpha(base_foreground.alpha() * fade);
        let glyph = Icon::feather(if button.playing { "pause" } else { "play" });
        for child in children.iter_descendants(entity) {
            let Ok((text, color, icon, initialized)) = content.get_mut(child) else { continue };
            if !initialized {
                commands.entity(child).insert((PlayButtonContent, Pickable::IGNORE));
            }
            if let Some(mut text) = text {
                if text.0 != label {
                    text.0 = label.to_string();
                }
            }
            if let Some(mut color) = color {
                color.set_if_neq(TextColor(foreground));
            }
            if let Some(mut icon) = icon {
                if icon.icon != glyph {
                    icon.icon = glyph.clone();
                }
                if icon.color != foreground {
                    icon.color = foreground;
                }
                // Never change icon.size, its Node, or its transform.
            }
        }
    }
}