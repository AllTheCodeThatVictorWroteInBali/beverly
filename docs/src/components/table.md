# Table

Tables are best for structured, comparable data. They work particularly well in admin, metrics, and audit-heavy interfaces.

## Example

```rust
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
