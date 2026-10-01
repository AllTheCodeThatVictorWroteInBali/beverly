# Card

A Card is a container for related content.

It provides a visual boundary around a group of components so that information, controls, and other content can be presented as a single coherent unit. A Card does not own the meaning of its contents, does not own application state, and does not decide what the application should do with the information inside it.

Cards are intentionally simple. They provide structure, presentation, composition, and optional interaction without becoming an application-specific abstraction.

> **The Card provides the surface. Its contents provide the meaning.**

## Basic Card

The simplest Card contains one piece of content.

```rust id="h7j2qa"
card()
    .children(text(localize("user.profile")))
```

More commonly, a Card contains several components:

```rust id="p4k8dm"
card()
    .children([
        text(localize("user.name")),
        text(localize("user.email")),
        button().text(localize("user.edit")),
    ])
```

The Card does not need to understand what the content represents. It simply provides a container in which the components can be composed.

This makes the Card useful for dashboards, settings, profiles, documents, agent activity, data records, notifications, and almost any other interface where related information benefits from a visual boundary.

## Composition

`children()` is the fundamental composition mechanism.

A Card can contain a single value:

```rust id="v3wq9n"
card()
    .children(text(localize("user.profile")))
```

Or many values:

```rust id="a6d2kp"
card()
    .children([
        text(localize("user.profile")),
        text(localize("user.description")),
        button().text(localize("user.edit")),
    ])
```

Those children can themselves be Cards or any other Beverly components:

```rust id="m9f4zs"
card()
    .children([
        card()
            .children(text(localize("user.account"))),

        card()
            .children(text(localize("user.preferences"))),
    ])
```

There is no separate composition system for Cards. A Card uses the same `children()` primitive as the rest of Beverly.

> **Build the primitive once. Compose it everywhere.**

## Named Sections

Cards can provide named sections when an application benefits from a common semantic structure such as a header, body, and footer.

```rust id="r5h3tx"
card()
    .header(text(localize("user.profile")))
    .body([
        text(localize("user.name")),
        text(localize("user.email")),
    ])
    .footer(button().text(localize("user.edit")))
```

These sections are implemented by Beverly as child entities carrying
`CardHeader`, `CardBody`, and `CardFooter` markers. They do not replace
`children()`; content without a named section remains a normal card child.

The same sections can accept one value or multiple values:

```rust id="q8n6yc"
card()
    .header([
        icon("user"),
        text(localize("user.profile")),
    ])
    .body([
        text(localize("user.name")),
        text(localize("user.email")),
        badge(localize("user.active")),
    ])
```

This gives applications a useful vocabulary for common layouts while keeping the underlying composition model simple.

## Styling

A Card is commonly configured through chained style methods.

```rust id="j2v8pf"
card()
    .padding(16)
    .radius(12)
    .shadow(4)
    .children([
        text(localize("user.profile")),
        text(localize("user.description")),
    ])
```

Styling remains part of the component configuration rather than requiring a separate stylesheet or styling abstraction.

Because these methods compose naturally, the same Card primitive can be used across an application's design system:

```rust id="k5s7rm"
card()
    .padding(24)
    .radius(16)
    .children([
        icon("database"),
        text(localize("database.status")),
        badge(localize("database.connected")),
    ])
```

The exact visual treatment can belong to the application's design system while the underlying Card remains the same reusable primitive.

## Card State

Cards can respond to state when the application needs state-specific presentation.

```rust id="n4c8wy"
card()
    .hover(self.elevation(8))
    .focused(self.outline_width(2))
    .disabled(self.opacity(0.5))
    .error(self.border_width(2))
    .success(self.border_width(2))
```

These declarations describe how the Card presents itself when its state changes. They do not make the Card responsible for deciding why that state exists.

For example, an application may have a Card representing a remote resource:

```rust id="t7p2mx"
card()
    .disabled(Resource::is_unavailable)
    .error(Resource::has_error)
    .children([
        text(Resource::name),
        text(Resource::status),
    ])
```

The Card presents the state. The Model or application state determines the state.

> **State determines when. Components determine what.**

## Interactive Cards

A Card is not inherently interactive.

If an application wants a Card to respond to pointer or keyboard interaction, that interaction should be explicitly declared rather than being assumed from the component's appearance.

```rust id="u6d9br"
card()
    .label(localize("document.card.label"))
    .aria(localize("document.card.description"))
    .on("click", Command::Document::Open)
    .children([
        text(Document::title),
        text(Document::summary),
    ])
```

The important part is that the Card still does not know what `Command::Document::Open` means. The Card produces a UI Event, and application wiring determines what happens next.

For a UI-specific interaction, the Card can expose an abstract UI Event:

```rust id="w8k3fs"
card()
    .label(localize("document.card.label"))
    .aria(localize("document.card.description"))
    .on("click_start", Ui::DocumentCard::Pressed)
    .on("click_end", Ui::DocumentCard::Released)
    .on("click", Ui::DocumentCard::Clicked)
```

The distinction is the same as with Button:

```text id="e2r5nk"
UI interaction
      ↓
Ui::DocumentCard::Clicked
      ↓
application wiring
      ↓
Command::Document::Open
```

The Card does not need to know that it represents a document. It only exposes the interface interaction.

## Accessibility

A purely visual Card that only contains content does not automatically become an interactive control simply because it looks like one.

When a Card is interactive, its accessible identity and description should be explicit:

```rust id="c9x4vd"
card()
    .label(localize("document.card.label"))
    .aria(localize("document.card.description"))
    .on("click", Command::Document::Open)
    .children([
        text(Document::title),
        text(Document::summary),
    ])
```

This is particularly important for Cards that visually resemble buttons, links, selectable records, or other controls.

The visual appearance should not be the only indication of what the Card does. Its semantics should be represented explicitly in the component configuration.

> **A visual container is not automatically an interactive control.**

## Data-Driven Cards

Cards become especially useful when their contents come from application data.

```rust id="b7q2lm"
card()
    .header(text(User::name))
    .body([
        text(User::email),
        badge(User::status),
    ])
```

The Card does not own `User`. It receives a projection of application state and presents it.

This means the same Card structure can represent many records:

```rust id="z5r8kc"
users.iter()
    .map(|user| {
        card()
            .header(text(user.name()))
            .body([
                text(user.email()),
                badge(user.status()),
            ])
    })
```

The Model remains the source of truth. The View turns that state into Cards.

This separation is important for AI-generated applications because the component does not need to understand where its data came from. The same Card can present database records, API responses, local files, streams, or AI-generated data.

## Conditional Content

Cards can also compose content based on application state.

```rust id="q3v6xa"
card()
    .children([
        text(User::name),

        if User::is_active {
            badge(localize("user.active"))
        } else {
            badge(localize("user.inactive"))
        },
    ])
```

The Card remains responsible only for presenting the resulting composition.

The decision about whether a user is active belongs to application state or domain logic, not to the Card itself.

## Loading and Async State

A Card can present loading state without implementing the operation that is loading.

```rust id="n8f4qy"
card()
    .children([
        text(localize("document.title")),
        text(Document::summary),
    ])
    .submitting(self.opacity(0.5))
```

A richer presentation can replace or augment its contents:

```rust id="p6t9we"
card()
    .children([
        text(Document::title),
        text(Document::summary),
    ])
    .submitting(self.children([
        spinner(),
        text(localize("document.loading")),
    ]))
```

The Card does not perform the asynchronous operation. It presents the state produced by the application.

This follows the same principle used throughout Beverly:

**The Model owns the state. The View presents the state. Events describe interaction. Controllers connect interaction to application behavior.**

## Error and Success Presentation

Cards can present application state such as errors or successful operations.

```rust id="d4m7zp"
card()
    .error(self.children([
        icon("error"),
        text(localize("document.error")),
    ]))
    .success(self.children([
        icon("check"),
        text(localize("document.saved")),
    ]))
```

The messages should come from the application's localization and state mechanisms rather than being hard-coded into the component.

The Card presents the meaning supplied by the application; it does not decide what constitutes an error or success.

## Cards Within Cards

Because Cards are ordinary composable components, they can contain other Cards.

```rust id="f8k2jd"
card()
    .header(text(localize("dashboard")))
    .body([
        card()
            .children([
                text(localize("users")),
                text(User::count),
            ]),

        card()
            .children([
                text(localize("documents")),
                text(Document::count),
            ]),
    ])
```

This is useful for dashboards and hierarchical interfaces, but it is not a special Card feature. It is simply the result of components being composable.

`children()` provides the common language.

## Card Events

When interaction is required, Card Events should remain about the Card's UI lifecycle.

```rust id="r2w6hb"
card()
    .label(localize("document.card.label"))
    .aria(localize("document.card.description"))
    .on("click_start", Ui::DocumentCard::Pressed)
    .on("click_end", Ui::DocumentCard::Released)
    .on("click", Ui::DocumentCard::Clicked)
```

These Events can be used independently of application Commands.

For example, a pressed Event could control a visual effect:

```rust id="s7n3mc"
.on("click_start", Ui::DocumentCard::Pressed)
```

while the completed interaction can be connected to application behavior:

```rust id="v4k9qa"
.on("click", Command::Document::Open)
```

This keeps the component's vocabulary independent from the application's domain vocabulary.

> **The Card reports interaction. The application decides what the interaction means.**

## The Card Contract

The Card has a deliberately small conceptual contract:

**Create. Configure. Compose. Present. Optionally interact.**

A Card provides:

- visual structure
- spacing and presentation
- composition through `children()`
- optional semantic sections such as `header`, `body`, and `footer`
- state-specific presentation
- accessibility semantics when acting as an interactive control
- UI Events when interaction is required

It does not own:

- application data
- business rules
- domain validation
- application Commands
- database operations
- API operations
- agent behavior

Those concerns belong elsewhere in the architecture.

The result is a component that can be reused from a simple settings page to a high-throughput enterprise dashboard without changing what a Card fundamentally is.

> **The Card is a container, not an application.**
