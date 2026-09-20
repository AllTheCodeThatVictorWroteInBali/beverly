# Text

Text is the default body-content primitive in Beverly. It is used for descriptions, helper copy, metadata, and any other supporting content.

## When to use

Use text when the content should read as part of the surrounding surface instead of as a headline or control.

## Implementation pattern

Block Studio-style composition keeps text as a lightweight semantic block that inherits the app's typography rules. That keeps copy consistent across cards, panels, headers, and other surfaces without creating a separate text system for each screen.

## Example

```rust
fn build_copy(mut commands: Commands) {
    commands.spawn(BeverlyText::new("Last synced 2 minutes ago"));
}
```

## Guidance

- use text for supporting copy and short explanations
- keep it legible in dense layouts
- let the surrounding layout control spacing and alignment