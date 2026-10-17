# Styling DSL

`UiElement` exposes fluent layout and paint methods. Styles are explicit
overrides, separate from a component's state, bindings, and semantic behavior.
`BeverlyPlugin` installs `StylingPlugin`; custom Bevy apps must install the
styling plugin to keep overrides applied as component visuals change.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

let settings = UiElement::form(Form::new())
    .padding(24)
    .radius(8)
    .gap(16)
    .width(400)
    .max_width(percent(100))
    .background("#ffffff")
    .border_width(1)
    .border_color("#d4d4d4")
    .color("#171717")
    .font_size(16)
    .header(text("Account settings"))
    .body(UiElement::input("Email").label("Email address"))
    .footer(text("Changes apply when submitted."));
```

Place the resulting element inside `app().children(...)`, or materialize it
using `spawn_in_context`. Concrete control builders retain their existing
configuration methods; wrap them in `UiElement` or call `.bind(...)` / `.error(...)`
before applying the shared styling methods.

## Methods

| Purpose | Methods | Values |
|---|---|---|
| Padding | `padding`, `padding_x`, `padding_y`, `padding_rect` | Pixel number, `Val`, or `UiRect` for the rect variant |
| Margin | `margin`, `margin_x`, `margin_y`, `margin_rect` | Pixel number, `Val`, or `UiRect` for the rect variant |
| Dimensions | `width`, `height`, `min_width`, `max_width`, `min_height`, `max_height` | Pixel number or `Val` |
| Gaps | `gap`, `row_gap`, `column_gap` | Pixel number or `Val` |
| Corners | `radius` | Nonnegative pixel radius |
| Fill | `background` | `Color`, CSS color string, or `Paint` |
| Border | `border_width`, `border_color` | Nonnegative pixel width; `Color` or CSS color string |
| Typography | `color`, `font_size` | `Color` or CSS color string; nonnegative pixel size |
| Alignment | `align_items`, `align_self`, `justify_content` | Bevy's corresponding enum |
| Flex sizing | `flex_grow`, `flex_shrink` | Nonnegative number |
| Transform | `scale`, `scale_xy` | Uniform number or `Vec2` |
| Opacity | `opacity` | Finite number, clamped to `0..=1` |
| Outline | `outline_width`, `outline_color` | Nonnegative pixel width; `Color` or CSS color string |
| Interaction states | `hover`, `focused`, `pressed`, `active`, `inactive`, `disabled_style` | `FnOnce(UiStyle) -> UiStyle` patch |
| Form status states | `submitting_style`, `success_style` | `FnOnce(UiStyle) -> UiStyle` patch |

Numbers accept ordinary integer and floating-point literals. For example,
`.width(400)`, `.width(px(400))`, `.width(percent(100))`, and `.width(Val::Auto)`
use the same API. `.gap(16)` sets both axes; follow it with `.row_gap(8)` to
override only the row gap. Axis-specific spacing preserves the other explicitly
configured axis, and rect variants accept individual side values.

CSS color strings support named colors, hex, `rgb(...)`, and `hsl(...)` through
`csscolorparser`. Invalid strings produce a clear construction-time panic;
parse untrusted user input before passing it to the styling builder.

Interaction-state styles are explicit patches built from `UiStyle`; they do
not use an implicit `self` component context:

```rust
button()
    .text("Save")
    .background("#2563eb")
    .hover(|style| style.background("#1d4ed8"))
    .focused(|style| style.outline_width(2).outline_color("#f59e0b"))
    .pressed(|style| style.scale(0.98))
    .disabled_style(|style| style.opacity(0.5))
```

For an already-bound form control, the same styles follow checked/focus and
app-owned form status:

```rust
CheckboxConfig::new()
    .label("Enable alerts")
    .bind(User::alerts)
    .active(|style| style.color("#15803d"))
    .inactive(|style| style.color("#737373"))
    .submitting_style(|style| style.opacity(0.6))
    .success_style(|style| style.outline_color("#16a34a"))
```

`active`/`inactive` read checked, selected, or pressed semantic state.
`focused` reads Beverly semantic/input focus; `hover` and `pressed` use Bevy's
`Interaction`. `disabled_style` reads disabled semantic state, explicit
`DisabledInteraction`, or the component's supported disabled property.
`submitting_style` and `success_style` read app-owned `FormStatusState`.
They therefore do nothing until the corresponding state exists on the styled
entity. State precedence is inactive, active, hover, focused, pressed,
submitting/success, then disabled: later active states override only the
properties they set. For example, disabled opacity wins while a disabled
control still happens to be hovered. Boolean configuration remains
`.disabled(true)`; use `.disabled_style(|style| ...)` for its appearance.

```rust
column()
    .padding_x(16)
    .padding_y(12)
    .margin_rect(UiRect::top(px(8)))
    .gap(8)
    .align_items(AlignItems::Stretch)
    .justify_content(JustifyContent::FlexStart)
    .children([
        text("Summary").font_size(20).color("rgb(23, 23, 23)"),
        button().text("Continue").background(Color::BLACK).radius(6),
    ])
```

## Precedence and Composition

Later assignments to the same property win. Styling can be interleaved with
section composition and bound/error wrappers without changing the returned
control entity or its form metadata:

```rust
UiElement::input("Email")
    .label("Email address")
    .bind(User::email)
    .error(|errors| text(errors.join("\n")))
    .width(320)
    .padding(12)
    .border_color("#737373")
```

Explicit values are reapplied after component/theme updates and before layout.
Unspecified properties retain the component's own behavior. Parent foreground
color/font size styles apply to descendant text and icons; a descendant's own
explicit fluent style wins over inherited styling. Error presentation remains
outside the control's styled content subtree, and its host tracks styled width.

For Modal and Toast, sizing and paint target the visible surface rather than
the overlay or notification-positioning lane. For Select and Dropdown, sizing
targets the containing control and padding/paint target the trigger. Sections
remain in their normal top/middle/bottom order.

Opacity multiplies through ancestors and applies to surface fills/borders,
text, icons, and ordinary node backgrounds. It preserves gradient/procedural
paint structure and does not compound on successive updates. Arbitrary custom
material uniforms and external images are not opacity adapters; configure
those materials directly. Semantic focus decorations remain independent from
fill/border styling. An outline is an additional static outline, not a
replacement for keyboard focus indication.

## Scope

This is an explicit styling API, not a CSS parser or selector engine. The
interaction-state methods accept `UiStyle` patches; the older implicit
`.hover(self...)`/`.focused(self...)` spelling is not supported. `.success(...)`
and `.submitting(...)` are separate content callbacks; use
`.success_style(...)` / `.submitting_style(...)` to change paint or layout.
Style callbacks are static patches, not arbitrary per-frame application
predicates. Opacity and scale are static values, not animations. Clearing an
optional style field stops
enforcing it; it does not restore a captured original value. Assign a concrete
replacement such as `Val::Auto` or the desired theme paint when resetting a
property. Opacity restores its tracked base alpha when reset to `1`.

Opacity and scale
do not change disabled state or accessible visibility; use the appropriate
control/semantic APIs for those behaviors. Check contrast and focus visibility
when overriding theme colors.