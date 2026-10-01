//! Headless fixtures for testing Beverly systems from downstream crates.
//!
//! Enable the `test-support` feature to use these helpers from integration
//! tests. The fixtures use Beverly's real input, focus, keyboard, interaction,
//! and semantic plugins without creating a window or GPU context.

use std::time::Duration;

use bevy::ecs::message::Messages;
use bevy::input::InputPlugin;
use bevy::input_focus::InputFocus;
use bevy::prelude::*;

use crate::primitives::focus::FocusPlugin;
use crate::primitives::interaction::InteractionPlugin;
use crate::primitives::keyboard::KeyboardPlugin;
use crate::primitives::semantic::SemanticPlugin;

/// Builds a headless app with Beverly's input, focus, keyboard, and pointer
/// interaction systems installed.
pub fn interaction_app() -> App {
    let mut app = App::new();
    app.init_resource::<Time>()
        .init_resource::<InputFocus>()
        .add_plugins((InputPlugin, FocusPlugin, KeyboardPlugin, InteractionPlugin));
    app
}

/// Builds an [`interaction_app`] with semantic tree and AccessKit projection
/// systems installed as well.
pub fn semantic_app() -> App {
    let mut app = interaction_app();
    app.add_plugins(SemanticPlugin);
    app
}

/// Advances deterministic Bevy time and runs one app update.
pub fn advance(app: &mut App, seconds: f32) {
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(seconds));
    app.update();
}

/// Drains messages emitted by a test app since its previous update.
pub fn drain_messages<M: Message>(app: &mut App) -> Vec<M> {
    app.world_mut()
        .resource_mut::<Messages<M>>()
        .drain()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interaction_fixture_runs_a_frame() {
        let mut app = interaction_app();
        advance(&mut app, 1.0 / 60.0);
    }

    #[test]
    fn semantic_fixture_runs_a_frame() {
        let mut app = semantic_app();
        advance(&mut app, 1.0 / 60.0);
    }
}
