# Column

Columns arrange related content along a vertical axis. They establish a reading order from top to bottom, making them the natural structure for forms, detail panels, settings sections, and other content that should unfold progressively.

## When to use

Use a column when vertical order carries meaning or when each item needs room to be understood before the next one appears. Columns work well for:

- page and panel sections
- forms with labels, fields, and validation messages
- titles followed by supporting content
- detail views with metadata and actions
- stacked controls that should remain easy to scan

Use a row when related items belong on one horizontal line. Use a stack when the main concern is consistent spacing between a sequence of children. Use a grid when repeated content needs alignment across multiple columns.

## Composition model

A column should follow the way a user reads or completes the content. Put the most important context first, keep supporting information close to the content it explains, and place actions where they naturally follow from the preceding information.

```text
Column
├── Heading
├── Supporting content
├── Primary content
└── Actions
```

Each child keeps its own semantic role. The column provides the shared order, width, and spacing that lets those roles read as one coherent section.

## Example structure

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_settings_column(mut commands: Commands) {
	commands.spawn((
		NodeBundle::default(),
		BeverlyStack::vertical(),
	)).with_children(|column| {
		column.spawn(BeverlyTitle::new("Workspace settings"));
		column.spawn(BeverlyText::new("Manage the defaults for this workspace."));
		column.spawn(BeverlyInput::new("Workspace name"));
		column.spawn(BeverlyButton::primary("Save changes"));
	});
}
```

The column shapes the View, but it does not own the settings or the save operation. The Model remains the source of truth, and an event or controller action can handle the user's intent when the button is pressed.

## Guidance

- order children according to the user's reading or completion flow
- keep related labels, content, and actions close together
- use consistent widths when several columns appear in the same view
- give dense sections enough vertical spacing to preserve hierarchy
- move to a row only when items are genuinely related on the same horizontal line
