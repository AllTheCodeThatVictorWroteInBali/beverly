# Responsive Layout

Responsive layout adapts composition when the available space changes. It is not only about shrinking dimensions; it is about preserving hierarchy, readability, and usable interaction across viewport sizes.

## When to use

Use responsive rules when a composition may appear in different windows, panels, or device sizes:

- dashboards that move from multi-column to single-column content
- toolbars whose actions need a narrower arrangement
- forms that change from side-by-side fields to a vertical flow
- panels whose width should remain readable instead of filling all space

## Adapt the structure first

When space becomes constrained, change the relationship between components before reducing their legibility. A row may become a column, a grid may reduce its column count, or secondary content may move below the primary content.

```text
Wide space:   [ navigation ] [ main content ] [ details ]
Narrow space: [ navigation ]
			  [ main content ]
			  [ details ]
```

Use width, height, and min/max constraints to provide stable bounds. Use flex direction, wrapping, and alignment to express how the composition should respond inside those bounds.

## Example

```rust
Node {
	width: Val::Percent(100.0),
	max_width: Val::Px(960.0),
	flex_direction: FlexDirection::Column,
	..default()
}
```

The example keeps the region fluid while protecting its maximum reading width. The surrounding composition can decide when related regions should stack or change priority.

## Guidance

- test layouts at the smallest and largest useful sizes
- preserve action visibility and readable text before visual symmetry
- use progressive disclosure for secondary content when space is limited
- avoid relying on one fixed viewport size as the design target
- keep responsive decisions at the parent layout boundary when possible
