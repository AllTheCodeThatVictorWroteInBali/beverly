# Loading

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Loading states should reassure the user that progress is happening without creating unnecessary motion. They should answer one question clearly: “What is happening now, and how far along is it?” A loading state is not just a spinner; it is a statement about the app’s current state and the user’s expectation of continuity.

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
- explain what is loading when the state is not otherwise obvious

A loading state should reduce uncertainty, not create more of it.

## Avoid

- flashy progress ornaments that are unrelated to the actual task
- large full-screen loaders for small, local transitions
- motion that makes a loading state look more important than the content underneath
- reliance on animation alone when a textual status or label would be clearer
- false precision when the app does not actually know how much progress is complete

## Progress vs. activity

If progress cannot be measured, emphasize activity without implying precision. A subtle pulse or spinner is acceptable for background work, but it should not pretend to represent a concrete completion percentage.

This distinction matters because users interpret loading states as signals of trust and reliability. Honest states are much better than decorative ones.

## Loading and layout stability

A good loading UI usually preserves structure. Instead of replacing the entire screen with a generic loader, the interface may:

- show a localized loading state in the relevant panel
- keep the page layout intact while content is refreshed
- use placeholder content that hints at the final structure
- preserve the user’s place in the list or form while data is arriving

This makes the app feel stable and prevents the user from losing context during an asynchronous update.

## Reduced motion treatment

In reduced-motion modes, loading cues should become quieter and more static. A spinner may soften or disappear, while labels, placeholders, and status text carry the clarity. The key is to retain confidence without decorative motion.

Loading states should still be informative even when motion is reduced; they should just become more restrained and readable.

