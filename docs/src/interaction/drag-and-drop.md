# Drag and Drop

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Drag-and-drop patterns support reordering and transfers across surfaces. They are useful when the user needs to move content between areas, reorder items, or adjust layout in a direct, tactile way. But like any gesture-heavy interaction, drag-and-drop should remain clear, intentional, and easy to recover from.

## Why drag-and-drop is useful

Direct manipulation is powerful because it matches the user’s mental model. Instead of navigating through menus or form fields, the user can move a thing to where it belongs. This works especially well for:

- list reordering
- file organization
- panel or card grouping
- transferring items between columns or regions
- adjusting data layout or sorting order

A good drag-and-drop interaction feels immediate and understandable because the user can see what is moving and where it will land.

## The core requirements

A drag-and-drop system should provide:

- an obvious drag target
- a visible pointer or movement state while dragging
- clear drop targets or landing zones
- feedback about valid vs invalid drops
- cancellation or rollback behavior when the drag is abandoned

If the app does not clearly communicate these states, the interaction becomes frustrating and unpredictable.

## Feedback and state transitions

A good drag system should tell the user what is happening at each stage:

- the item is being dragged
- the current drop target is valid
- the drop has succeeded or failed
- the item is being moved or re-ordered in the underlying data

This feedback can come through opacity changes, highlight states, ghost outlines, or a temporary insertion indicator. The important thing is that the state is visible and understandable while the drag is active.

## Safety and reversibility

Because drag operations often have meaningful consequences, they should be designed with safety in mind. A user should not accidentally reorder or transfer content without seeing the result clearly.

This is especially important for:

- destructive or irreversible actions
- bulk operations on many items
- reordering in dense lists or tables
- cross-surface transfers where the destination is not obvious

If a drop is invalid, the system should make that clear without ambiguity. If a drag is canceled, the interface should return to its previous state smoothly.

## Drag and drop in a structured interface

A drag system should fit naturally into the rest of the interaction model, not create an isolated special case. That means it should align with:

- pointer behavior
- hover and press feedback
- focus and selection patterns
- keyboard alternatives where possible

For example, a drag-enabled list should still allow keyboard navigation and maybe keyboard reordering or selection updates when pointer drag is not available.

## Guidance

Drag-and-drop is powerful because it mirrors the user’s intention directly, but it should be structured, visible, and recoverable. When the system behaves clearly during the drag, the user can trust the interface and move through it with confidence.
