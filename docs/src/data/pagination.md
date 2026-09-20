# Pagination

Pagination organizes large result sets into manageable pages. It is a strong pattern when the data should still feel browseable and the user may need to move across many pages deliberately.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_pagination(mut commands: Commands) {
    commands.spawn(BeverlyPagination::new(1, 12)
        .current_page(3));
}
```

## Guidance

Keep the boundaries obvious and the page state visible. Pagination works best when the user can understand current position and move forward or backward without making assumptions.
