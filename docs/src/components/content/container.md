# Container

Containers define the boundary of a content region. They give related children a shared width, alignment, and context so that a view can grow without becoming a loose collection of unrelated elements.

## When to use

Use a container when you need to:

- establish the readable boundary of a page or panel
- align a heading, body, and actions to the same content edge
- group a section that has one clear purpose
- keep internal rows, columns, or stacks inside a predictable region

A container is especially useful at the point where page structure becomes local content structure. Use a row or column inside it to describe how that content is arranged.

## Composition model

A container should provide context without hiding the hierarchy of its children. Give it a clear purpose, then let rows, columns, stacks, and individual components express the relationships inside that boundary.

```text
Container
├── Heading
├── Content region
│   ├── Row
│   └── Column
└── Actions
```

The container owns the region's boundary and alignment. Its children remain responsible for their own semantics, presentation, and interaction.

## Example structure

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_workspace_panel(mut commands: Commands) {
	commands.spawn((
		NodeBundle::default(),
		BeverlyContainer::new(),
	)).with_children(|container| {
		container.spawn(BeverlyTitle::new("Workspace"));
		container.spawn(BeverlyText::new("Review the latest activity."));
		container.spawn(BeverlyButton::primary("Open workspace"));
	});
}
```

The container makes these elements read as one panel without deciding what the workspace data means or what happens when the button is pressed. The Model owns the data, and the View presents it within the container's boundary.

## Guidance

- give each container a clear content boundary and purpose
- align related headings, content, and actions to the same edge
- keep internal structure explicit with rows, columns, or stacks
- avoid stretching dense content across the full viewport
- do not use a container as a substitute for semantic grouping or state ownership
