# Width and Height

Width and height define the intended dimensions of a layout region. They can be fixed, relative to the parent, or left for the layout system and content to resolve automatically.

## When to use

Use explicit dimensions when a region has a stable format or interaction requirement:

- icon and media surfaces with a known aspect
- navigation bars and tool regions
- full-width fields and panels
- modal or card bounds

Use relative values when a region should participate in its parent's available space. Use intrinsic or automatic sizing when content should determine the dimension.

## Dimension choices

```text
Fixed       -> stable pixel or logical size
Percentage  -> proportional to the parent
Auto        -> resolved from content and layout rules
Min / Max   -> bounded flexible size
```

Prefer the least rigid choice that preserves usability. A fixed height can stabilize a toolbar, while a content region usually benefits from a flexible width with a readable maximum.

## Example

```rust
Node {
	width: Val::Percent(100.0),
	height: Val::Auto,
	min_height: Val::Px(120.0),
	max_width: Val::Px(720.0),
	..default()
}
```

## Guidance

- use stable dimensions for fixed-format controls and media
- let text-heavy regions grow when content can vary
- pair fluid width with max width to preserve readable line length
- avoid fixed heights around content that may wrap or localize
- check dimensions with empty, short, and long content
