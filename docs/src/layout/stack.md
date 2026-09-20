# Stack

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Stacks provide vertical or horizontal grouping with spacing between elements. They are the simplest and most flexible way to compose sequences of controls, text, and content blocks in a predictable way.

## When to use

Use a stack when the interface should feel linear and easy to follow: forms, action groups, theorem lists, cards, or command rows all work well in stacks.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_stack(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyStack::vertical(),
    )).with_children(|stack| {
        stack.spawn(BeverlyTitle::new("Project setup"));
        stack.spawn(BeverlyInput::new("Workspace name"));
        stack.spawn(BeverlyButton::primary("Create workspace"));
    });
}
```

## Guidance

Use stacks to create natural reading flow. Vertical stacks are best for forms and narrative sections; horizontal stacks are best for short control groups such as actions or chips that belong together visually.
