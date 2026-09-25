# Title

Titles identify the screen, panel, card, or section a user is viewing. They are the highest-emphasis content primitive in a local hierarchy and should make the purpose of a surface clear at a glance.

## When to use

Use a title when text names a meaningful region or introduces the content that follows. Titles work well for:

- page and screen headings
- panel and card headers
- section labels in long forms or settings views
- modal headings that establish the task

Use `ThemedText` for supporting copy, metadata, labels, and status. A title should not be used only because text needs to be larger; its semantic role should justify the emphasis.

## Title levels

`TitleLevel` provides a predictable hierarchy from the largest display treatment to the smallest heading:

- `Display` - prominent app or feature-level heading
- `H1` - primary screen heading
- `H2` - major panel or subsection heading
- `H3` - standard section heading
- `H4` - supporting section heading
- `H5` - compact heading with muted emphasis
- `H6` - smallest heading, used sparingly

The active theme resolves the actual size and color for each level. Higher levels use stronger foreground emphasis, while `H4` through `H6` are progressively quieter for nested structure.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_panel_header(mut commands: Commands) {
	commands.spawn((
		Text::new("Project settings"),
		ThemedTitle::new(TitleLevel::H2),
	));
}
```

## Changing size or color

Prefer changing `TitleLevel` when the title's place in the hierarchy changes. When one title genuinely needs a one-off treatment, use `.size(...)` or `.color(...)` on `ThemedTitle`:

```rust
commands.spawn((
	Text::new("Ready to publish"),
	ThemedTitle::new(TitleLevel::H3)
		.size(24.0)
		.color(Color::srgb(0.12, 0.45, 0.32)),
));
```

Manual overrides bypass the corresponding theme defaults. Check contrast and verify the result in every theme where the title appears.

## Composition guidance

- keep one clear primary title for each major surface
- place the title before the content it introduces
- use supporting text for explanations instead of making titles paragraph-length
- preserve a consistent level relationship between neighboring sections
- allow long titles to wrap rather than forcing a fixed height that can clip text

Titles inherit the same typography pipeline as other text entities, so font family changes made through `Typography` or the global font manager can apply across the application without rewriting each title.
