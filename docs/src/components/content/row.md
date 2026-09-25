# Row

Rows arrange related content along a horizontal axis. They make the relationship between items explicit when those items belong together but should remain distinct: a label and value, a group of actions, or a set of compact controls.

## When to use

Use a row when horizontal order carries meaning or helps users scan a group quickly. Rows work well for:

- headings paired with actions
- labels paired with status or metadata
- short groups of buttons, filters, or navigation items
- compact content that should share one visual line

Use a column when the content needs a vertical reading order. Use a stack when consistent spacing and linear flow matter more than the specific horizontal relationship between items. Use a grid when several items need repeated alignment across both rows and columns.

## Composition model

A row should have one clear purpose and a small number of related children. Each child keeps its own semantic role, while the row provides the shared alignment and spacing that makes the group read as one unit.

```text
Row
├── Label
├── Supporting content
└── Action
```

Prefer stable alignment over decorative variation. Keep the primary content easy to find, place actions consistently, and allow long content to wrap or yield space rather than pushing related controls out of view.

## Example structure

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_status_row(mut commands: Commands) {
	commands.spawn(NodeBundle::default()).with_children(|row| {
		row.spawn(BeverlyText::new("Sync status"));
		row.spawn(Badge::new("Up to date"));
		row.spawn(BeverlyButton::secondary("Details"));
	});
}
```

The row is responsible for the relationship between these children. The Model still owns the status, and the View decides how that state should be presented. Keeping those responsibilities separate makes the row easy to reuse and change.

## Guidance

- keep children related by meaning, not only by proximity
- use consistent alignment when several rows appear in the same view
- reserve the most stable space for the primary label or value
- avoid placing unrelated actions in the same row
- switch to a column when the content becomes difficult to scan horizontally
