# Content & Children

Beverly components are designed to be composed.

The fundamental primitive for composition is:

```rust
.children(...)
```

A component can accept **one child or many children**.

```rust
card()
    .children(
        text("Hello")
    )
```

Or:

```rust
card()
    .children([
        text("Hello"),
        text("World"),
    ])
```

This simple rule provides a consistent way to build interfaces from small pieces.

> **Everything can be a child. Everything can be composed.**

## One Child or Many

`children()` accepts either a single value or a collection of values.

A single child is useful when the relationship is straightforward:

```rust
button("Save")
    .children(
        icon("save")
    )
```

Multiple children allow a component to become a container:

```rust
card()
    .children([
        text("Project"),
        text("Description"),
        button("Open"),
    ])
```

The API stays the same regardless of how many children a component contains.

There is no need to learn separate APIs for one child, two children, or a list of children.

## Composition Over Special Cases

Beverly components are designed to be building blocks rather than complete screens.

A component can contain another component.

That component can contain more components.

```text
Application
    ↓
Page
    ↓
Card
    ↓
Header
    ↓
Icon + Text + Button
```

The same composition model works at every level.

```rust
card()
    .children([
        row()
            .children([
                icon("document"),
                text("Report"),
            ]),

        text("Quarterly financial report"),

        button("Open"),
    ])
```

This makes components reusable without requiring the framework to predict every possible combination developers might need.

> **Build the primitive once. Compose it everywhere.**

## Named Sections

Some components have a meaningful internal structure.

A Card, for example, may have:

- Header
- Body
- Footer

For these components, Beverly can provide named sections alongside general `children()` composition.

```rust
card()
    .header(
        text("Account")
    )
    .body(
        text("Account information")
    )
    .footer(
        button("Save")
    )
```

Named sections make the intent of the interface immediately obvious.

They also provide components with a way to express their own semantic structure without requiring developers to manually construct the underlying layout.

## Sections Can Still Be Composed

Named sections follow the same content rules.

A section can receive a single value:

```rust
card()
    .header(
        text("Settings")
    )
```

Or multiple values:

```rust
card()
    .header([
        icon("settings"),
        text("Settings"),
        button("Edit"),
    ])
```

The section itself becomes a composition boundary.

```text
Card
├── Header
│   ├── Icon
│   ├── Text
│   └── Button
│
├── Body
│   └── Content
│
└── Footer
    └── Actions
```

This allows a component to provide useful semantic structure without sacrificing the flexibility of general composition.

## `children()` Is the Universal Escape Hatch

Named sections are useful when a component has a well-defined structure.

But developers should never be trapped by them.

`children()` remains the general composition primitive.

For a simple container:

```rust
container()
    .children([
        text("First"),
        text("Second"),
        text("Third"),
    ])
```

For a more structured component:

```rust
card()
    .header(text("Title"))
    .children([
        chart(data),
        table(rows),
    ])
    .footer(button("Close"))
```

The component can provide convenient named sections while still allowing arbitrary content where appropriate.

This keeps the API expressive without creating a separate abstraction for every possible layout.

## Content Is Data

Beverly treats component content as values that can be passed, returned, stored, and composed.

That means a component does not need to know where its content came from.

Content might be:

- Static text
- Another component
- A list of components
- Data-driven content
- Conditional content
- Application state
- AI-generated content

The component simply receives content and presents it according to its role.

```rust
card()
    .header(text(user.name()))
    .children([
        text(user.email()),
        badge(user.status()),
    ])
```

The View decides what should be presented.

The component decides how that content is presented.

The Model remains responsible for the underlying application data.

## Consistent Composition

The same mental model should work throughout a Beverly application.

```rust
container()
    .children([
        navbar(),
        card()
            .header(text("Users"))
            .children([
                user_table(),
            ])
            .footer([
                button("Previous"),
                button("Next"),
            ]),
    ])
```

Developers don't need to memorize a different composition system for every component.

They learn one fundamental concept:

```text
Component
    ↓
children()
    ↓
Child or Children
```

Named sections simply add semantic structure where it is useful.

## Why This Matters for AI

A consistent composition model is particularly valuable when code is generated by LLMs.

The model does not need to invent a new layout abstraction for every component.

It learns a small vocabulary:

```text
component()
children(...)
header(...)
body(...)
footer(...)
```

That makes generated code easier to predict and easier for humans to review.

A developer looking at:

```rust
card()
    .header(...)
    .children(...)
    .footer(...)
```

can immediately understand the structure.

The code itself becomes a description of the interface.

> **Composition should be obvious from the code.**

## The Beverly Composition Rule

Beverly keeps content composition deliberately simple:

**Use `children()` for general composition.**

**Use named sections when a component has meaningful semantic regions.**

**Accept one child or many children.**

**Keep components composable.**

**Don't create a new abstraction when ordinary composition already solves the problem.**

The result is a UI system where small primitives can become complex interfaces without the API becoming complex.

> **Small primitives. Simple composition. Unlimited combinations.**
