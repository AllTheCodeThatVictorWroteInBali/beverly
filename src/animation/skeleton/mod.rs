//! Native loading silhouettes. Layout belongs to `Node`, paint to `Surface`,
//! animation time to the renderer, and loading semantics to `SkeletonGroup`.

mod component;
mod group;
mod systems;

pub use crate::rendering::ShimmerDirection as SkeletonDirection;
pub use component::Skeleton;
pub use group::{SkeletonGroup, SkeletonTextLines};

use crate::primitives::semantic::SemanticAccessibilityPlugin;
use crate::theme::{AccessibilityVisualPolicyResource, ThemeResource};
use bevy::prelude::*;
use bevy::ui::UiSystems;

/// Resolves skeleton authoring before native layout and Surface material sync.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SkeletonSystems;

pub struct SkeletonPlugin;

impl Plugin for SkeletonPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<SemanticAccessibilityPlugin>() {
            app.add_plugins(SemanticAccessibilityPlugin);
        }
        app.init_resource::<ThemeResource>()
            .init_resource::<AccessibilityVisualPolicyResource>()
            .add_systems(
                PostUpdate,
                (systems::sync_skeletons, systems::sync_groups)
                    .in_set(SkeletonSystems)
                    .before(UiSystems::Prepare),
            );
    }
}

#[cfg(test)]
mod tests;
