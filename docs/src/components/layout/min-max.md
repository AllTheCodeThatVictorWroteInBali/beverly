# Min and Max

Minimum and maximum constraints keep a layout usable across changing content and viewport sizes. They prevent a region from collapsing below a readable size or expanding until its content becomes difficult to scan.

## When to use

Use constraints when a component has a useful range rather than one fixed dimension:

- readable content columns
- sidebars and navigation regions
- cards that should not become excessively wide
- controls that need room for their labels
- panels that must remain usable on narrow screens

## Constraint model

Think of the final size as a bounded range:

```text
minimum size <= resolved size <= maximum size
```

The available space and the component's content determine the resolved size inside that range. Constraints should protect readability, not replace responsive composition.

## Example

```rust
Node {
	width: Val::Percent(100.0),
	min_width: Val::Px(240.0),
	max_width: Val::Px(720.0),
	..default()
}
```

## Guidance

- set a minimum only when the content truly needs protected space
- use maximum widths to preserve readable line lengths
- test constraints with both short and long content
- avoid incompatible constraints that leave no useful resolved size
- combine min and max values with responsive flow rather than fixed offsets
