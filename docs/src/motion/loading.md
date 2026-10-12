# Loading

## Live Demo: Progress Bar

<iframe src="../assets/demos/index.html?demo=progress_bar" title="Interactive Beverly progress bar demo" loading="lazy" allow="fullscreen" style="display: block; width: 100%; height: min(820px, 90vh); min-height: 580px; border: 0;"></iframe>

<a href="../assets/demos/index.html?demo=progress_bar" target="_blank" rel="noopener">Open the demo in a separate tab</a>.

## Live Demo: Spinner

<iframe src="../assets/demos/index.html?demo=spinner" title="Interactive Beverly spinner demo" loading="lazy" allow="fullscreen" style="display: block; width: 100%; height: min(820px, 90vh); min-height: 580px; border: 0;"></iframe>

<a href="../assets/demos/index.html?demo=spinner" target="_blank" rel="noopener">Open the demo in a separate tab</a>.

## Progress Bar API

`BeverlyPlugin` installs the progress bar systems. Spawn a bar inside a UI parent
with `spawn_progress_bar_into`; progress ranges from `0.0` to `1.0`.

```rust
use beverly::components::progress_bar::{ProgressBar, spawn_progress_bar_into};
use bevy::prelude::*;

fn add_progress(parent: &mut ChildSpawnerCommands) -> Entity {
	spawn_progress_bar_into(parent, ProgressBar::new().progress(0.56).size(300.0, 12.0))
}
```

Update the returned entity's `ProgressBar::progress` as work completes, keeping
the value in range. The visual fill animates toward the target. Keep a visible
label or percentage beside the bar; the example supplies these separately.

## Spinner API

The spinner example uses the bundled Feather `loader` icon at three sizes, not a
separate spinner builder. Beverly's icon systems animate the loader. Pair it
with status text for work whose completion percentage is unknown.

```rust
use beverly::icons::IconCommands;
use bevy::prelude::*;

fn add_activity(parent: &mut ChildSpawnerCommands) {
	parent.spawn_feather_sized("loader", 32.0);
	parent.spawn(Text::new("Loading content"));
}
```

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
- use skeleton or placeholder structure for larger content surfaces (see [Loading Skeleton](skeleton.md))
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

