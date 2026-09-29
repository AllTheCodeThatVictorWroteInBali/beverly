# Tooltip

A Tooltip is a small text primitive that provides additional context about another UI element.

Tooltips are useful for explaining unfamiliar controls, providing concise descriptions, revealing secondary information, or giving additional context without permanently occupying interface space.

A Tooltip does not own the meaning of the thing it describes. It simply provides additional information about its target.

> **A Tooltip explains. The component it describes provides the interaction.**

## Basic Usage

A Tooltip can be attached to a component:

```rust id="x3k7pd"
button(localize("user.save"))
    .label(localize("user.save.label"))
    .aria(localize("user.save.description"))
    .tooltip(localize("user.save.tooltip"))
    .on("click", Command::User::Save)
```

The Button remains the interactive element.

The Tooltip provides additional context.

The application action still belongs to the Button and its Event/Controller wiring.

This keeps the responsibilities separate:

```text id="b6m2qx"
Component
   ↓
Tooltip
   ↓
Additional Context
```

The Tooltip does not become another application state system simply because it appears in response to interaction.

---

## Tooltip Is Text

Like Badge, Tooltip is fundamentally a form of text.

Its job is to present a small amount of additional information.

```rust id="q4a8hx"
button(localize("user.delete"))
    .label(localize("user.delete.label"))
    .aria(localize("user.delete.description"))
    .tooltip(localize("user.delete.tooltip"))
```

The Tooltip can contain localized application text:

```rust id="p8c2zw"
.tooltip(localize("user.delete.tooltip"))
```

Or application data when additional context is data-driven:

```rust id="c3y6vn"
text(User::name)
    .tooltip(User::email)
```

The Tooltip does not need to understand where the text came from.

The text could be static, localized, Model-derived, generated from application metadata, or produced by another system.

> **The Tooltip presents context. The Model owns data.**

---

## Tooltips Are Context, Not Controls

A Tooltip should not be confused with an interactive component.

Consider:

```rust id="t4kq91"
button(localize("user.delete"))
    .label(localize("user.delete.label"))
    .aria(localize("user.delete.description"))
    .tooltip(localize("user.delete.tooltip"))
    .on("click", Command::User::Delete)
```

The Button receives the interaction.

The Tooltip does not.

The user hovering or focusing the Button may cause the Tooltip to become visible, but the Tooltip itself is not the thing being activated.

The conceptual relationship is:

```text id="v8m0cx"
User Interaction
      ↓
Target Component
      ↓
Tooltip Presentation
```

This prevents a common architectural mistake: treating every visible UI reaction as an application Event.

Showing contextual help is generally a **UI presentation concern**.

Deleting a user is an **application action**.

Those are fundamentally different things.

---

## Tooltip Visibility

Tooltip visibility is normally controlled by the UI lifecycle rather than application state.

For example, Beverly can show a Tooltip when its target is hovered or focused.

```rust id="1p5j5k"
button(localize("user.save"))
    .label(localize("user.save.label"))
    .aria(localize("user.save.description"))
    .tooltip(localize("user.save.tooltip"))
```

Conceptually:

```text id="d1x3ha"
Pointer Hover
      ↓
Tooltip Visible

Keyboard Focus
      ↓
Tooltip Visible
```

The application does not need to create a `User::tooltip_open` field just because a Tooltip appears.

This is an important distinction:

> **UI presentation state does not automatically belong in the Model.**

If Tooltip visibility is purely a consequence of pointer position or focus, Beverly can manage that lifecycle internally.

If an application has a genuine domain reason to explicitly control contextual information, it can do so through ordinary application state and composition.

---

## Hover and Focus

Tooltips should not depend exclusively on a mouse.

A user navigating with a keyboard should be able to receive the same contextual information when the associated control receives focus.

For an interactive component:

```rust id="5x0d9k"
button(localize("user.archive"))
    .label(localize("user.archive.label"))
    .aria(localize("user.archive.description"))
    .tooltip(localize("user.archive.tooltip"))
    .on("click", Command::User::Archive)
```

The Tooltip is associated with the Button regardless of whether the user reached it through a pointer, keyboard, or another supported input mechanism.

This is particularly important in Beverly because the framework is intended to support serious enterprise interfaces rather than mouse-only desktop interactions.

> **Context should follow the control, not the input device.**

---

## Accessibility

A Tooltip is supplementary information. It should not be the only place where essential information exists.

For example, this is appropriate:

```rust id="j5k7wq"
button(localize("user.save"))
    .label(localize("user.save.label"))
    .aria(localize("user.save.description"))
    .tooltip(localize("user.save.tooltip"))
```

The Button already has a meaningful label and accessible description.

The Tooltip provides additional contextual information for users who benefit from seeing it.

The Tooltip should not be used to hide an essential label from assistive technology or replace the semantic identity of the control.

Beverly's accessibility principle remains:

> **Accessible by construction, not accessible by cleanup.**

A Tooltip enhances an accessible component. It should not be responsible for making an otherwise inaccessible component usable.

---

## Localization

Tooltip content should normally use the same localization system as other user-facing text.

```rust id="x0d8kw"
button(localize("user.delete"))
    .label(localize("user.delete.label"))
    .aria(localize("user.delete.description"))
    .tooltip(localize("user.delete.tooltip"))
```

This keeps the UI's visible text, accessible semantics, and contextual help within the same localization vocabulary.

The Tooltip should never require developers to hard-code English text simply because it is secondary UI.

Secondary text is still user-facing text.

---

## Data-Driven Tooltips

Tooltips become particularly useful in dense interfaces.

For example, a table may display a shortened identifier while the Tooltip provides the complete value:

```rust id="7bq0t4"
text(document.short_id())
    .tooltip(document.id())
```

Or a Badge may display a compact status while the Tooltip provides additional context:

```rust id="m5q9ya"
badge(document.status())
    .tooltip(document.status_description())
```

The presentation remains compact while additional information remains available on demand.

This is especially useful for high-density enterprise interfaces, where showing every piece of information permanently would overwhelm the primary interface.

---

## Tooltip and Badge

Badge and Tooltip complement each other but serve different purposes.

```rust id="p7x1vn"
badge(DeploymentStatus::Running)
    .tooltip(DeploymentStatus::description())
```

The Badge provides persistent, compact context.

The Tooltip provides additional, temporary context.

Conceptually:

| Primitive   | Purpose                                   |
| ----------- | ----------------------------------------- |
| `text()`    | Present information                       |
| `badge()`   | Present compact contextual information    |
| `tooltip()` | Provide additional contextual information |
| `button()`  | Provide interaction                       |

The same underlying Model data can drive all of them.

```text id="c4q7zx"
Model Data
   ├──→ Text
   ├──→ Badge
   └──→ Tooltip
```

The presentation changes.

The source of truth does not.

---

## Tooltip and Events

A Tooltip normally does not need application Events.

Pointer movement, focus changes, delays, positioning, and visibility are UI concerns.

The Button might emit:

```rust id="f1m8rx"
.on("click", Command::User::Save)
```

The Tooltip does not need to emit an application Command simply because it became visible.

This keeps the Event vocabulary focused on meaningful application behavior rather than every internal UI lifecycle detail.

If Beverly exposes Tooltip lifecycle Events for advanced UI behavior, those should remain clearly distinguished from application Commands.

> **Not every UI transition is an application Event.**

This is an important part of keeping Beverly's Event system useful.

---

## Composition

Tooltips should compose naturally with ordinary Beverly components.

```rust id="b4n6mz"
card()
    .children([
        text(Document::title),
        badge(Document::status)
            .tooltip(Document::status_description()),
    ])
```

Or:

```rust id="r8w2cd"
row()
    .children([
        text(User::name),

        button(localize("user.edit"))
            .label(localize("user.edit.label"))
            .aria(localize("user.edit.description"))
            .tooltip(localize("user.edit.tooltip"))
            .on("click", Command::User::Edit),
    ])
```

The Tooltip does not require a special composition model.

It attaches context to existing components.

> **Add context without adding architecture.**

---

## Placement and Presentation

The Tooltip is responsible for presentation concerns such as:

- positioning near its target
- appearing and disappearing appropriately
- handling hover and focus
- avoiding unnecessary clipping
- managing display timing
- rendering the tooltip surface
- adapting to the available viewport

These are UI concerns.

The application should not have to calculate tooltip coordinates or maintain timers merely to explain a control.

This is exactly the kind of complexity Beverly should hide internally while keeping the public API simple.

> **Magic in implementation. Explicitness at the API.**

The developer should be able to say:

```rust id="e3m9qw"
.tooltip(localize("user.save.tooltip"))
```

without needing to understand how Beverly determines where, when, or how the Tooltip is rendered.

---

## Tooltips Should Stay Small

A Tooltip is designed for concise contextual information.

It should not become a hidden modal, documentation system, or application panel.

If the user needs:

- substantial documentation,
- multiple controls,
- rich interaction,
- persistent information,
- complex content,

another component is probably more appropriate.

For example, use a Modal for substantial temporary interaction:

```rust id="k2d6ps"
modal()
    .visible(User::is_help_open)
    .children([
        text(localize("user.help.title")),
        text(localize("user.help.content")),
    ])
```

The Tooltip remains intentionally small.

> **A Tooltip answers a small question without interrupting the user's work.**

---

## The Principle

Tooltip demonstrates another Beverly design principle:

**Not every piece of UI behavior belongs in application architecture.**

A Tooltip may have internal state:

- hovered
- focused
- pending display delay
- visible
- positioned

But none of that necessarily belongs in the application's Model.

Those are implementation details of presenting contextual information.

The application only needs to provide the context:

```rust id="f0c7pa"
.tooltip(localize("user.save.tooltip"))
```

Beverly handles the UI mechanics.

This keeps application code focused on things that actually matter to the application.

---

## Why Tooltip Is Simple

A Tooltip should not become:

- a state-management system
- a notification system
- an Event bus
- an application workflow
- a data-fetching system
- a second accessibility system
- a replacement for documentation

It is a small presentation primitive.

The target component owns the interaction.

The application owns the meaning.

The Tooltip provides additional context.

> **The Tooltip explains. The component acts. The application decides.**

That simplicity makes Tooltip predictable for humans, easy for AI systems to generate, and easy to compose throughout an application.

## Summary

The Tooltip follows Beverly's broader architecture:

- **Text** provides the content.
- **Tooltip** provides additional contextual presentation.
- **The target component** remains responsible for interaction.
- **Model** owns application data.
- **Events** describe meaningful application activity.
- **Controller** connects application behavior.
- **Beverly** manages UI mechanics such as hover, focus, timing, positioning, and visibility.

The common case should remain extremely small:

```rust id="z4v8cy"
button(localize("user.save"))
    .label(localize("user.save.label"))
    .aria(localize("user.save.description"))
    .tooltip(localize("user.save.tooltip"))
    .on("click", Command::User::Save)
```

One line provides additional context. The rest of the application's architecture remains exactly where it belongs.

> **Context without complexity.**
