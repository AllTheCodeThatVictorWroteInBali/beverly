# Pagination

Pagination helps users move through large result sets or multi-page collections.

## Example

```rust
fn build_pager(mut commands: Commands) {
    commands.spawn(BeverlyPagination::new(1, 12)
        .current_page(4));
}
```

## Use when

- a list is too long for a single view
- the user needs to scan through a collection in discrete pages
- content has reasonable page boundaries
