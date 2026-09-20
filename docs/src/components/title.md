# Title

Titles are the primary hierarchy marker in Beverly. They help users scan a screen quickly and understand which surface they are looking at.

## When to use

Use title text for page headers, panel headings, card labels, and other high-emphasis content that should stand out from supporting copy.

## Implementation pattern

Block Studio treats title as a reusable semantic block rather than a one-off label. Beverly follows the same approach: the title component should inherit the shared font, theme, and spacing system instead of carrying custom styling per screen.

## Example

```rust
fn build_header(mut commands: Commands) {
    commands.spawn(BeverlyTitle::new("Workspace"));
}
```

## Guidance

- keep titles short and direct
- place them near the top of the relevant surface
- use them to anchor the page or section hierarchy