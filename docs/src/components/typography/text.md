# Text

Text carries the supporting content around a title: explanations, labels, metadata, helper copy, and status details. It is the flexible, lower-emphasis counterpart to the title primitive and should inherit the active theme's typography rules.

## When to use

Use themed text when content belongs inside an existing hierarchy rather than naming the hierarchy itself. Common uses include:

- descriptions below a title
- labels attached to controls
- captions and timestamps
- helper and validation messages
- muted metadata or status details

Use `ThemedTitle` when the text introduces a meaningful screen, panel, card, or section.

## Text roles

`TextRole` gives common content a semantic size and color:

- `Heading` - title-sized supporting heading
- `Body` - ordinary readable copy
- `Label` - compact text for controls and field labels
- `Caption` - small muted supporting information
- `Muted` - body-sized secondary copy
- `Accent` - compact text using the theme's primary color
- `Disabled` - text for unavailable controls or content

The active theme resolves the exact size and color for each role. Prefer a role change over a manual override so the text remains coherent across themes.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_metadata(mut commands: Commands) {
	commands.spawn((
		Text::new("Last synced 2 minutes ago"),
		ThemedText::new(TextRole::Caption),
	));
}
```

## Overrides

When a single piece of text genuinely needs a different treatment, use `.size(...)` or `.color(...)` on `ThemedText`. These overrides are local decisions and should not replace the shared role system:

```rust
commands.spawn((
	Text::new("Sync failed"),
	ThemedText::new(TextRole::Body)
		.color(Color::srgb(0.86, 0.2, 0.2)),
));
```

Manual colors bypass the theme's default contrast choices, so verify them in every theme where the text appears. Font family is controlled through the `Typography` component or the application's global typography setup.

## Guidance

- use the smallest role that preserves comfortable reading
- keep labels and metadata concise
- use muted roles for secondary information, not essential instructions
- let the surrounding layout control text spacing and alignment
- allow body text to wrap rather than clipping or forcing a fixed height
