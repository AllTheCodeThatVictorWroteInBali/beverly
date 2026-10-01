# Motion Overview

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Motion in Beverly should do one job well: communicate change without stealing focus from the task at hand. Used sparingly, motion clarifies hierarchy, reinforces state, and preserves continuity across transitions. Used carelessly, it becomes decoration and increases cognitive load.

## Beverly's Motion APIs

`BeverlyPlugin` installs both `UiAnimationPlugin` and `UiMotionPlugin`. For a smaller custom setup, add the plugin that owns the behavior you need.

Beverly provides two complementary mechanisms:

- **Property transitions** animate a value toward a target using a duration, easing curve, and optional delay. Public target components currently cover `Surface`, `TextColor`, `Transform`, and a node's left position as a percentage. See [Transitions](./transitions.md).
- **Enter and exit motion** uses the `UiMotion` component with built-in fade, fade-up, fade-down, and pop presets. See [Animation](./animation.md).

The broader motion chapter also covers [scrolling](./scrolling.md), [loading](./loading.md), and [reduced motion](./reduced-motion.md). These patterns should work together: animation may clarify a change, but state, labels, focus, and layout must remain understandable without it.

## Motion Principles

- Tie movement to a user action or meaningful state change.
- Keep similar interactions consistent in duration and easing.
- Prefer short, direct transitions over looping or decorative movement.
- Preserve reading position, focus, and orientation as content changes.
- Ensure the resulting state is clear when motion is absent.

## Choosing A Duration

Use the shortest duration that makes a change legible. Beverly's default `Transition` is 140 ms with ease-out; built-in theme transition values are shorter for interaction feedback and modestly longer for modal changes. Treat these as a consistent baseline, not a requirement to animate every property.

Large layout changes should not rely on long travel distances or delayed starts. If movement competes with the content, reduce its distance or remove it.

## Design Check

Before shipping a motion pattern, ask whether it communicates a real change, preserves focus and context, and remains clear when motion is reduced. If it fails any of those checks, simplify it or make the state change immediate.

