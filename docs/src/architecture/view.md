# 3.3 View

The **View is the interface of the application**.

It turns application state into an interactive, visual experience using Beverly's component system.

Beverly provides a library of reusable primitives — Buttons, Cards, Inputs, Modals, Dropdowns, Tables, Navigation, Alerts, Badges, and more — that can be composed into larger interfaces.

The goal is simple:

> **Build the primitive once. Compose it everywhere.**

## Built-In Components

Beverly comes with a growing set of UI primitives designed for real applications.

```text id="8z6x2p"
Layout
├── Container
├── Row
├── Column
└── Stack

Input
├── Button
├── Input
├── Checkbox
├── Toggle
└── Dropdown

Content
├── Card
├── Text
├── Image
├── Icon
└── Table

Feedback
├── Alert
├── Toast
├── Modal
└── Badge

Navigation
├── Navbar
├── Tabs
├── Menu
└── Sidebar
```

These are not one-off widgets.

They are **reusable building blocks** intended to form the foundation of an application's design system.

A Button can be used once or a thousand times.

```rust id="1q5n8r"
button("Save")
button("Cancel")
button("Delete")
button("Create User")
```

The same primitives can be composed into completely different interfaces without creating a new implementation for every screen.

## Composition

Beverly views are built through composition.

```rust id="k4r7v2"
card()
    .padding(16)
    .radius(12)
    .children([
        text("Welcome to Beverly"),
        button("Get Started"),
    ])
```

Components contain other components.

Components can be nested.

Components can be reused.

This allows small primitives to become increasingly sophisticated interfaces without requiring increasingly complicated abstractions.

```text id="z2c6mv"
Primitive
    ↓
Component
    ↓
Composite Component
    ↓
Application View
    ↓
Complete Application
```

The same principle applies to your own components.

You can build a reusable `UserCard` from a Card, Text, Image, and Button, then use `UserCard` throughout the application.

> **Complex interfaces are compositions of simple things.**

## Familiar Syntax

Beverly deliberately uses a fluent API for component construction.

```rust id="e8r1kw"
card()
    .padding(16)
    .radius(12)
    .children([
        text("Hello"),
        button("Save"),
    ])
```

Traits and methods can be chained together to progressively describe a component.

If you've ever written jQuery, the pattern should feel familiar:

```text id="p5x3wd"
select something
    ↓
configure it
    ↓
configure it again
    ↓
attach behavior
    ↓
compose
```

Beverly brings that familiar style to a strongly typed Rust UI.

The difference is that the compiler understands the API.

You get fluent composition without giving up Rust's type system.

> **Familiar syntax. Strong types. Composable components.**

## Designed for Design Systems

Beverly is designed to make building and maintaining a design system a first-class use case.

Instead of every application inventing its own Button, Modal, Input, Dropdown, or notification system, teams can establish a common vocabulary of primitives and compose applications from them.

A design system might define:

```rust id="m8w2qk"
primary_button("Save")
secondary_button("Cancel")
danger_button("Delete")
```

Those components can themselves be built on Beverly primitives.

```text id="c9v4hs"
Beverly Primitives
        ↓
   Design System
        ↓
 Application Components
        ↓
     Application
```

This creates consistency across an application without forcing every screen to share the same implementation.

The primitives provide the foundation.

The design system provides the language.

The application composes that language into an experience.

## Accessibility by Construction

Accessibility is part of Beverly's component architecture.

It is not something developers are expected to remember at the end of a project.

Beverly primitives carry accessibility requirements as part of their contracts.

For example, an interactive Button has an accessibility/ARIA requirement:

```rust id="v7q3kx"
button("Save")
    .aria("label", "Save")
```

If a component requires accessibility metadata and that requirement is not satisfied, Beverly can reject the application at compile time.

That changes the accessibility workflow completely.

Instead of:

```text
Build
  ↓
QA
  ↓
Accessibility audit
  ↓
Discover missing semantics
  ↓
Fix
```

Beverly aims for:

```text
Write UI
  ↓
Compiler checks requirements
  ↓
Accessible component
```

Accessibility becomes part of the programming model.

This is particularly important for complex enterprise interfaces where accessibility problems can become expensive compliance and legal issues.

Beverly's philosophy is:

> **Don't remember to add accessibility. Build accessibility into the primitive.**

The goal is not merely to provide accessibility documentation.

The goal is to make entire categories of accessibility mistakes impossible to ship when they can be detected statically.

## Modular by Design

Beverly's component system is modular.

You don't need to build your application as one enormous UI tree.

Small components can be assembled into larger components, which can then become reusable application primitives themselves.

```rust id="f2n7kc"
fn user_card(user: &UserModel) -> impl Component {
    card()
        .padding(16)
        .children([
            text(user.name()),
            button("View Profile"),
        ])
}
```

That component can then be reused anywhere:

```rust id="r6c1ym"
column()
    .children([
        user_card(&alice),
        user_card(&bob),
        user_card(&charlie),
    ])
```

The architecture encourages developers to create a vocabulary that matches their application.

A financial application can build `AccountCard`, `TransactionTable`, and `PortfolioChart`.

A healthcare application can build `PatientCard`, `AppointmentList`, and `RecordViewer`.

An AI application can build `ToolCallCard`, `AgentStatus`, `ApprovalDialog`, and `EventInspector`.

All of them are compositions of the same underlying primitives.

## Reusability Is the Point

A UI framework becomes powerful when its components stop being screens and start becoming **assets**.

A Button isn't a screen.

A Card isn't a screen.

A Dropdown isn't a screen.

They are reusable pieces that can appear throughout an application and across applications.

That means improvements to the primitive can propagate everywhere it is used.

Accessibility improvements.

Visual improvements.

Performance improvements.

Interaction improvements.

A component becomes a maintained piece of infrastructure rather than duplicated application code.

## The View's Role

The View has a deliberately focused responsibility:

**Present application state, provide interaction, and compose the interface from reusable components.**

It does not own the application's underlying data.

It does not contain the application's business rules.

It does not need to know whether an interaction came from a human or an AI agent.

It presents the current Model and emits Events when users interact with it.

```text id="x7k4q1"
              Model
                │
             State
                ↓
              View
                │
        ┌───────┴────────┐
        ↓                ↓
   Components         Events
        │                │
   Compose +          Controller
   Render                │
                         ↓
                       Model
```

The result is a View system that is familiar to developers, predictable for AI-generated code, modular enough for serious design systems, and accessible by construction.

> **The View is where simple primitives become an application.**
