# Divider

Dividers create visual separation between content groups or sections.

## Example

```rust
fn build_section(mut commands: Commands) {
    commands.spawn(BeverlyDivider::horizontal());
}
```

## Usage guidance

Use dividers sparingly. They are most effective for separating different content scopes rather than every row or paragraph.
