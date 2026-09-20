# File Input

File inputs allow the user to select local assets or documents from the operating system file picker.

## Example

```rust
fn build_upload(mut commands: Commands) {
    commands.spawn(BeverlyFileInput::new("Upload asset")
        .accept("image/*"));
}
```

## Recommended behavior

- describe allowed file types clearly
- show a label or selected file summary
- support drag and drop where appropriate
- keep file input actions accessible via keyboard
