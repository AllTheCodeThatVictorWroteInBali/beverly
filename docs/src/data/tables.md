# Tables

Tables help users compare structured information across rows and columns. They are especially valuable in operational, analytical, and audit-heavy interfaces where users need to reason across many values at once.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_table(mut commands: Commands) {
    commands.spawn(BeverlyTable::new()
        .column("Name")
        .column("Status")
        .column("Updated")
        .row("Deploy API", "Healthy", "2 min ago")
        .row("Sync workers", "Warning", "6 min ago"));
}
```

## Guidance

Use tables when comparison matters more than narrative flow. Maintain strong headers, clear alignment, and limited visual noise. Dense tables are most useful when the user needs to analyze and act, not simply browse.
