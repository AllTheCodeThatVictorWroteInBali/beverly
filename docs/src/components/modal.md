# Modal

Modals interrupt the current flow to request a decision or present a focused task.

## Example

```rust
fn open_delete_modal(mut commands: Commands) {
    commands.spawn(BeverlyModal::new("Delete item")
        .body("This action cannot be undone.")
        .confirm_label("Delete")
        .cancel_label("Cancel"));
}
```

## Usage guidance

Use modals sparingly. They are best for critical decisions or short focused operations. Keep the content narrow and the action labels explicit.

## Accessibility

Trap focus while the modal is open, restore focus when it closes, and maintain clear labels for close and action controls.
