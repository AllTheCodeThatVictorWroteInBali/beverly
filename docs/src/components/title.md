# Title

Titles are the primary hierarchy marker in Beverly. They help users scan a screen quickly and understand which surface they are looking at.

## When to use

Use title text for page headers, panel headings, card labels, and other high-emphasis content that should stand out from supporting copy. Titles should establish the immediate context before the user reads the details below.

## Implementation pattern

Block Studio treats title as a reusable semantic block rather than a one-off label. Beverly follows the same approach: the title component should inherit the shared font, theme, and spacing system instead of carrying custom styling per screen.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_header(mut commands: Commands) {
    commands.spawn(BeverlyTitle::new("Workspace"));
}
```

## Guidance

- keep titles short and direct
- place them near the top of the relevant surface
- use them to anchor the page or section hierarchy
- avoid stacking multiple large titles in a row without a clear relationship between them

## Accessibility

Titles should be semantic and meaningful. They help screen-reader users orient themselves in the interface and should fit the surrounding content without creating a noisy or overloaded reading order.
