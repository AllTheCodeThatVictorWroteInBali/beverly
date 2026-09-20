# Spacing

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Beverly uses a consistent spacing scale to keep layout rhythm predictable. Spacing is not merely a cosmetic detail; it is one of the primary ways the system communicates hierarchy, state, and readability. A well-spaced interface feels calmer, more intentional, and easier to trust because the user can quickly understand which elements belong together and which ones are separate.

## Why spacing matters

Spacing shapes the interface in several important ways:

- it separates groups of content that should be read together from groups that should be read separately
- it makes dense surfaces easier to scan without turning them into a wall of text or controls
- it supports alignment and rhythm across cards, forms, toolbars, and navigation surfaces
- it creates a sense of breathing room that keeps the interface from feeling overloaded

Without a clear scale, every screen risks becoming inconsistent: one card has too much padding, another too little, a toolbar feels cramped, and section boundaries become ambiguous. A shared spacing system keeps those decisions stable and scalable.

## Core principle

Spacing should be semantic and regular, not arbitrary. Beverly uses a small set of spacing tokens so component layout decisions remain consistent across the whole app.

The default scale is usually something like:

- xs
- sm
- md
- lg
- xl
- 2xl

The smaller tokens are used for compact controls, list items, and tight layout groups. The larger tokens are used for major section transitions, page-level rhythm, and surfaces that need more breathing room.

The important idea is that a token such as md should mean the same visual rhythm regardless of where it appears: inside a card, between form fields, between buttons in a toolbar, or between panels on a page.

## How spacing functions in the system

Think of the spacing scale as a shared vocabulary rather than a set of arbitrary pixel values. Components and layouts ask for a spacing token, and the theme or design system resolves that token into the actual distance.

That gives you several benefits:

- layouts stay predictable across the app
- components can be composed without constantly re-tuning spacing
- visual density can be adjusted globally by changing the scale implementation instead of editing every screen
- consistent rhythm helps a product feel mature and designed rather than improvised

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

Here, the larger spacing separates the title block from the content area, while the content itself can use tighter spacing inside the grid. This preserves a strong sense of hierarchy without forcing every surface to have the same density.

## Example: card content

```rust
fn build_card(mut commands: Commands) {
    commands.spawn(BeverlyCard::new("Deployment status")).with_children(|card| {
        card.spawn(BeverlyText::new("All services are healthy."));
        card.spawn(BeverlyText::new("Last updated 2 minutes ago"));
    });
}
```

In a card, spacing usually controls the inset around the content and the vertical distance between the title, body text, and footer details. A card that is too dense may become hard to read; a card with excessive padding may feel disconnected from the rest of the screen.

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

Compact controls work best with tighter spacing so the toolbar stays efficient without feeling crowded. This is an example of using the spacing scale deliberately: the same rules that create hierarchy on a page also support quick action groups in a toolbar.

## Layout strategy

Beverly’s spacing system works best when it is used with the same intention across the product:

- use tighter spacing inside related component groups
- use wider spacing between independent sections
- keep the relationship between neighboring elements clear and intentional
- avoid mixing arbitrary gaps that create an inconsistent rhythm

This keeps the app visually coherent and easier to maintain as more screens are added.

## Summary

Spacing is a foundational design tool because it creates rhythm, hierarchy, and calm. In a product system, the spacing scale is part of the interface’s logic, not just its aesthetic finish. When used consistently, it helps the whole app feel unified and easier to read.
