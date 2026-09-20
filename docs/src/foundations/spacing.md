# Spacing

Beverly uses a consistent spacing scale to keep layout rhythm predictable.

Spacing is one of the main pieces that makes a UI feel cohesive. Instead of inventing margins and padding ad hoc in every component, Beverly uses a small shared scale so cards, forms, sidebars, and page layouts all feel like they belong to the same system.

The scale is used anywhere the app needs distance between elements:

- padding inside surfaces
- gaps between items in a stack or grid
- margins around sections
- inset spacing for labels, controls, and content blocks

The important idea is consistency. A `md` gap should mean the same visual rhythm whether it appears inside a card, between toolbar buttons, or between sections on a page.

## Scale
- xs
- sm
- md
- lg
- xl
- 2xl

Use the smaller steps for dense content and compact controls. Use the larger steps for section breaks, major layout regions, and places where the interface needs breathing room.

## How it works

Think of the spacing scale as a shared vocabulary rather than a set of arbitrary pixel values. Components and layouts ask for a spacing token, and the theme or design system resolves that token into the actual distance.

That gives you three benefits:

- layouts stay predictable across the app
- components can be composed without constantly re-tuning spacing
- visual density can be adjusted globally by changing the scale implementation instead of editing every screen

## Example: page section spacing

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_page(mut commands: Commands) {
	commands.spawn(BeverlyStack::vertical())
		.with_children(|parent| {
			parent.spawn(BeverlyHeading::new("Overview"));
			parent.spawn(BeverlyText::body("Key metrics and activity appear below."));
			parent.spawn(BeverlyGrid::new().gap(Spacing::lg));
		});
}
```

Here, the larger spacing separates the title block from the content area, while the content itself can use tighter spacing inside the grid.

## Example: card content

```rust
fn build_card(mut commands: Commands) {
	commands.spawn(BeverlyCard::new("Deployment status")).with_children(|card| {
		card.spawn(BeverlyText::new("All services are healthy."));
		card.spawn(BeverlyText::new("Last updated 2 minutes ago"));
	});
}
```

In a card, spacing usually controls the inset around the content and the vertical distance between the title, body text, and footer details.

## Example: dense controls

```rust
fn build_toolbar(mut commands: Commands) {
	commands.spawn(BeverlyStack::horizontal())
		.with_gap(Spacing::sm)
		.with_children(|toolbar| {
			toolbar.spawn(BeverlyButton::new("Refresh"));
			toolbar.spawn(BeverlyButton::new("Export"));
			toolbar.spawn(BeverlyButton::new("Share"));
		});
}
```

Compact controls work best with tighter spacing so the toolbar stays efficient without feeling crowded.
