use bevy::prelude::*;

/// Shows the pointer (hand) cursor while this entity is hovered, to signal it is clickable.
///
/// Applied by `ButtonPlugin`'s cursor system, which also owns the other cursor shapes.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct PointerCursorOnHover;

/// Shows the default arrow cursor while this entity is hovered.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct DefaultCursorOnHover;
