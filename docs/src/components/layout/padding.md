# Padding

Padding creates space inside a component's boundary, between that boundary and its children. It gives content room to breathe and makes the extent of a surface clear without changing the relationships between sibling components.

## When to use

Use padding for:

- card and panel insets
- control touch and interaction areas
- separating text from a surface edge
- creating a stable content region inside a container

Use gap to separate siblings. Use margin to separate the component from content outside its boundary.

```text
[ component boundary ]
	< padding > content < padding >
[ component boundary ]
```

## Example

```rust
Node {
		padding: UiRect::axes(
				Val::Px(20.0),
				Val::Px(16.0),
		),
		..default()
}
```

## Guidance

- keep padding consistent across surfaces that share a visual role
- use larger insets for major panels and smaller insets for compact controls
- make sure padding does not reduce the usable space of dense content too far
- treat padding as part of the component's boundary, not as a child-specific fix
