# Alignment

Alignment determines how children are positioned inside the space made available by their parent. It is the layout decision that turns a group of elements into a readable relationship: centered actions, leading labels, or controls aligned to a common edge.

## When to use

Use alignment to establish a shared visual edge or axis. Common patterns include:

- leading-aligning labels and body content
- centering a compact action or status group
- stretching fields to the width of a form
- aligning toolbar items along their cross axis

Alignment should describe the relationship of a group, not compensate for inconsistent child sizes.

## Main and cross axes

Flex direction determines the main axis. In a row, children flow horizontally; in a column, they flow vertically. Alignment then controls placement along the main axis and the cross axis.

```text
Column flow
main axis:  top -> bottom
cross axis: left <-> right

Row flow
main axis:  left -> right
cross axis: top <-> bottom
```

Choose alignment based on the content's reading path. A form usually stretches fields across its cross axis, while a compact toolbar often centers controls vertically.

## Example

```rust
Node {
	display: Display::Flex,
	flex_direction: FlexDirection::Column,
	align_items: AlignItems::Stretch,
	..default()
}
```

## Guidance

- align related children to a shared edge
- use center alignment for compact groups, not long reading content
- let the parent establish alignment instead of adding offsets to each child
- verify alignment at narrow and wide viewport sizes
