//! Native loading silhouettes. Layout belongs to `Node`, paint to `Surface`,
//! animation time to the renderer, and loading semantics to `SkeletonGroup`.

mod component;
mod group;
mod systems;

pub use component::Skeleton;
pub use group::{SkeletonGroup, SkeletonTextLines};
pub use crate::rendering::ShimmerDirection as SkeletonDirection;

use bevy::prelude::*;
use bevy::ui::UiSystems;
use crate::theme::{AccessibilityVisualPolicyResource, ThemeResource};

/// Resolves skeleton authoring before native layout and Surface material sync.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SkeletonSystems;

pub struct SkeletonPlugin;

impl Plugin for SkeletonPlugin {
    fn build(&self, app: &mut App) {
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