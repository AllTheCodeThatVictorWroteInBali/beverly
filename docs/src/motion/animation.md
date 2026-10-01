# Animation

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Animation is useful when a small, purposeful movement makes a system response feel immediate and understandable. Beverly favors restrained motion over expressive, loop-heavy effects. The best animations in a design system feel like feedback, not performance.

## Enter And Exit Motion

Use `UiMotion` when an entity should animate as it appears or before it is removed. `UiMotion::fade()`, `UiMotion::fade_up()`, and `UiMotion::pop()` provide paired enter and exit settings; `UiMotion::new` and `UiMotionSpec` let you choose each side separately.

```rust
use bevy::prelude::*;
use beverly::animation::motion::{UiMotion, UiMotionExitRequested};

fn spawn_panel(mut commands: Commands) {
	commands.spawn((
		Node::default(),
		UiMotion::fade_up(),
	));
}

fn close_panel(mut commands: Commands, panel: Entity) {
	commands.entity(panel).insert(UiMotionExitRequested);
}
```

The enter animation begins when `UiMotion` is added. To play it again, call `restart_enter()` on the component. To animate removal, add `UiMotionExitRequested`; the exit phase then completes and despawns the entity and its children. An entity without an exit spec is removed immediately when an exit is requested.

Presets affect opacity for solid `Surface` fills and `TextColor`, and use the node's top margin for vertical movement. The `Pop` preset uses a small vertical offset; it is not a scale animation. Configure enter and exit independently with `UiMotionSpec::fade`, `fade_up`, `fade_down`, or `pop`, and optionally add a delay with `.delay(seconds)`.

## Appropriate uses

Animation is valuable for:

- button press and release feedback
- active selection or status changes
- indicator movement tied to state
- safe micro-interactions that reinforce action without interrupting the task
- confirmation that a state has changed or a task is underway

It is not ideal for decorative effects that do not reveal state, structure, or intent.

## Design guidance

- Keep animation physically plausible and visually light
- Prefer easing that feels measured and calm
- Avoid frequent or repeated motion that competes with content
- Let the animation reinforce the action, not replace clarity
- Keep animation durations aligned with the broader motion system

## Motion categories

### Micro-interactions

Micro-interactions should feel responsive and immediate. A press state, hover reveal, or selection pulse should be quick enough that the user perceives the system as responsive rather than delayed.

A micro-interaction should answer the question, “Did the system receive my input?” without creating a separate entertainment layer.

### Status and feedback

When something changes, such as a success state or an active filter, animation can help confirm a result. The motion should read as confirmation, not as a separate event.

This makes animation a communication tool rather than a purely aesthetic one.

### Continuous motion

Continuous motion is acceptable only when it is informative. Progress indicators, status pulses, and subtle ambient motion can support awareness, but they should never become the primary focus of the interface.

In a professional product, continuous decorative motion is usually a sign that the animation is doing too much.

## Practical limits

Avoid:

- looping animations on static content
- large objects moving across the viewport for no reason
- repeated bounce or wobble patterns
- motion that makes reading difficult or makes controls feel unstable
- long, accidental delays that create the feeling of lag

The best motion is brief, clear, and aligned with the action it supports.

## Reduced-Motion Behavior

`UiMotion` enter/exit presets currently run independently of `AccessibilityVisualPolicyResource`; they are not automatically suppressed when reduced motion is enabled. If your application uses these presets, check the policy before attaching `UiMotion` or provide an immediate/static alternative. Property transitions have separate policy behavior described in [Transitions](./transitions.md).

Do not assume Beverly automatically detects the platform's `prefers-reduced-motion` setting. The shared policy can be initialized using `UI_REDUCED_MOTION` or set directly by the application. A state change must remain understandable without the visual flourish.

