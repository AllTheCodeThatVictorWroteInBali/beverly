# Scrolling

Scrolling is one of the most common user interactions in a UI system, so it should feel stable, predictable, and respectful of reading flow.

## Goals

Good scrolling behavior should:

- maintain a clear reading position
- allow users to navigate large content sets without loss of context
- feel smooth without introducing jank or oversensitivity
- support keyboard, touch, and pointer navigation consistently

## Expectations

- The scroll position should remain understandable and recoverable
- Large content areas should not unexpectedly jump when content updates
- Nested layouts should preserve the user's attention without abrupt resets
- Sticky headers and focus targets should not fight the scroll rhythm

## Content-aware behavior

Scrolling decisions should depend on the surface:

- list views should prioritize predictable item flow
- dashboards should keep critical controls visible without overblocking content
- detail views should preserve focus and heading context during updates

When content changes under the user, prefer preserving the current context instead of forcing a jump to the top.

## Performance and smoothness

Motion is not just visual; it is also a perception problem. Smooth scrolling depends on predictable layout, limited reflow, and careful handling of expensive effects. Beverly should avoid heavy blur, expensive shadows, and large visual layers on frequently scrolling surfaces when a simpler treatment is enough.

## Accessibility and control

Users should remain in control of scrolling speed, momentum, and focus. Avoid motion that hijacks the viewport or introduces surprising snap behavior. If motion is reduced, the system should still allow safe and predictable navigation even when animation is toned down.

## Rule of thumb

Scrolling should support reading and orientation. If it calls attention to itself, it is doing too much.

