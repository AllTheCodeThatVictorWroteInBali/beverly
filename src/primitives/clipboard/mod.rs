// src/ui/clipboard/mod.rs

mod component;

pub use component::Clipboard;

use bevy::prelude::*;

pub struct ClipboardPlugin;

impl Plugin for ClipboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Clipboard>();
    }
}
