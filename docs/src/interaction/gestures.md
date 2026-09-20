# Gestures

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Gestures support touch and pointer-based interactions for rich applications. They are especially important in interfaces that rely on direct manipulation, quick actions, or touch-first patterns, but they should still fit the same structural principles as the rest of the UI system.

## What gestures are for

A gesture is more than a raw input event. It is a pattern of motion or multi-step action that has semantic meaning in the app, such as:

- swipe to change a panel or tab
- drag to reorder items
- pan to navigate content
- pinch or scale gestures in media or canvas interactions
- long press or hold for secondary actions

These behaviors are powerful, but they should remain predictable and consistent. A gesture should feel like part of the interface language, not a hidden mechanic whose rules only exist in a single widget.

## Designing gestures carefully

Every gesture should have a clear user-facing purpose. If a gesture is not understandable or does not have a visible state, it creates friction and confusion. The design should make it obvious:

- what the gesture does
- when it is active
- whether it is safe to trigger
- how to cancel or reverse it

This matters even more in touch interfaces because gestures may be harder to discover than a visible button or control.

## Gesture safety and reversibility

Because gestures often map to direct manipulation, they should be designed carefully around safety and undoability. For example:

- dragging to reorder items should feel reversible or inspectable
- swipe actions should not permanently trigger without confirmation when high-impact
- touch actions should not accidentally trigger if the user is simply scrolling the page
- the system should clearly distinguish between selection gestures and destructive gestures

The more direct the gesture, the more important it is that the interface remains understandable and recoverable.

## Gesture patterns in design systems

The best design systems use gestures as part of a small, well-understood set rather than inventing custom patterns everywhere. This allows the product to remain coherent and reduces the learning curve for users.

Examples of sane gesture use include:

- horizontal swipe between panels or tabs
- vertical drag in scrollable content
- drag and drop interactions for list reordering
- long-press for alternative commands in a controlled menu or action layer

The key is not to eliminate gestures, but to make them deliberate and readable.

## Guidance

Gestures should support the interface without hiding meaning. They are best used when they reduce friction or enable natural direct manipulation, and they should always keep the user’s understanding and control in view.
