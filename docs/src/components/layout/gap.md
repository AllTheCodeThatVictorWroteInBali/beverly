# Gap

Gap creates consistent space between siblings in a flex or grid layout. It expresses rhythm at the parent level, which keeps related children evenly separated without requiring each child to know about its neighbors.

## When to use

Use gap for repeated spacing between items that share a layout context:

- fields in a form
- actions in a toolbar
- cards in a grid
- sections in a vertical content flow

Use margin when the space belongs to one component's relationship with something outside its own group. Use padding when the space belongs inside a component boundary.

## Row and column gaps

For a horizontal flow, column gap separates neighboring items. For a vertical flow, row gap separates items from top to bottom. In a grid, both directions may need independent values.

```text
Parent group
├── Child A
│   < gap >
├── Child B
│   < gap >
└── Child C
```

## Example

```rust
Node {
	display: Display::Flex,
	flex_direction: FlexDirection::Column,
	row_gap: Val::Px(12.0),
	..default()
}
```

## Guidance

- choose a shared spacing token or value for siblings at the same level
- use larger gaps to separate independent sections
- use smaller gaps for tightly related labels, values, or controls
- avoid mixing child margins and parent gaps unless the distinction is deliberate
