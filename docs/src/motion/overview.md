# Motion Overview

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Motion in Beverly should do one job well: communicate change without stealing focus from the task at hand. Used sparingly, motion clarifies hierarchy, reinforces state, and preserves continuity across transitions. Used carelessly, it becomes decoration and increases cognitive load.

## Core ideas

- Motion should support understanding, not replace it
- Transitions should be short, predictable, and reversible
- Meaningful movement should follow user intent and system state
- Motion must degrade gracefully when reduced-motion preferences are enabled
- The motion system should feel consistent across components, screens, and themes

## Principles

- Use motion to reinforce state changes, not to announce them after the fact
- Favor brief, direct animations over long or looping spectacle
- Preserve context when UI changes, especially during navigation or filtering
- Keep the motion system consistent across components so the interface feels coherent
- Provide static alternatives for critical information when motion is reduced or disabled
- Allow the user to feel control, not interruption

## Motion categories

Beverly treats motion as a small set of related patterns:

1. Transitions for opening, closing, switching, and layout changes
2. Animation for meaningful feedback such as press, focus, and status updates
3. Scrolling for content flow, navigation, and viewport stability
4. Loading states for long-running actions and deferred content
5. Reduced-motion behavior for accessibility and calm interfaces

These categories are not isolated features; they are part of a single motion language. A UI should feel coherent when a panel expands, a list updates, a button responds to input, and a loading layer appears.

## Recommended rhythm

The system should feel responsive without feeling busy. Typical durations should stay within a narrow range so the interface feels intentional instead of inconsistent.

- micro-interactions: 120-180 ms
- short transitions: 180-260 ms
- larger layout changes: 260-400 ms
- loading or background motion: avoid unless it adds meaning

This rhythm keeps motion subtle enough to feel polished while still being noticeable when the user needs to understand what changed.

## When motion is appropriate

Motion is most useful when it answers a real user question:

- “What changed?”
- “Where did focus go?”
- “Is this action still in progress?”
- “Am I moving through a known structure?”

If the answer is unclear, the animation should be removed or simplified. A decorative move that does not communicate state is usually a liability.

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
- Does it preserve user focus and orientation?

If motion cannot pass those checks, it is usually better as a static transition or a small opacity change.

## The product-level goal

Motion should make the UI feel alive without making it feel noisy. In a well-designed product, motion helps the user understand continuity and intent. The best motion is evidence of thoughtfulness: it gives feedback, preserves context, and fades into the background when it is no longer needed.

