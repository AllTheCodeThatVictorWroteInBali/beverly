# Avatar

Avatars represent a person, team, or entity in a compact UI form.

## Example

```rust
fn build_member_row(mut commands: Commands) {
    commands.spawn(BeverlyAvatar::new("VW")
        .size(AvatarSize::Medium));
}
```

## Use cases

- user lists
- team selectors
- comments and notifications
- presence indicators
