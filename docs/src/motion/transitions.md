# Transitions

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Transitions are the most common motion pattern in product interfaces. They connect one state to the next and help users understand what changed without feeling lost in the UI. A transition is not just an effect; it is one of the ways the product communicates continuity, hierarchy, and context across state changes.

## Property Transitions In Beverly

Beverly represents a property transition with a target component. Insert or update that component on the entity along with the property it should animate. `UiAnimationPlugin` detects the target and interpolates from the current value; `BeverlyPlugin` already installs this plugin.

For example, a `Surface` can transition from its current fill to a new solid fill:

```rust
use std::time::Duration;

use bevy::prelude::*;
use beverly::animation::animation::{Easing, SurfaceTransitionTarget, Transition};
use beverly::rendering::{Paint, Shape, Surface};

fn highlight(commands: &mut Commands, entity: Entity) {
	let target = Surface::new(
		Shape::rounded_rect(8.0),
		Paint::solid(Color::srgb(0.12, 0.42, 0.78)),
	);
	let transition = Transition::new(Duration::from_millis(180), Easing::EaseOut);
	commands.entity(entity).insert(SurfaceTransitionTarget::new(target, transition));
}
```

The other public target components are `TextColorTransitionTarget`, `TransformTransitionTarget`, and `NodeLeftPercentTransitionTarget`. The first two accept a target value and a `Transition`; the node-left target accepts a percentage. For Bevy UI nodes, transform translation is interpreted in logical pixels and the supported transform animation is 2D translation, scale, and Z rotation. Non-UI entities retain full 3D `Transform` animation.

`Transition::new(duration, easing)` starts with no delay and the `Decorative` motion class. Use `.with_delay(...)` for a deliberate stagger, and `.with_motion_class(MotionClass::Semantic)` for a transition whose movement conveys essential state rather than decoration. Easing options include `Linear`, `EaseIn`, `EaseOut`, `EaseInOut`, `Smooth`, and `CubicBezier`.

When a new target interrupts a running transition, interpolation continues from the current value rather than snapping back to the original start. Setting the target equal to the current value completes immediately.

## What transitions should do

A transition should serve one of three goals:

- show that a change has occurred
- preserve context during a layout shift
- orient the user toward the next logical action

This means transitions are usually short, smooth, and tied to a meaningful state change such as an expanded panel, a selected tab, or a newly visible alert.

A transition that does not help the user understand the interface is usually just visual noise.

## Good transition qualities

- subtle and directional, not distracting
- consistent across similar UI states
- tied to a clear semantic change
- readable at a glance even without animation
- short enough to feel responsive

## Typical motion patterns

### Panel and drawer motion

Panels should move in a controlled way that suggests hierarchy. The movement should feel steady and maintain the user's attention on the content, not on the motion itself.

This is especially important when opening a drawer, expanding a panel, or switching between primary and secondary views. The motion should preserve the sense that the user is still in the same product rather than being displaced into a separate visual experience.

### Tab and state switches

When switching between views or modes, the UI should preserve the relationship between the old and new state. Content can crossfade or shift slightly, but it should not feel like a separate visual event.

The goal is clarity: the user should understand what changed and where they are now without losing orientation.

### Lists and collections

Insert, remove, and reorder actions should remain legible. A list item entering or leaving should not create sudden jumps that make the layout hard to track.

This is especially important in dynamic data surfaces where inserts, filters, and ordering changes happen frequently. The transition should help preserve continuity, not make the list harder to follow.

## Timing guidance

Keep transitions compact and predictable:

- short change: 120-200 ms
- medium change: 200-300 ms
- larger layout reflow: 300-400 ms

Avoid long easing curves or oversized travel distances. The motion should feel like a subtle explanation, not a spectacle.

## Accessibility note

Transitions should still be understandable when animation is disabled. If a panel is shown or hidden, the content should remain readable and focus should land predictably. Layout changes should not trap users or reset context unexpectedly.

A transition is not complete if it depends on the animation to preserve meaning.

Beverly's property-animation system checks `AccessibilityVisualPolicyResource`. When `reduced_motion` is enabled, decorative transitions resolve immediately (duration and delay become zero); semantic transitions retain their configured timing. The policy is initialized from the `UI_REDUCED_MOTION` environment variable (`1`, `true`, `yes`, or `on` enables it), and applications can also update the resource at runtime. Beverly does not automatically read an operating-system preference in this runtime.

## Rule of thumb

If the interface still communicates the state clearly without animation, the motion is probably too elaborate. Aim for confidence, not flourish.

