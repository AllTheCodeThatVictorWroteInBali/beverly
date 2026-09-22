//! Generic root/anchor primitives used by layout-aware components.
//!
//! Host applications are expected to spawn a UI root hierarchy and mark it
//! with these components/resources so Beverly components (icons, sidebar,
//! nav buttons, semantic tree helpers, ...) can discover it without Beverly
//! having to own application layout itself.

use bevy::prelude::*;

/// Shared font handles used by text-bearing Beverly components.
///
/// Host applications insert this resource with their own font handle(s)
/// after loading a font via the asset server.
#[derive(Resource, Clone)]
pub struct UiFonts {
    pub text: Handle<Font>,
}

/// Marker for the entity that hosts the application's primary scrollable content.
#[derive(Component)]
pub struct ContentRoot;

/// Marker for the root surface entity of the whole UI tree.
#[derive(Component)]
pub struct AppRootSurface;
