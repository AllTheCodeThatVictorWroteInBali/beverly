# Motion Overview

Motion in Beverly should do one job well: communicate change without stealing focus from the task at hand. Used sparingly, motion clarifies hierarchy, reinforces state, and preserves continuity across transitions. Used carelessly, it becomes decoration and increases cognitive load.

## Core ideas

- Motion should support understanding, not replace it
- Transitions should be short, predictable, and reversible
- Meaningful movement should follow user intent and system state
- Motion must degrade gracefully when reduced-motion preferences are enabled

## Principles

- Use motion to reinforce state changes, not to announce them after the fact
- Favor brief, direct animations over long or looping spectacle
- Preserve context when UI changes, especially during navigation or filtering
- Keep the motion system consistent across components so the interface feels coherent
- Provide static alternatives for critical information when motion is reduced or disabled

## Motion categories

Beverly treats motion as a small set of related patterns:

1. Transitions for opening, closing, switching, and layout changes
2. Animation for meaningful feedback such as press, focus, and status updates
3. Scrolling for content flow, navigation, and viewport stability
4. Loading states for long-running actions and deferred content
5. Reduced-motion behavior for accessibility and calm interfaces

## Recommended rhythm

The system should feel responsive without feeling busy. Typical durations should stay within a narrow range so the interface feels intentional instead of inconsistent.

- micro-interactions: 120-180 ms
- short transitions: 180-260 ms
- larger layout changes: 260-400 ms
- loading or background motion: avoid unless it adds meaning

## When motion is appropriate

Motion is most useful when it answers a real user question:

- "What changed?"
- "Where did focus go?"
- "Is this action still in progress?"
- "Am I moving through a known structure?"

If the answer is unclear, the animation should be removed or simplified.

## Example behavior

A panel expanding from collapsed to expanded state should:

- animate height or opacity smoothly
- maintain the user's reading position when possible
- keep focus on the triggering control or the newly active content
- avoid decorative loops that distract from the task

On reduced-motion devices, the same state change should still be clear through structure, contrast, and timing alone, without requiring a visual flourish.

## Design check

Before shipping a motion pattern, ask:

- Does it communicate state or hierarchy?
- Is it fast enough to feel responsive?
- Does it still make sense when motion is removed?
- Does it preserve accessibility and readability?

If motion cannot pass those checks, it is usually better as a static transition or a small opacity change.

