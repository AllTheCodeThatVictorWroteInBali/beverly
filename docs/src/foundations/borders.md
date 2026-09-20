# Borders

Borders define structure, separation, and interaction emphasis. They provide a clean visual edge without needing to add heavy fills or extra decoration.

In Beverly, borders are not just decorative lines. They carry meaning: a neutral border can separate surfaces, a stronger border can signal focus or active state, and a high-contrast outline can call attention to an interactive control.

## Border roles

Use borders according to their intent:

- subtle separators for low-importance boundaries
- focus rings for keyboard or pointer emphasis
- emphasis borders for selected or highlighted content
- input outlines for form controls and editable fields

The rule is simple: the border should support the content, not compete with it. A dense dashboard can use subtle edges almost everywhere, while an active state or critical panel can move to a stronger visual treatment.

## Scale

Border weight should be consistent across the app. A thin stroke works for separators and form outlines, while a heavier stroke works for focus or selected surfaces.

A useful scale is:

- thin
- regular
- strong
- focus

Thin borders keep layouts clean and calm. Stronger borders appear when the UI needs to show intent, focus, or hierarchy.

## When to use each one

### Subtle separators

Use subtle borders to separate areas that belong together but still need a visual edge. This is common in cards, sidebars, tables, and stacked panels.

### Focus rings

Use focus rings when a control is active, selected, or keyboard-focused. The border should be obvious enough to stand out but still remain part of the surrounding layout.

### Emphasis borders

Use emphasis borders when a panel, table row, or card needs to feel important without turning into a large highlighted block. This is a good way to show selection or state without overusing color alone.

### Input outlines

Inputs are one of the most common cases for borders. A standard outline communicates editability and structure, while a stronger outline signals validation, active state, or keyboard focus.

## Example: form field

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_form(mut commands: Commands) {
    commands.spawn(BeverlyCard::new("Project details")).with_children(|card| {
        card.spawn(BeverlyInput::new("Project name"));
        card.spawn(BeverlyInput::new("Owner"));
        card.spawn(BeverlyButton::new("Save changes"));
    });
}
```

In a form, the border is doing a quiet but important job: it defines the control boundary, keeps the field readable, and reinforces focus when the user interacts with it.

## Example: selection state

```rust
fn build_dashboard(mut commands: Commands) {
    commands.spawn(BeverlyList::new()).with_children(|list| {
        list.spawn(BeverlyListItem::new("Overview").selected());
        list.spawn(BeverlyListItem::new("Deployments"));
        list.spawn(BeverlyListItem::new("Activity"));
    });
}
```

A selected item can use a stronger border or a more visible edge treatment to signal that it is the current context. This keeps the selection easy to notice without forcing the whole row to become a loud accent block.

## Theme consistency

Border color should come from the active theme, not from arbitrary hard-coded values. That means the same border semantics can hold across light mode, dark mode, and custom branded variants.

A strong border in one theme should feel equivalent to a strong border in another, even if the specific color values differ. The semantics remain stable; the implementation adapts to the palette.

## Rule of thumb

Keep borders intentional and restrained:

- separate when needed
- emphasize when the state matters
- focus when the user is interacting
- avoid using borders as decoration alone

When border treatment is consistent, the interface looks more cohesive and feels easier to scan.
