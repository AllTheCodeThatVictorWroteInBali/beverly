# Motion

Motion is one of the clearest ways to communicate change in a UI, but it should never become the primary message. Beverly treats motion as a supportive layer: it reinforces structure, confirms state, and helps users understand transitions without distracting them from their task.

## Principles

- Use motion to reinforce state changes, not to replace clarity
- Keep movement brief, direct, and predictable
- Preserve continuity when the interface changes
- Respect `prefers-reduced-motion` without sacrificing comprehension
- Favor system-level consistency over isolated decorative effects

## What motion should do

Motion is useful when it answers a simple question for the user:

- What changed?
- Where did focus go?
- Is this action still running?
- Is the content moving in a known direction?

A good motion pattern makes the interface easier to track. A weak one adds noise and increases cognitive load.

## Timing and scale

The motion system should remain compact and consistent across the product. Beverly keeps motion within a narrow timing band so transitions feel intentional rather than dramatic.

- micro-interactions: 120-180 ms
- short state changes: 180-260 ms
- medium layout updates: 260-400 ms
- longer flows: only when there is a real need to show progress or context

Longer durations should be the exception, not the default. If an interaction takes more than a few hundred milliseconds, it should usually be paired with clear status text or loading feedback.

## Examples of how to add motion

The exact API names may vary by implementation, but the pattern should stay consistent: motion should be attached to a semantic state change, use a short, controlled duration, and fall back cleanly when reduced motion is active.

### Example: animated panel reveal

```rust
use bevy::prelude::*;
use std::time::Duration;

fn build_expanding_panel(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyPanel::new()
            .with_transition(Transition::slide_in(
                Duration::from_millis(220),
                Easing::standard(),
            ))
            .with_initial_state(PanelState::Collapsed),
    ));
}
```

This keeps the change tied to a real state transition. The panel opens with a short slide or fade, and the motion supports the user's understanding of the control rather than acting as a decorative flourish.

### Example: button feedback

```rust
use bevy::prelude::*;

fn handle_press(mut button: Query<(&mut Transform, &Interaction), Changed<Interaction>>) {
    for (mut transform, interaction) in &mut button {
        if *interaction == Interaction::Pressed {
            transform.scale = Vec3::splat(0.98);
        } else {
            transform.scale = Vec3::ONE;
        }
    }
}
```

This type of micro-interaction should feel immediate but subtle. The component should not wobble, overshoot, or distract from the action itself.

### Example: reduced-motion-aware transition

```rust
use bevy::prelude::*;

fn transition_duration(reduced_motion: Res<ReducedMotion>) -> Duration {
    if reduced_motion.enabled() {
        Duration::ZERO
    } else {
        Duration::from_millis(220)
    }
}
```

This is the key rule: when motion is reduced, remove or shorten the non-essential animation while keeping the state change itself clear and accessible.

## State transitions

Transitions are the most common use of motion in Beverly. They should make state changes feel natural and legible without stealing attention.

Examples include:

- panels expanding and collapsing
- tabs or sections switching
- filtered lists updating
- dialog or drawer entrances and exits
- content appearing as the user changes context

In these cases, motion should support continuity. The user should feel that the interface is responding coherently, not that a new visual layer has appeared on top of the old one.

## Feedback and micro-interactions

Small interaction feedback is useful when it confirms a user action immediately. Examples include button press states, selection highlights, active-state pulses, and subtle confirmation motion on successful actions.

The key is restraint. Motion should be noticeable enough to feel responsive, but too small to become a performance or distraction problem. If the action is already obvious from labels, state, and color, extra motion may be unnecessary.

## Loading and progress

Loading movement should reassure the user that work is happening while preserving orientation. Beverly prefers lightweight, informative progress patterns over oversized or flashy effects.

Good loading states:

- maintain layout stability
- communicate progress or activity accurately
- avoid making the UI feel unstable or visually noisy
- remain readable in reduced motion modes

When exact progress is unknown, motion should communicate activity without implying false precision. A quiet pulse or placeholder skeleton is usually more reliable than an over-animated indicator.

## Reduced motion

Reduced motion is a core accessibility requirement, not a secondary option. Beverly should respect `prefers-reduced-motion` by decreasing or removing non-essential animation while preserving semantic clarity.

When motion is reduced:

- decorative loops should be suppressed
- non-critical transitions can become immediate
- status still needs to remain clear through text, structure, and contrast
- content should not lose context during layout changes

The goal is not to make the interface static; it is to keep it understandable and calm.

## Motion checklist

Before shipping any motion pattern, ask:

- Does it clarify state or hierarchy?
- Is it short enough to feel responsive?
- Does it still make sense without animation?
- Does it respect reduced-motion preferences?
- Does it help the user understand the next action, not just look polished?

If motion cannot answer those questions, the safer default is a simple state change with strong structure and clear labels.

## Summary

Beverly’s motion language should feel quiet, intentional, and trustworthy. Motion is a tool for guidance, not spectacle. When used well, it helps people understand the interface at a glance while keeping the system calm, accessible, and efficient.
