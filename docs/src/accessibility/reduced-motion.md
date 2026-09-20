# Reduced Motion

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Reduced-motion preferences exist to protect users from distraction, vestibular discomfort, and motion overload. Beverly should treat motion as a design signal, not as a required delivery mechanism for understanding.

## Core principle

When motion is reduced, the interface should stay clear and usable. The meaning of state changes, focus movement, and responsive behavior must remain intact even if animations are shortened or removed.

## What should calm down

- decorative hover and press effects
- large panel reveal transitions
- list insertion or removal animation
- background effects that do not carry meaning
- repeated micro-animations on controls and cards

## What should stay clear

- focus movement
- status updates
- loading and progress states
- error and warning communication
- app state transitions that are essential to context

## Recommended implementation

Prefer a shared preference model instead of component-by-component exceptions. Motion policies should be driven centrally so the app behaves consistently across panels, cards, and overlays.

If reduced motion is enabled:

- shorten or remove non-essential transitions
- keep state changes immediate and understandable
- preserve readability and focus flow
- avoid hiding important information behind animation timing

## Rule of thumb

If motion is removed and the interface still communicates clearly, the design is working. Reduced motion should reduce distraction without reducing understanding.

## Example: motion-aware state transitions

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn spawn_status(mut commands: Commands) {
    let prefers_reduced_motion = false;

    commands.spawn((
        NodeBundle::default(),
        BeverlyAlert::new("Sync complete")
            .variant(AlertVariant::Success)
            .with_transition(if prefers_reduced_motion {
                TransitionPolicy::Instant
            } else {
                TransitionPolicy::Fade
            }),
    ));
}
```

This pattern preserves clarity while reducing unnecessary motion when the user has requested a calmer interaction model.
