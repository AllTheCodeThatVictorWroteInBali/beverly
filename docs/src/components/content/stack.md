# Stack

Stacks sequence related content with consistent spacing. They are the simplest way to create a predictable flow when the order of children matters more than a specialized alignment relationship.

A stack can run vertically or horizontally. The direction determines the reading flow; the spacing gives each child enough separation to remain distinct without breaking the group into unrelated pieces.

## When to use

Use a stack when the interface should feel linear and easy to follow. Stacks work well for:

- forms and settings sections
- headings followed by supporting content
- lists of related items
- action groups and compact controls
- cards or panels with a consistent internal rhythm

Use a row when the horizontal relationship between specific children is the important part. Use a column when you need to describe a vertical hierarchy. Use a grid when repeated items need alignment across both axes.

## Composition model

A stack should contain children that belong to the same sequence. It provides the rhythm between those children, while each child remains responsible for its own meaning and interaction.

```text
Stack
├── First item
├── Second item
└── Third item
```

Choose vertical flow for reading, completion, and narrative order. Choose horizontal flow for short groups of controls or values that users should scan as one line. When a horizontal group becomes too long or wraps unpredictably, move it into a vertical stack or a more structured layout.

## Example structure

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_project_setup(mut commands: Commands) {
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

The stack controls the order and spacing of the setup flow. It does not own the workspace value or the create operation. The Model owns the data, while the View presents the form and emits the user's intent through the application's event and controller flow.

## Guidance

- keep all children in a stack related by sequence or purpose
- use one spacing rhythm for items at the same level
- prefer vertical stacks for content that must be read or completed in order
- keep horizontal stacks short enough to scan without wrapping
- use nested stacks sparingly and give each one a clear responsibility
