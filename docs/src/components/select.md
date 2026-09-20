# Select

Selects let users choose from a curated set of options when the list is longer than a simple toggle or radio group.

## Example

```rust
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
