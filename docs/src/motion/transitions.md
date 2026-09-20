# Transitions

Transitions are the most common motion pattern in product interfaces. They connect one state to the next and help users understand what changed without feeling lost in the UI.

## What transitions should do

A transition should serve one of three goals:

- show that a change has occurred
- preserve context during a layout shift
- orient the user toward the next logical action

This means transitions are usually short, smooth, and tied to a meaningful state change such as an expanded panel, a selected tab, or a newly visible alert.

## Good transition qualities

- subtle and directional, not distracting
- consistent across similar UI states
- tied to a clear semantic change
- readable at a glance even without animation

## Typical motion patterns

### Panel and drawer motion

Panels should move in a controlled way that suggests hierarchy. The movement should feel steady and maintain the user's attention on the content, not on the motion itself.

### Tab and state switches

When switching between views or modes, the UI should preserve the relationship between the old and new state. Content can crossfade or shift slightly, but it should not feel like a separate visual event.

### Lists and collections

Insert, remove, and reorder actions should remain legible. A list item entering or leaving should not create sudden jumps that make the layout hard to track.

## Timing guidance

Keep transitions compact and predictable:

- short change: 120-200 ms
- medium change: 200-300 ms
- larger layout reflow: 300-400 ms

Avoid long easing curves or oversized travel distances. The motion should feel like a subtle explanation, not a spectacle.

## Accessibility note

Transitions should still be understandable when animation is disabled. If a panel is shown or hidden, the content should remain readable and focus should land predictably. Layout changes should not trap users or reset context unexpectedly.

## Rule of thumb

If the interface still communicates the state clearly without animation, the motion is probably too elaborate. Aim for confidence, not flourish.

