# Loading

Loading states should reassure the user that progress is happening without creating unnecessary motion. They should answer one question clearly: "What is happening now, and how far along is it?"

## Good loading patterns

Use loading patterns to indicate:

- a request is in flight
- content is being prepared
- a task is waiting on external state
- the UI is temporarily unavailable but remains in context

The best loading states maintain orientation and avoid surprising the user.

## Recommended behaviors

- prefer lightweight, non-blocking feedback for short actions
- use skeleton or placeholder structure for larger content surfaces
- show progress when a task has a meaningful duration or expected completion
- keep the existing layout stable so the user does not lose their place

## Avoid

- flashy progress ornaments that are unrelated to the actual task
- large full-screen loaders for small, local transitions
- motion that makes a loading state look more important than the content underneath
- reliance on animation alone when a textual status or label would be clearer

## Progress vs. activity

If progress cannot be measured, emphasize activity without implying precision. A subtle pulse or spinner is acceptable for background work, but it should not pretend to represent a concrete completion percentage.

## Reduced motion treatment

In reduced-motion modes, loading cues should become quieter and more static. A spinner may soften or disappear, while labels, placeholders, and status text carry the clarity. The key is to retain confidence without decorative motion.

