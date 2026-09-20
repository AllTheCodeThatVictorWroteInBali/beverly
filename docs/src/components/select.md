# Select

Selects let users choose from a curated set of options when the list is longer than a simple toggle or radio group. They are ideal for configuration surfaces where a compact control needs to show a single chosen option while still supporting many possible values.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_select(mut commands: Commands) {
    commands.spawn(BeverlySelect::new("Theme")
        .options(["System", "Light", "Dark", "High contrast"])
        .selected("Dark"));
}
```

## Usability guidance

- keep option labels concise and understandable
- use sorted values when appropriate
- allow keyboard navigation for long lists
- prefer a clear default when the choice is not obvious
- avoid overusing select controls for short binary choices where a toggle or radio group is clearer

## Accessibility

Select fields should have a visible label and support keyboard opening and navigation. The selected value should be discoverable and should not depend on hover-state or color alone.
