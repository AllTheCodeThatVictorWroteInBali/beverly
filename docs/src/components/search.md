# Search

Search controls help users filter or locate items in a data-heavy interface.

## Example

```rust
fn build_search(mut commands: Commands) {
    commands.spawn(BeverlySearch::new("Search files")
        .with_action("Submit"));
}
```

## Guidance

Large interfaces should provide instant filtering feedback, clear empty states, and visible results count where possible.
