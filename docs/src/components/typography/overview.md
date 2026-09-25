# Typography

Typography gives Beverly interfaces a readable hierarchy. It defines how titles, body copy, labels, captions, and other text roles differ while keeping their font, scale, and color connected to the active theme.

The typography system has three responsibilities:

- establish hierarchy so users can tell what matters first
- keep text readable across cards, panels, forms, and app shells
- provide theme-driven defaults with deliberate per-instance overrides when needed

```text
Typography
├── Titles
│   └── Mark the hierarchy of screens and sections
├── Text roles
│   └── Describe body, label, caption, and status content
└── Theme
	└── Supplies shared family, size, and color defaults
```

## Choosing a text primitive

Use a title when the text names or introduces a screen, panel, card, or section. Use a text role when the content explains, labels, annotates, or reports status within that structure.

Choose a semantic role before reaching for a manual size or color. Roles preserve a consistent hierarchy when the theme changes, while one-off overrides should be reserved for content that genuinely needs different emphasis.

## Theme relationship

Titles and themed text read their defaults from the active theme. This keeps a font or scale change global and predictable: updating the theme can restyle an entire application without editing every component that contains text.

The View chooses the semantic primitive and content. The theme supplies the visual defaults. Application state and business rules remain outside the typography layer.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_header(mut commands: Commands) {
	commands.spawn((
		Text::new("Workspace"),
		ThemedTitle::new(TitleLevel::H1),
	));
	commands.spawn((
		Text::new("Last synced 2 minutes ago"),
		ThemedText::new(TextRole::Caption),
	));
}
```

## Guidance

- use titles to establish hierarchy, not to style ordinary body copy
- choose the closest semantic text role before overriding size or color
- keep titles short enough to remain scannable at narrow widths
- let the active theme control shared typography defaults
- verify contrast and legibility whenever applying a manual color override
