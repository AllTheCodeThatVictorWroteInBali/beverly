# Divider

<img src="../assets/divider.svg" alt="Divider component illustration" width="860" />

Dividers create visual separation between content groups or sections. They help the eye understand where one scope ends and another begins without adding extra visual weight.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_section(mut commands: Commands) {
    commands.spawn(BeverlyDivider::horizontal());
}
```

## Usage guidance

Use dividers sparingly. They are most effective for separating different content scopes rather than every row or paragraph. Overuse produces noise, weakens hierarchy, and makes the interface feel visually crowded.

## Accessibility

Dividers are generally decorative, so they should not carry critical meaning. Important structure should still come from headings, labels, and semantic grouping, not only from a line across the screen.
