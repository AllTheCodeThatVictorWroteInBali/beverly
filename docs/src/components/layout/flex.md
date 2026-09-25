# Flex

Flex layout distributes children along a direction and lets the available space determine how that group behaves. It is the foundation for rows, columns, toolbars, forms, and other compositions whose children have a meaningful order.

## When to use

Use flex layout when content should flow as a group rather than occupy unrelated absolute positions. It works well for:

- navigation and action bars
- form sections
- split content regions
- reusable component shells
- layouts that need to grow with their children

## Direction and growth

Set the direction first. A row creates a horizontal flow; a column creates a vertical flow. Then decide which children may grow, shrink, or keep their intrinsic size.

```text
Flex container
├── Child A  -> fixed or intrinsic size
├── Child B  -> may grow into remaining space
└── Child C  -> fixed or intrinsic size
```

Keep growth intentional. A flexible content region can absorb remaining space, while labels and actions should usually retain enough room to stay recognizable.

## Example

```rust
Node {
	display: Display::Flex,
	flex_direction: FlexDirection::Row,
	align_items: AlignItems::Center,
	column_gap: Val::Px(8.0),
	..default()
}
```

## Guidance

- use flex for relationships that should survive content changes
- define direction before tuning spacing or alignment
- allow the content area to grow only when that behavior is useful
- avoid deeply nested flex containers when one parent rule is enough
- use a grid when both axes need repeated alignment
