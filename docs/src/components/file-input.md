# File Input

File inputs allow the user to select local assets or documents from the operating system file picker. They are most useful in upload workflows, media management, and content ingestion screens.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

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
- provide a clear empty state and a readable selected-file result

## Accessibility

The file input should have an explicit label and an understandable status area. Users should be able to identify accepted file types and understand what was selected without relying only on visual hints or an unlabelled upload button.
