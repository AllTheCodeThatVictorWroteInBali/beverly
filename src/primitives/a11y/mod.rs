//! Core accessibility primitives shared by every widget in this library.
//!
//! This module wires three things together:
//!
//! 1. **Keyboard focus** — `bevy_input_focus`'s `InputFocus`/`InputFocusVisible`
//!    resources are already installed by `DefaultPlugins`. This module adds
//!    [`TabNavigationPlugin`] on top so Tab / Shift+Tab move focus between any
//!    entity that carries [`TabIndex`], scoped to the nearest ancestor
//!    [`TabGroup`].
//! 2. **Keyboard activation** — normalized key events and default semantic
//!    actions (Enter, Space, arrows, Home/End) are provided by
//!    `crate::primitives::keyboard::KeyboardPlugin`, which converges keyboard input
//!    into the same semantic action pipeline used by pointer/a11y/automation.
//! 3. **Screen reader semantics** — the `node`/`*_node` helpers build an
//!    AccessKit [`accesskit::Node`] (wrapped as bevy's [`AccessibilityNode`])
//!    describing a widget's role, label and state for assistive technology.
//!
//! Widgets are expected to attach [`TabIndex`] + the relevant `*_node` helper
//! at spawn time, and keep the node's state (checked/selected/expanded/value)
//! in sync via their existing visual-update systems.

use accesskit::{Node as AccessKitNode, Role, Toggled};
use bevy::a11y::AccessibilityNode;
use bevy::input_focus::tab_navigation::TabNavigationPlugin;
use bevy::input_focus::{InputFocus, InputFocusVisible};
use bevy::prelude::*;

pub use bevy::input_focus::tab_navigation::{TabGroup, TabIndex};
pub use bevy::input_focus::{AutoFocus, FocusCause, FocusGained, FocusLost, FocusedInput};

use crate::rendering::{Paint, Surface};
use crate::animation::animation::{SurfaceTransitionTarget, should_animate_target, themed_transition};
use crate::theme::{
    AccessibilityVisualPolicyResource,
    FocusStyleRequest,
    FocusVisibilityPolicy,
    SurfaceContext,
    SurfaceTone,
    ThemeResource,
};

/// Installs Tab-key navigation plus this library's focus ring semantics on
/// top of Bevy's default UI interaction handling.
///
/// `InputFocusPlugin`, `InputDispatchPlugin` and `AccessibilityPlugin` are
/// already added by `DefaultPlugins`, so this only needs to add the pieces
/// specific to keyboard navigation and this component library's styling.
pub struct A11yPlugin;

impl Plugin for A11yPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TabNavigationPlugin)
            .add_systems(Update, sync_focus_ring);
    }
}

// ---------------------------------------------------------------------------
// AccessKit node builders
// ---------------------------------------------------------------------------

/// Builds an [`AccessibilityNode`] with the given AccessKit `role` and label.
pub fn node(role: Role, label: impl Into<String>) -> AccessibilityNode {
    let mut n = AccessKitNode::new(role);
    n.set_label(label.into());
    AccessibilityNode(n)
}

/// Marks (or clears) the disabled state on an existing accessibility node.
pub fn set_disabled(node: &mut AccessibilityNode, disabled: bool) {
    if disabled {
        node.0.set_disabled();
    } else {
        node.0.clear_disabled();
    }
}

pub fn button_node(label: impl Into<String>) -> AccessibilityNode {
    node(Role::Button, label)
}

pub fn link_node(label: impl Into<String>) -> AccessibilityNode {
    node(Role::Link, label)
}

pub fn image_node(label: impl Into<String>) -> AccessibilityNode {
    node(Role::Image, label)
}

pub fn text_input_node(label: impl Into<String>) -> AccessibilityNode {
    node(Role::TextInput, label)
}

pub fn search_input_node(label: impl Into<String>) -> AccessibilityNode {
    node(Role::SearchInput, label)
}

pub fn password_input_node(label: impl Into<String>) -> AccessibilityNode {
    node(Role::PasswordInput, label)
}

pub fn number_input_node(label: impl Into<String>) -> AccessibilityNode {
    node(Role::NumberInput, label)
}

pub fn email_input_node(label: impl Into<String>) -> AccessibilityNode {
    node(Role::EmailInput, label)
}

pub fn multiline_text_input_node(label: impl Into<String>) -> AccessibilityNode {
    node(Role::MultilineTextInput, label)
}

pub fn checkbox_node(
    label: impl Into<String>,
    checked: bool,
    indeterminate: bool,
) -> AccessibilityNode {
    let mut n = AccessKitNode::new(Role::CheckBox);
    n.set_label(label.into());
    n.set_toggled(toggled_state(checked, indeterminate));
    AccessibilityNode(n)
}

pub fn switch_node(label: impl Into<String>, checked: bool) -> AccessibilityNode {
    let mut n = AccessKitNode::new(Role::Switch);
    n.set_label(label.into());
    n.set_toggled(toggled_state(checked, false));
    AccessibilityNode(n)
}

pub fn radio_node(label: impl Into<String>, selected: bool) -> AccessibilityNode {
    let mut n = AccessKitNode::new(Role::RadioButton);
    n.set_label(label.into());
    n.set_toggled(toggled_state(selected, false));
    AccessibilityNode(n)
}

pub fn radio_group_node(label: impl Into<String>) -> AccessibilityNode {
    node(Role::RadioGroup, label)
}

pub fn tab_node(label: impl Into<String>, selected: bool) -> AccessibilityNode {
    let mut n = AccessKitNode::new(Role::Tab);
    n.set_label(label.into());
    n.set_selected(selected);
    AccessibilityNode(n)
}

pub fn tab_list_node(label: impl Into<String>) -> AccessibilityNode {
    node(Role::TabList, label)
}

pub fn tab_panel_node(label: impl Into<String>) -> AccessibilityNode {
    node(Role::TabPanel, label)
}

pub fn slider_node(label: impl Into<String>, value: f32, min: f32, max: f32) -> AccessibilityNode {
    let mut n = AccessKitNode::new(Role::Slider);
    n.set_label(label.into());
    n.set_numeric_value(value as f64);
    n.set_min_numeric_value(min as f64);
    n.set_max_numeric_value(max as f64);
    AccessibilityNode(n)
}

pub fn combo_box_node(label: impl Into<String>, expanded: bool) -> AccessibilityNode {
    let mut n = AccessKitNode::new(Role::ComboBox);
    n.set_label(label.into());
    n.set_expanded(expanded);
    AccessibilityNode(n)
}

pub fn list_box_option_node(label: impl Into<String>, selected: bool) -> AccessibilityNode {
    let mut n = AccessKitNode::new(Role::ListBoxOption);
    n.set_label(label.into());
    n.set_selected(selected);
    AccessibilityNode(n)
}

pub fn spin_button_node(label: impl Into<String>) -> AccessibilityNode {
    node(Role::SpinButton, label)
}

pub fn dialog_node(label: impl Into<String>) -> AccessibilityNode {
    let mut n = AccessKitNode::new(Role::Dialog);
    n.set_label(label.into());
    n.set_modal();
    AccessibilityNode(n)
}

pub fn progress_node(label: impl Into<String>, progress_0_to_1: f32) -> AccessibilityNode {
    let mut n = AccessKitNode::new(Role::ProgressIndicator);
    n.set_label(label.into());
    n.set_numeric_value((progress_0_to_1.clamp(0.0, 1.0) * 100.0) as f64);
    n.set_min_numeric_value(0.0);
    n.set_max_numeric_value(100.0);
    AccessibilityNode(n)
}

/// A polite live region, e.g. for a status message that updates in place.
pub fn status_node(label: impl Into<String>) -> AccessibilityNode {
    node(Role::Status, label)
}

/// An assertive live region, e.g. a toast that should interrupt the screen
/// reader to announce itself immediately.
pub fn alert_node(label: impl Into<String>) -> AccessibilityNode {
    node(Role::Alert, label)
}

fn toggled_state(on: bool, indeterminate: bool) -> Toggled {
    if indeterminate {
        Toggled::Mixed
    } else if on {
        Toggled::True
    } else {
        Toggled::False
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Resolves semantic focus decoration on the focused surface and writes it
/// into [`Surface::decorations`], independent from the surface border.
fn sync_focus_ring(
    mut commands: Commands,
    focus: Res<InputFocus>,
    focus_visible: Res<InputFocusVisible>,
    theme: Res<ThemeResource>,
    policy: Res<AccessibilityVisualPolicyResource>,
    mut last: Local<Option<Entity>>,
    mut surfaces: Query<(Entity, &mut Surface, Option<&SurfaceTransitionTarget>)>,
) {
    if !focus.is_changed()
        && !focus_visible.is_changed()
        && !theme.is_changed()
        && !policy.is_changed()
    {
        return;
    }

    let focused = focus.get();
    let should_show = match policy.current.focus_visibility {
        FocusVisibilityPolicy::Always => focused.is_some(),
        FocusVisibilityPolicy::KeyboardOnly => focused.is_some() && focus_visible.0,
        FocusVisibilityPolicy::Programmatic => focused.is_some(),
        FocusVisibilityPolicy::Hidden => false,
    };
    let wanted = if should_show { focused } else { None };

    if *last != wanted {
        if let Some(previous) = *last {
            if let Ok((entity, surface, current_target)) = surfaces.get_mut(previous) {
                let mut next_surface = (*surface).clone();
                next_surface.decorations.focus_ring = None;

                if should_animate_target(&current_target.map(|value| value.target.clone()), &next_surface) {
                    commands.entity(entity).insert(SurfaceTransitionTarget::new(
                        next_surface,
                        themed_transition(&theme, |tokens| tokens.focus),
                    ));
                }
            }
        }
        *last = wanted;
    }

    if let Some(current) = wanted {
        if let Ok((entity, surface, current_target)) = surfaces.get_mut(current) {
            let tone = if surface.backdrop.is_some() {
                SurfaceTone::Glass
            } else {
                SurfaceTone::Neutral
            };

            let dark_background = match &surface.fill {
                Paint::Solid(color) => color.to_linear().luminance() < 0.5,
                Paint::Shimmer(shimmer) => shimmer.base_color.to_linear().luminance() < 0.5,
                Paint::LinearGradient(_)
                | Paint::RadialGradient(_)
                | Paint::AngularGradient(_) => theme.current.mode == crate::theme::ThemeMode::Dark,
            };

            let context = SurfaceContext {
                tone,
                dark_background,
            };

            let mut next_surface = (*surface).clone();
            next_surface.decorations.focus_ring = theme.current.colors.focus_ring(
                FocusStyleRequest {
                    visible: true,
                    ..Default::default()
                },
                context,
                policy.current,
            );

            if should_animate_target(&current_target.map(|value| value.target.clone()), &next_surface) {
                commands.entity(entity).insert(SurfaceTransitionTarget::new(
                    next_surface,
                    themed_transition(&theme, |tokens| tokens.focus),
                ));
            }
        }
    }
}
