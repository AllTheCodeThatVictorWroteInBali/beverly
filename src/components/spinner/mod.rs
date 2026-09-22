use bevy::prelude::*;

/// Reusable loading spinner.
///
/// Attach this component to an entity containing an SVG/image node.
/// The entity will continuously rotate while the spinner is active.
#[derive(Component, Debug, Clone)]
pub struct Spinner {
    /// Rotation speed in radians per second.
    pub speed: f32,

    /// Whether the spinner is currently animating.
    pub active: bool,
}

impl Default for Spinner {
    fn default() -> Self {
        Self {
            speed: 5.0,
            active: true,
        }
    }
}

/// Optional anchor for keeping a spinning icon centered.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct SpinnerAnchor {
    pub center: Vec3,
    pub offset: Vec3,
}

impl Spinner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn speed(speed: f32) -> Self {
        Self {
            speed,
            ..Default::default()
        }
    }

    pub fn inactive() -> Self {
        Self {
            active: false,
            ..Default::default()
        }
    }
}

/// Plugin for the reusable spinner component.
pub struct SpinnerPlugin;

impl Plugin for SpinnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            animate_spinners.after(crate::icons::component::UiIconSyncSet::Sync),
        );
    }
}

/// Rotate every active spinner.
fn animate_spinners(
    time: Res<Time>,
    mut spinners: Query<(&Spinner, Option<&SpinnerAnchor>, &mut Transform)>,
) {
    for (spinner, anchor, mut transform) in &mut spinners {
        if !spinner.active {
            continue;
        }

        transform.rotate_z(spinner.speed * time.delta_secs());

        if let Some(anchor) = anchor {
            transform.translation = anchor.center - transform.rotation * anchor.offset;
        }
    }
}
