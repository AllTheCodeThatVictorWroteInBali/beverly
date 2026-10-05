# Dots

Dots is an animated run of trailing periods that signals ongoing work. It
cycles through no dots, `.`, `..`, and `...`, then returns to no dots,
repeating for as long as the entity exists.

## When to use

Use Dots after a short status phrase when something is loading, thinking, or
waiting and the result has no measurable progress:

- "Thinking" while a model prepares a response
- "Loading" while content is fetched
- "Waiting for connection" in a status line

Use a progress bar when the amount of work done is known. Use a spinner when
the indicator should stand alone without text.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_status(mut commands: Commands) {
    commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|row| {
            row.spawn((text("Thinking"), ThemedText::new(TextRole::Muted)));
            row.spawn((Dots::new(), ThemedText::new(TextRole::Muted)));
        });
}
```

`Dots` owns the entity's `Text`, so do not set the text yourself. Pair it with
`ThemedText` for color and size so it matches the surrounding copy.

## Options

- `Dots::new()` - three dots, 0.4 seconds per step
- `.interval(seconds)` - how long each step, including the empty one, is shown
- `.max_dots(count)` - the most dots shown before the cycle returns to none

## Guidance

- place Dots in its own text entity beside the label so the label does not
  re-wrap or shift as the dots change
- remove the entity when the work finishes; Dots does not stop on its own
- keep the interval slow enough to read; around 0.3 to 0.5 seconds works well
- do not rely on the animation alone to convey status, as the label carries the
  meaning
