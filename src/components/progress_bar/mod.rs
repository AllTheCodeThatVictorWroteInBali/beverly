mod component;
mod systems;

pub use component::*;
pub use systems::*;

use bevy::a11y::AccessibilityNode;
use bevy::prelude::*;

fn sync_progress_bar_a11y(
    mut bars: Query<(&ProgressBar, &mut AccessibilityNode), Changed<ProgressBar>>,
) {
    for (bar, mut node) in &mut bars {
        node.0
            .set_numeric_value((bar.progress.clamp(0.0, 1.0) * 100.0) as f64);
    }
}

pub struct ProgressBarPlugin;

impl Plugin for ProgressBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_progress_bars, sync_progress_bar_a11y));
    }
}
