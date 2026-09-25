# Layout

Layout components control how content occupies space inside a Beverly view. They define alignment, direction, spacing, sizing, and adaptation so that a composition remains readable as its content and viewport change.

Layout is a presentation responsibility. It should make relationships visible and predictable without taking ownership of application state or business rules.

## Layout model

Use the smallest primitive that expresses the relationship you need:

- alignment positions children within their available space
- flex direction determines whether content flows by row or column
- gap creates consistent space between siblings
- margin controls space outside a component's own boundary
- padding controls space between a boundary and its content
- min and max constraints keep dimensions within useful limits
- width and height define the intended size of a region
- responsive rules let the composition adapt when available space changes

```text
Layout
├── Boundary
│   ├── Width / Height
│   └── Min / Max
├── Flow
│   └── Flex direction
├── Rhythm
│   ├── Gap
│   ├── Margin
│   └── Padding
└── Placement
	└── Alignment
```

## Composition guidance

Start with structure, then add spacing, then tune sizing. A stable layout usually needs only a few deliberate decisions: which direction content flows, how children align, how much space separates them, and what should happen when the available space is smaller or larger than expected.

Prefer shared layout rules over per-child offsets. This keeps a screen easier to scan and prevents one component from quietly compensating for another component's incorrect sizing.

## Example

```rust
use bevy::prelude::*;

fn build_panel(mut commands: Commands) {
	commands.spawn((
		Node {
			display: Display::Flex,
			flex_direction: FlexDirection::Column,
			align_items: AlignItems::Stretch,
			row_gap: Val::Px(12.0),
			padding: UiRect::all(Val::Px(16.0)),
			width: Val::Percent(100.0),
			..default()
		},
	));
}
```

The example establishes a vertical flow, a shared rhythm, an inset boundary, and a width that can participate in its parent's layout. The content components inside the panel can then focus on their own meaning.
