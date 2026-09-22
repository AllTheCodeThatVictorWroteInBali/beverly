use accesskit::{Live, Node as AccessKitNode, Role};
use bevy::{a11y::AccessibilityNode, prelude::*};

use crate::rendering::{Paint, Shimmer, Surface};
use crate::theme::{AccessibilityContrastMode, AccessibilityVisualPolicyResource, Theme, ThemeResource};
use super::{Skeleton, SkeletonGroup, component::{SkeletonLayoutState, finite_nonnegative}};

pub(super) fn sync_skeletons(
    mut commands: Commands,
    mut cpu_metrics: Option<ResMut<crate::rendering::skeleton_metrics::SkeletonSyncCpuMetrics>>,
    theme: Res<ThemeResource>,
    policy: Res<AccessibilityVisualPolicyResource>,
    mut query: Query<(Entity, Ref<Skeleton>, &mut Node, &mut SkeletonLayoutState, Option<&mut Surface>)>,
) {
    let cpu_start = cpu_metrics.as_ref().map(|_| std::time::Instant::now());
    let preferences_changed = theme.is_changed() || policy.is_changed();
    for (entity, skeleton, mut node, mut layout, surface) in &mut query {
        if !preferences_changed && !skeleton.is_changed() && !node.is_changed() && surface.is_some() {
            continue;
        }

        // Never replace Node: margin, positioning, flex, clipping, etc. are caller-owned.
        if let Some(width) = skeleton.width {
            if node.width != width { node.width = width; }
        }
        let height = if let Some(height) = skeleton.height {
            Some(height)
        } else if skeleton.text && (node.height == Val::Auto || layout.text_height == Some(node.height)) {
            Some(Val::Px(finite_nonnegative(theme.current.typography.font_size_body, 16.0) * 1.25))
        } else {
            None
        };
        let text_height = if skeleton.text && skeleton.height.is_none() { height } else { None };
        if layout.text_height != text_height { layout.text_height = text_height; }
        if let Some(height) = height {
            if node.height != height { node.height = height; }
        }

        let next = resolved_surface(&skeleton, &theme.current, &policy);
        if let Some(mut surface) = surface {
            // Avoid marking Surface changed for typography-only/unrelated theme changes.
            if *surface != next { *surface = next; }
        } else {
            commands.entity(entity).insert(next);
        }
    }
    if let (Some(start), Some(metrics)) = (cpu_start, cpu_metrics.as_mut()) {
        metrics.record_skeleton_sync(start.elapsed());
    }
}

pub(super) fn resolved_surface(skeleton: &Skeleton, theme: &Theme, policy: &AccessibilityVisualPolicyResource) -> Surface {
    let high_contrast = policy.current.contrast == AccessibilityContrastMode::High;
    // High contrast deliberately overrides custom colors to keep a strong silhouette.
    let mut base = if high_contrast { theme.colors.text_muted } else {
        skeleton.base_color.unwrap_or(theme.colors.skeleton_base)
    };
    let mut highlight = skeleton.highlight_color.unwrap_or(theme.colors.skeleton_highlight);
    if high_contrast || policy.current.reduced_transparency {
        base = base.with_alpha(1.0);
        highlight = highlight.with_alpha(1.0);
    }

    // The shared animation infrastructure uses this same visual policy for
    // MotionClass::Decorative transitions. No independent motion setting/clock.
    let enabled = skeleton.enabled && !high_contrast
        && !policy.current.reduced_motion && !policy.current.reduced_effects
        && !theme.visual_effects.reduced_effects;
    let mut shimmer = Shimmer::new(base, highlight);
    shimmer.duration = skeleton.duration;
    shimmer.direction = skeleton.direction;
    shimmer.phase = skeleton.phase;
    shimmer.width = skeleton.highlight_width;
    shimmer.softness = skeleton.highlight_softness;
    shimmer.intensity = skeleton.highlight_intensity;
    shimmer.enabled = enabled;
    let shimmer = shimmer.sanitized();
    let fill = if enabled { Paint::Shimmer(shimmer) } else { Paint::solid(base) };
    Surface::rounded_rect_fill(skeleton.radius, fill)
}

pub(super) fn sync_groups(
    mut commands: Commands,
    mut query: Query<(Entity, Ref<SkeletonGroup>, Option<&mut AccessibilityNode>)>,
) {
    for (entity, group, accessibility) in &mut query {
        if !group.is_changed() && accessibility.is_some() { continue; }
        let mut next = AccessKitNode::new(Role::Group);
        next.set_label(group.label.clone());
        next.set_live(Live::Polite);
        if group.busy { next.set_busy(); }
        if let Some(mut accessibility) = accessibility {
            if accessibility.0 != next { accessibility.0 = next; }
        } else {
            commands.entity(entity).insert(AccessibilityNode(next));
        }
    }
}