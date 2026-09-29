# Alert

An Alert is a pre-styled Card used to communicate information to the user.

Alerts follow the familiar contextual pattern popularized by Bootstrap: a common surface with a predefined visual treatment for different kinds of information, such as success, warning, error, and informational messages.

Under the hood, an Alert is simply a Card with a predefined look and feel. It does not introduce a new layout system, state-management system, or application architecture.

The type is provided when the Alert is created:

```rust
alert(AlertType::Success)
```

This makes the semantic purpose of the component immediately visible in the code.

> **An Alert is a Card with an opinionated presentation.**

## Basic Alert

The simplest Alert presents a message:

```rust
alert(AlertType::Info)
    .children(
        text(localize("user.profile_updated"))
    )
```

The Alert provides the visual treatment. The content provides the meaning.

A more complete Alert might contain an icon and text:

```rust
alert(AlertType::Info)
    .children([
        icon("info"),
        text(localize("user.profile_description")),
    ])
```

The Alert does not need to understand what the message means. It simply presents its children using the visual language associated with the selected Alert type.

## Alert Types

The Alert type is the first parameter because it is fundamental to what the component represents.

```rust
alert(AlertType::Info)
    .children(text(localize("user.profile_updated")))
```

```rust
alert(AlertType::Success)
    .children(text(localize("user.created")))
```

```rust
alert(AlertType::Warning)
    .children(text(localize("user.unsaved_changes")))
```

```rust
alert(AlertType::Error)
    .children(text(localize("user.create_failed")))
```

The type determines the predefined presentation. It does not determine application state.

Conceptually:

```text
Alert Type
    ↓
Visual Treatment
    ↓
Alert Content
```

The application chooses `AlertType::Error` because it wants to present something as an error. The Alert does not create or detect that error.

> **The type describes the presentation. The application owns the meaning.**

## Why the Type Is a Parameter

The Alert type is not merely a style modifier.

Compare:

```rust
alert(AlertType::Success)
```

with:

```rust
alert()
    .success()
```

The first form makes the semantic variant part of the component's construction. You can understand the component by reading its first argument.

That makes the API particularly useful for AI-generated code:

```rust
alert(AlertType::Warning)
```

immediately communicates:

> This is a warning Alert.

There is no need to inspect later configuration to discover the Alert's primary semantic variant.

The remaining chained methods configure the Alert itself:

```rust
alert(AlertType::Warning)
    .padding(16)
    .children(...)
```

This follows a useful Beverly convention:

> **Construction establishes what something is. Configuration establishes how it behaves or looks.**

## Alerts Are Cards

An Alert is built on the Card primitive.

A general Card might look like:

```rust
card()
    .padding(16)
    .radius(12)
    .children([
        icon("info"),
        text(localize("user.profile_updated")),
    ])
```

An Alert provides the same fundamental container behavior with a predefined semantic presentation:

```rust
alert(AlertType::Info)
    .children([
        icon("info"),
        text(localize("user.profile_updated")),
    ])
```

The Alert saves the application from repeatedly defining the same visual recipe.

Instead of every part of an application independently deciding how an informational message should look, the design system establishes that convention once.

The implementation can remain simple because the Alert does not need to become a new UI framework. It is a specialized presentation of an existing primitive.

> **If it is fundamentally a Card, make it a Card. Give it a name when the name carries useful meaning.**

## Composition

Alerts use `children()` just like Cards.

A single child:

```rust
alert(AlertType::Success)
    .children(
        text(localize("user.saved"))
    )
```

Multiple children:

```rust
alert(AlertType::Success)
    .children([
        icon("check"),
        text(localize("user.saved")),
        badge(localize("user.synced")),
    ])
```

More complex content can be composed normally:

```rust
alert(AlertType::Warning)
    .children([
        text(localize("user.unsaved_changes")),

        button(localize("user.save"))
            .label(localize("user.save.label"))
            .aria(localize("user.save.description"))
            .on("click", Command::User::Save),
    ])
```

The Alert does not need special APIs for every possible message layout.

It provides the visual container and the predefined treatment. `children()` provides composition.

## Styling

Alerts have a predefined visual identity, but they remain Beverly components and can participate in normal styling.

```rust
alert(AlertType::Success)
    .padding(16)
    .radius(12)
    .children([
        icon("check"),
        text(localize("user.saved")),
    ])
```

The Alert type establishes its default visual treatment. Additional styling can refine the component where appropriate.

This keeps the API expressive without forcing developers to manually reproduce the Alert design.

```rust
alert(AlertType::Error)
```

is enough to get the application's standard error presentation.

The developer does not need to know which colors, borders, spacing, typography, or icon treatment make up that presentation.

> **Declare the meaning. Let the component provide the convention.**

## Accessibility

An Alert communicates information, so its accessible semantics should be explicit.

```rust
alert(AlertType::Error)
    .label(localize("user.create_status.label"))
    .aria(localize("user.create_status.description"))
    .children([
        icon("error"),
        text(localize("user.create_failed")),
    ])
```

The Alert's accessibility metadata describes the communication surface itself.

Its children retain their own accessibility requirements. An interactive Button inside an Alert still declares its own label and ARIA description:

```rust
alert(AlertType::Warning)
    .children([
        text(localize("user.unsaved_changes")),

        button(localize("user.save"))
            .label(localize("user.save.label"))
            .aria(localize("user.save.description"))
            .on("click", Command::User::Save),
    ])
```

Accessibility therefore remains compositional.

The Alert provides semantics for the message container. Interactive children provide their own semantics.

Color alone should not be relied upon to communicate the meaning of an Alert; the content or accessible semantics should make that meaning available to assistive technology as well. This is also a documented concern in Bootstrap's Alert guidance.

> **The visual treatment communicates. The accessible semantics make that communication explicit.**

## Static Alerts

An Alert does not need to be connected to changing application state.

It can simply communicate information that is always present:

```rust
alert(AlertType::Info)
    .children(
        text(localize("user.profile_public"))
    )
```

The Alert is simply part of the View.

There is no requirement that every Alert represent a changing state.

## Data-Driven Alerts

An Alert can also be generated from application state.

```rust
if User::has_error {
    alert(AlertType::Error)
        .children(
            text(User::error_message)
        )
}
```

Or different application conditions can determine which presentation should be used:

```rust
if User::is_saved {
    alert(AlertType::Success)
        .children(text(localize("user.saved")))
} else if User::has_error {
    alert(AlertType::Error)
        .children(text(User::error_message))
} else {
    alert(AlertType::Info)
        .children(text(localize("user.ready")))
}
```

The Alert is still not responsible for deciding which condition is true.

The Model owns the state. The View chooses the appropriate presentation.

## Alert Type Is Not Application State

This distinction is important.

Consider:

```rust
alert(AlertType::Error)
    .children(text(User::error_message))
```

The Alert type says:

> Present this content using the error treatment.

It does **not** mean:

> Change the application's state to Error.

The application already knows that an error exists. The View chooses the Alert presentation that communicates that state to the user.

This keeps presentation separate from state management.

> **Presentation describes state. It does not create state.**

## Alert Events

An Alert does not inherently represent an interaction.

Unlike a Button, an Alert's primary responsibility is communication. It does not need to emit an Event merely because it exists.

If an Alert contains an interactive component, that component produces its own UI Events.

```rust
alert(AlertType::Warning)
    .children([
        text(localize("user.unsaved_changes")),

        button(localize("user.save"))
            .label(localize("user.save.label"))
            .aria(localize("user.save.description"))
            .on("click", Command::User::Save),
    ])
```

The Alert remains a presentation container. The Button handles the interaction.

> **An Alert communicates. Interactive children interact.**

## Dismissible Alerts

An Alert can optionally contain a control that allows the user to dismiss it.

The dismissal interaction belongs to the child Button:

```rust
alert(AlertType::Info)
    .children([
        text(localize("user.profile_updated")),

        button(localize("user.dismiss"))
            .label(localize("user.dismiss.label"))
            .aria(localize("user.dismiss.description"))
            .on("click", Command::Alert::Dismiss),
    ])
```

The Alert does not need to become a notification manager or invent its own dismissal architecture.

The application decides what `Command::Alert::Dismiss` means and which application state should change.

This is an important difference from frameworks where an Alert component may contain its own imperative dismissal mechanism. Beverly keeps the application state in the application and uses the normal Event → Controller → Model architecture.

## Alert State

An Alert can also participate in normal component state presentation.

For example:

```rust
alert(AlertType::Error)
    .hover(self.opacity(0.95))
    .focused(self.outline_width(2))
    .children([
        icon("error"),
        text(User::error_message),
    ])
```

These are presentation states of the component. They do not change what kind of Alert it is.

The type remains:

```rust
AlertType::Error
```

while the component's visual state can change based on interaction or application state.

This gives us two separate concepts:

```text
Alert Type
    ↓
What kind of message is being presented?

Component State
    ↓
How should the Alert currently appear?
```

That separation prevents the two concepts from becoming tangled.

## Alert and Localization

Alert content should normally come from the application's localization system.

```rust
alert(AlertType::Success)
    .children(
        text(localize("user.created"))
    )
```

The Alert type itself is not language-specific. It provides the presentation convention.

For dynamic errors:

```rust
alert(AlertType::Error)
    .children(
        text(localize(User::error_key))
    )
```

The Model provides semantic state or error information. Localization provides the appropriate language. The Alert presents it.

> **The Model returns meaning. The UI presents language.**

## Alert as a Design-System Primitive

The real value of Alert is consistency.

Without an Alert primitive, different parts of an application might independently implement success messages:

```text
green Card
green banner
green notification
green bordered container
```

Each implementation could look slightly different.

With an Alert:

```rust
alert(AlertType::Success)
    .children(...)
```

the design system establishes the convention once.

The application gets consistent presentation without requiring every developer or AI agent to reconstruct the design.

This is particularly useful for AI-generated interfaces. An agent does not need to invent a visual treatment every time it needs to communicate success or failure. It can use the existing semantic primitive.

## Alert vs. Card

The relationship is deliberately simple.

| Component     | Purpose                                                   |
| ------------- | --------------------------------------------------------- |
| `card()`      | General-purpose content container                         |
| `alert(type)` | Pre-styled Card for communicating information             |
| `modal()`     | Temporary Card-like surface with an interaction lifecycle |

A Card is general.

An Alert is opinionated.

A Modal adds a temporary interaction boundary.

All three share the same underlying composition philosophy.

## Alert vs. Modal

An Alert and Modal can both communicate an error or success state, but they serve different purposes.

An Alert is part of the existing interface:

```rust
alert(AlertType::Error)
    .children(
        text(localize("user.create_failed"))
    )
```

A Modal temporarily places content above the existing interface:

```rust
modal()
    .label(localize("user.create_error.label"))
    .aria(localize("user.create_error.description"))
    .children([
        text(localize("user.create_failed")),
    ])
```

The difference is not the message. It is the interaction surface.

The Alert communicates within the current interface. The Modal establishes a temporary interface context.

## Alert Contract

The Alert has a deliberately small conceptual contract:

**Create with a type. Compose. Present.**

It provides:

- predefined information, success, warning, and error treatments
- Card-based composition
- `children()` support
- consistent design-system presentation
- accessibility semantics
- optional styling refinement
- optional interactive children

It does not own:

- application state
- business logic
- validation
- application Commands
- database operations
- API operations
- agent behavior
- notification infrastructure

Those concerns remain outside the Alert.

The Alert is therefore a small, semantic abstraction over Card rather than another subsystem.

## The Principle

The Alert should be one of Beverly's simplest components.

Its API says exactly what it is:

```rust
alert(AlertType::Success)
```

Then the rest of the configuration describes its contents:

```rust
alert(AlertType::Success)
    .children([
        icon("check"),
        text(localize("user.created")),
    ])
```

The type carries the semantic presentation. `children()` carries the content. The application state determines when the Alert exists. The Model owns the underlying meaning.

There is no need for an Alert state machine, notification framework, or separate composition system.

> **Create the primitive with its meaning. Configure the presentation. Compose the content.**
