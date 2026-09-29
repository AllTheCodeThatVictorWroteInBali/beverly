mod component;
mod systems;

pub use component::*;
pub use systems::*;

use bevy::prelude::*;
use crate::primitives::semantic::{SemanticNode, SemanticValue};

fn sync_progress_bar_a11y(
    mut bars: Query<(&ProgressBar, &mut SemanticNode), Changed<ProgressBar>>,
) {
    for (bar, mut semantic) in &mut bars {
        let value = SemanticValue::Range {
            value: (bar.progress.clamp(0.0, 1.0) * 100.0) as f64,
            min: 0.0,
            max: 100.0,
            step: None,
        };
        if semantic.semantic_value != value {
            semantic.semantic_value = value;
        }
    }
}

pub struct ProgressBarPlugin;

impl Plugin for ProgressBarPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<crate::primitives::semantic::SemanticAccessibilityPlugin>() {
            app.add_plugins(crate::primitives::semantic::SemanticAccessibilityPlugin);
        }
        app.add_systems(Update, (update_progress_bars, sync_progress_bar_a11y));
    }
}
