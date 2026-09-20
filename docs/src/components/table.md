# Table

Tables are best for structured, comparable data. They work particularly well in admin, metrics, and audit-heavy interfaces where users need to compare values across rows and columns.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_table(mut commands: Commands) {
    commands.spawn(BeverlyTable::new()
        .column("Name")
        .column("Status")
        .column("Updated"));
}
```

## Recommended patterns

- keep columns limited and meaningful
- support sorting where useful
- use compact row density for dense datasets
- maintain clear row and header alignment
- use strong contrast between header and body content

## Accessibility

Tables need clear headers, consistent row structure, and understandable sorting or selection states. Data density should not hide meaning or create an unreadable scan path for keyboard or screen-reader users.
