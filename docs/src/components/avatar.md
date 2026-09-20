# Avatar

Avatars represent a person, team, or entity in a compact, recognizable form. They add identity to a list item, workspace member, or conversation without taking much space.

## When to use

Use an avatar for:

- user or team lists
- message threads and activity feeds
- comment authorship
- presence and ownership metadata
- compact account selection surfaces

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_member_row(mut commands: Commands) {
    commands.spawn(BeverlyAvatar::new("VW")
        .size(AvatarSize::Medium));
}
```

## Guidance

Keep the visual treatment consistent across the app. The avatar should support the primary content, not compete with it. When the user is not identifiable by photo, initials or a short label are often more effective than a large decorative asset.

## Accessibility

Give the avatar a meaningful label or name when it communicates user identity. If the avatar is part of a control, the accessible name should still come from the item or label surrounding it, not only from a decorative visual.
