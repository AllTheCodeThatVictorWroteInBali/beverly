# Margin

Margin creates space outside a component's own boundary. It describes how one layout item relates to surrounding content, making it useful for separating sections or creating a deliberate offset within a larger composition.

## When to use

Use margin when the space belongs to the component's relationship with its parent or neighboring regions:

- separating a heading from the section above it
- creating page-level or panel-level breathing room
- placing a component at a deliberate outer edge
- reserving space around a standalone surface

Use gap for consistent spacing between siblings in one group. Use padding for space inside the component's boundary.

```text
parent boundary
	< margin > [ component boundary ] < margin >
```

## Example

```rust
Node {
		margin: UiRect {
				top: Val::Px(24.0),
				right: Val::Px(0.0),
				bottom: Val::Px(0.0),
				left: Val::Px(0.0),
		},
		..default()
}
```

## Guidance

- use margin to define a component's outer relationship
- prefer parent-level gap when all siblings share the same rhythm
- avoid using arbitrary margins to repair a broken alignment rule
- keep page-level margins consistent across related views
