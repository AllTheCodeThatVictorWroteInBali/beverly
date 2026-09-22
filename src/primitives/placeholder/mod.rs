//! Legacy shader placeholders. New loading UI should use `ui::skeleton`.
//! The historical layout-only `Skeleton` is now `LegacySkeletonGroup`;
//! these compatibility exports point to the single native implementation.

mod component;
mod effects;
mod material;
mod systems;

pub use component::*;
pub use effects::*;
pub use material::*;
pub use systems::*;
pub use crate::animation::skeleton::{Skeleton, SkeletonGroup};

use bevy::prelude::*;

#[deprecated(note = "Use ui::skeleton::SkeletonPlugin for new loading UI; retained for legacy Placeholder entities")]
pub struct PlaceholderPlugin;

#[allow(deprecated)]
impl Plugin for PlaceholderPlugin {
    fn build(&self, app: &mut App) {
        bevy::asset::embedded_asset!(app, "placeholder.wgsl");
        app.add_plugins(UiMaterialPlugin::<PlaceholderMaterial>::default())
            .add_systems(
                Update,
                (
                    setup_placeholders,
                    animate_placeholders,
                    update_placeholder_materials,
                    update_placeholder_visuals,
                ),
            );
    }
}
