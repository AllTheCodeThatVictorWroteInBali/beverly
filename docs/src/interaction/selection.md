# Selection

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Selection models describe how users pick content or options inside a view. They are one of the most important interaction patterns in data-heavy interfaces because they connect the user’s intention to the underlying app state. Selection is not only about highlighting a row or item; it is about representing choice in a way the user can understand and operate consistently.

## Why selection matters

Selection is the mechanism that turns one object in a view into a meaningful current choice. It is how users say:

- this item is the one I want to inspect
- this row is active
- this option is currently chosen
- this action applies to this target

Without a clear selection model, interfaces become ambiguous or frustrating, especially in tables, lists, forms, and multi-panel workspaces.

## Common selection patterns

Different interfaces need different selection models, but they should still be predictable:

- single selection for choosing one option from a set
- multi-selection for choosing several items at once
- range selection for contiguous groups in tables or lists
- active selection for the current focus or context object
- contextual selection for temporary highlight during inspection or drag states

The important thing is that the selection pattern clearly matches the UI’s purpose and remains stable as the user interacts.

## Interaction behavior

A selection model should define how the user changes selection, how the app displays it visually, and whether the user can clear or modify the selection. It should also integrate with keyboard navigation, focus behavior, and pointer interaction.

Examples:

- click selects a single item
- Ctrl or Command click adds to a multi-selection set
- arrow keys move the active selection within a list
- range selection is maintained clearly when the selection is contiguous or grouped

The interface should not leave the user guessing whether the current highlight is merely hover, focus, or a real selection state.

## Selection and state clarity

Selection should be legible. The user should be able to distinguish between:

- hover state
- focus state
- active selection state
- disabled or unavailable state

When these states are visually conflated, the interface becomes hard to understand. A row may appear selected when it is only focused, or a control may look active when it is not actually chosen.

This is why selection should be modeled as a first-class interaction state, not as a vague visual effect.

## Selection in dense interfaces

Selection is especially important in tables, virtualized lists, and data-heavy UIs where many items may be on screen at once. In those cases, the interface must maintain consistent mapping between the user’s visible representation and the underlying data model.

A strong selection model should support:

- stable row identity even during scrolling or virtualization
- keyboard navigation without losing the selected target
- clear focus restoration when content updates
- reliable interactions for bulk actions, sorting, and filtering

## Guidance

Selection should make the user’s current choice clear without introducing guesswork. A well-designed selection model keeps the interface expressive, predictable, and easy to operate, even in dense or dynamic data views.
