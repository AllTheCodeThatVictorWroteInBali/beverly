# Pagination

<img src="../assets/pagination.svg" alt="Pagination component illustration" width="860" />

Pagination helps users move through large result sets or multi-page collections. It is valuable when the data set is too large to display in a single pass but still benefits from clear page boundaries.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_pager(mut commands: Commands) {
    commands.spawn(BeverlyPagination::new(1, 12)
        .current_page(4));
}
```

## Use when

- a list is too long for a single view
- the user needs to scan through a collection in discrete pages
- content has reasonable page boundaries
- the app benefits from preserving context and avoiding endless scrolling

## Guidance

Use pagination to group content into manageable chunks rather than hiding too much information behind a complicated pager. Keep the current page, total page count, and next/previous actions obvious so the user always knows where they are.

## Accessibility

The pager should support keyboard interaction and convey state clearly. Users should not need to infer the current page from color alone or from a hidden label that only appears on hover.
