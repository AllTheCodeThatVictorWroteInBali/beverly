# Typography

Typography in Beverly is designed to support clear reading, strong hierarchy, and high readability in dense dashboards and AI-native tools.

## Global font

Set one font at the application root so every text style inherits the same family. That keeps the app visually consistent and makes brand updates easy to apply in one place.

In practice, you define the font once in your theme or app shell, then let titles, labels, and body text read from that shared setting.

```rust
// Conceptual example; exact API may vary by implementation.
let theme = Theme {
	typography: Typography {
		font_family: "Inter",
		..default()
	},
	..default()
};
```

## [Title](#title)

Use title text for the main message of a screen, card, or panel. Titles should be short, prominent, and easy to scan.

Titles usually sit above supporting copy and should carry the strongest size and weight in the local hierarchy.

## [Subtitle](#subtitle)

Use subtitle text for secondary context that supports the title without competing with it. Subtitles are useful for short descriptions, metadata, or explanatory lines.

Subtitles should remain visually lighter than the title while still being readable at a glance.

## Adding text

Text in Beverly works like the rest of the UI: you add it to the app tree as part of a parent layout, and it inherits the active theme and typography rules.

That means you can place text inside headers, cards, sidebars, or any other surface without inventing a separate text system for each screen. The title and subtitle styles are just two common ways to apply that same text pipeline.

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_screen(mut commands: Commands) {
	commands.spawn(NodeBundle::default()).with_children(|parent| {
		parent.spawn(BeverlyText::title("Workspace"));
		parent.spawn(BeverlyText::subtitle("Last synced 2 minutes ago"));
		parent.spawn(BeverlyText::body("Everything below this point uses the same global font."));
	});
}
```

## Text colors

Text color should come from the active theme instead of being picked ad hoc in each widget. Use the default foreground color for normal copy, a muted foreground for supporting text, and a stronger emphasis color only when the text itself carries interactive or semantic weight.

This keeps titles, subtitles, and body copy readable across light, dark, and branded themes. It also ensures text still contrasts correctly when the app switches to high-contrast or reduced-contrast presentations.

In other words, typography decides the structure of the text, while the theme decides the actual color values that flow through it.

## Changing size

Change text size by choosing a different type scale or by overriding the size on a specific text element when a screen needs emphasis.

Use the larger styles for page-level headers and the smaller styles for dense metadata, labels, and helper copy. Keep size changes deliberate so the hierarchy stays readable.

## Scale

- Display
- Heading
- Body
- Label
- Caption

The scale gives you a predictable set of steps instead of arbitrary one-off font sizes. That makes it easier to keep the interface balanced when you resize text for different surfaces or screen densities.
