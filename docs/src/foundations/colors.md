# Colors

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Color in Beverly is composed around semantic roles rather than raw RGB values.

## One source of truth

The app should not hand-pick colors in each widget or screen. Instead, Beverly keeps colors in a single theme definition and lets components read from that theme.

That means a dashboard, sidebar, card, alert, button, and form control all inherit the same palette from the root app theme instead of each one inventing its own colors.

This is the central idea behind the design-token system:

- one theme defines the visual language
- semantic color roles describe intent
- components consume those roles instead of hard-coded hex values

If the brand accent changes, or a dark mode palette is updated, the change happens once in the theme and flows through the whole app.

## Palette model

The color palette is organized around semantic roles:

- base
- surface
- accent
- success
- warning
- danger

These names describe meaning, not literal colors. For example, `accent` is the main highlight color for actions and emphasis, while `surface` is the background of panels and cards. A success state uses the `success` token instead of a custom green sprinkled across the app.

## Example of how it looks

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_shell(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyAppShell::new(),
        BeverlyTheme::dark(), // one theme controls all colors in the app
    )).with_children(|parent| {
        parent.spawn(NodeBundle::default());
        parent.spawn(button().text("Deploy"));
        parent.spawn(BeverlyAlert::success("Sync complete"));
    });
}
```

The important part is that the theme sits at the root of the UI tree. Every child component derives its foreground, background, border, and interaction color from that shared theme.

## Conceptual theme structure

This is the idea in simplified form:

```rust
// Conceptual example; exact API may vary by implementation.
let theme = Theme {
    colors: {
        base: "#0b1120",
        surface: "#111827",
        accent: "#7c3aed",
        success: "#22c55e",
        warning: "#f59e0b",
        danger: "#ef4444",
    },
};
```

In other words, the whole application has a single palette source, and all visual components inherit from it. That keeps the UI cohesive, easier to theme, and much easier to maintain over time.

## Usage

Use semantic color roles for states instead of direct color literals in app code.

That keeps components consistent across light, dark, and custom branded themes while preserving a clear accessibility story.

## Checking WCAG Contrast

For solid colors, `beverly::theme` exposes `relative_luminance`,
`contrast_ratio`, and `meets_contrast`, along with WCAG 2.2 threshold
constants. Use these helpers to check text and meaningful UI indicators:

```rust
use bevy::prelude::Color;
use beverly::theme::{
    contrast_ratio, meets_contrast, WCAG_AA_NORMAL_TEXT, WCAG_AA_UI_COMPONENT,
};

let foreground = Color::WHITE;
let background = Color::srgb(0.12, 0.18, 0.24);

let ratio = contrast_ratio(foreground, background);
let readable_text = meets_contrast(foreground, background, WCAG_AA_NORMAL_TEXT);
let clear_control = meets_contrast(foreground, background, WCAG_AA_UI_COMPONENT);
```

The thresholds are `4.5:1` for normal text at AA, `3:1` for large text at AA,
`7:1` and `4.5:1` for normal and large text at AAA, and `3:1` for meaningful
non-text UI components at AA. The helpers treat both inputs as opaque solid
colors. For gradients, translucent surfaces, or blur, first determine the
effective background that needs to meet the threshold; checking only one
nominal token cannot guarantee contrast over changing content.
