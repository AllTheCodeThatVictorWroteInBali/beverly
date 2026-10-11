# Loading Skeleton

A skeleton is a placeholder silhouette that stands in for content while it loads. It keeps the layout stable and shows where content will appear. `Skeleton` is decorative and non-interactive, and `SkeletonGroup` marks a whole loading region as busy for assistive technology.

`BeverlyPlugin` installs `SkeletonPlugin`. Skeletons are not progress indicators: they say that something is loading, not how far along it is. For that, see [Loading](loading.md).

## Basic Usage

`Skeleton` is a component. Spawn one with a size, and it paints itself from the theme:

```rust
use beverly::prelude::*;
use bevy::prelude::*;

fn add_placeholders(parent: &mut ChildSpawnerCommands) {
    // A circle for an avatar.
    parent.spawn(Skeleton::circle(56.0));

    // A text line that follows the theme's body type size.
    parent.spawn(Skeleton::text().width(Val::Percent(55.0)));

    // A block with an explicit size and corner radius.
    parent.spawn(
        Skeleton::new()
            .width(Val::Percent(100.0))
            .height(Val::Px(140.0))
            .radius(8.0),
    );
}
```

A skeleton adds a `Node` and a `Surface` for you and never replaces a `Node` you provide. Your margin, positioning, flex settings, and clipping are kept. Only the width and height you set on the skeleton are applied.

Skeletons ignore pointer input, so they never block clicks on anything behind them.

## Constructors

| Constructor | Shape |
| --- | --- |
| `Skeleton::new()` | A rounded block with a 6 px radius. It leaves your `Node` size untouched |
| `Skeleton::text()` | A text line with a 3 px radius and a height taken from the theme |
| `Skeleton::circle(size)` | A fixed `size` by `size` circle |

### Text height

`Skeleton::text()` sets its height to the theme's body font size times 1.25 when you don't give it one, so a line placeholder matches a real line of text. The height tracks the theme live. If you set `.height(...)` on the skeleton, or a `Node` height of your own, yours is kept.

## Text Blocks

`Skeleton::text_lines(count)` builds a labeled group of text lines in one step:

```rust
use beverly::prelude::*;
use bevy::prelude::*;

fn loading_paragraph(mut commands: Commands, parent: Entity) {
    let group = Skeleton::text_lines(3)
        .label("Loading description")
        .width(Val::Percent(100.0))
        .gap(8.0)
        .final_width(Val::Percent(60.0))
        .spawn(&mut commands);
    commands.entity(parent).add_child(group);
}
```

| Setter | Effect |
| --- | --- |
| `label(text)` | The accessible name of the loading region. Defaults to "Loading content" |
| `width(val)` | Width of the group |
| `gap(pixels)` | Space between lines. Defaults to 8 px |
| `final_width(val)` | Makes the last line shorter, like the end of a paragraph |
| `line(skeleton)` | Shared styling for every line, such as direction or phase |
| `node(node)` | Replaces the whole group layout |

`spawn` returns the group entity, so you can attach it to an existing parent.

## Loading Regions

Wrap any number of skeletons in a `SkeletonGroup` to announce one loading region instead of many separate shapes:

```rust
use beverly::animation::skeleton::SkeletonGroup;
use beverly::prelude::*;
use bevy::prelude::*;

fn loading_card(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn(SkeletonGroup::new("Loading biscuit delivery"))
        .with_children(|group| {
            group.spawn(Skeleton::circle(56.0));
            group.spawn(Skeleton::text().width(Val::Percent(55.0)));
            group.spawn(Skeleton::text().width(Val::Percent(100.0)));
        });
}
```

The group is exposed to assistive technology as one busy, polite region with your label, so a screen reader can announce it without reading each decorative shape. The individual skeletons are decorative and contribute no semantics of their own.

The group defaults to a column layout with an 8 px gap. Any layout you set on the group's own `Node` wins over those defaults.

When content arrives, **despawn the loading group** and spawn the real content. If you keep the region and swap its children instead, call `.busy(false)` on the group so the busy state clears:

```rust
use beverly::animation::skeleton::SkeletonGroup;

let finished = SkeletonGroup::new("Loading biscuit delivery").busy(false);
```

## Shimmer

By default a skeleton shows a soft highlight that sweeps across it. These settings control it:

| Setter | Default | Effect |
| --- | --- | --- |
| `direction(dir)` | `LeftToRight` | `SkeletonDirection::LeftToRight` or `RightToLeft`, from `beverly::animation::skeleton` |
| `duration(seconds)` | 1.5 | Time for one sweep, from 0.1 to 3600 seconds |
| `phase(cycles)` | 0 | Shifts the sweep in the cycle, so neighbors can be staggered |
| `highlight_width(value)` | 0.22 | Width of the highlight band as a fraction of the skeleton, from 0.01 to 0.8 |
| `highlight_softness(value)` | 0.8 | How soft the band's edges are, from 0.05 to 1 |
| `highlight_intensity(value)` | 0.65 | Strength of the highlight, from 0 to 1 |
| `enabled(bool)` | `true` | `false` shows a still bar instead of a sweep |

Invalid or out-of-range values are clamped to these ranges, and non-finite values fall back to their defaults.

The sweep runs on the GPU clock, so all skeletons share one time source and stay in step without any system of your own. Use `phase` to offset them if you want a staggered look.

## Colors

Skeletons use the theme's `skeleton_base` and `skeleton_highlight` colors, so they follow light and dark mode automatically:

| Theme | Base | Highlight |
| --- | --- | --- |
| Light | a pale cool grey | near white |
| Dark | a dark slate | a lighter slate |

Override them per skeleton with `.base_color(color)` and `.highlight_color(color)`, or use `.color(color)` for the base alone. An explicit color stays fixed when the theme changes. Call `.theme_colors()` to remove both overrides and follow the theme again.

## Accessibility

Skeletons adapt to the user's visual preferences without any extra code:

- **Reduced motion or reduced effects:** the sweep is turned off and each skeleton becomes a still bar in its base color. This also applies if the theme has reduced effects enabled.
- **High contrast:** the sweep is turned off, and the base color switches to the theme's muted text color to keep a strong silhouette. Your custom colors are **ignored** in this mode.
- **Reduced transparency:** the base and highlight colors are made fully opaque.

A skeleton never carries information on its own. Pair it with a `SkeletonGroup` label so the loading state is announced, and replace it with real content as soon as it is ready.

## Design Guidance

- Match the placeholders to the shape and size of the content they stand in for, so nothing jumps when the content arrives.
- Use skeletons for larger surfaces such as cards, lists, and profiles. For a short action, a spinner or button state is usually clearer.
- Don't leave a skeleton on screen indefinitely. If loading fails, replace it with an error message and a retry action.
- Keep the shapes simple. A few blocks that hint at the structure read better than a detailed imitation.

## Updating a Skeleton

A skeleton's settings are fixed when it is created. To change them, replace the component on the entity with a new `Skeleton`, or despawn and rebuild the group. The demo rebuilds its placeholders when you change the sweep controls.

## Demo

Run the interactive demo from the repository root:

```sh
cargo run --example skeleton
cargo run --example skeleton -- dark
```

The demo shows a loading card with a circle avatar, two header lines, an image block, and three text lines inside one `SkeletonGroup`. Three buttons control it:

- **Skeleton** shows the placeholders.
- **Content** swaps them for the real content.
- **Reload** shows the skeleton for 2.5 seconds, then switches to the content.

Two more buttons set the sweep direction, and sliders set the sweep duration (0.5 to 4 seconds) and highlight intensity (0 to 100 percent). Changing a sweep setting rebuilds the placeholders.

With reduced motion or high contrast on, the placeholders appear as still bars. The top-right toggle switches the page between light and dark mode.
