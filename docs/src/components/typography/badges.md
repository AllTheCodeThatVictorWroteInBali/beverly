# Badge

A Badge is a compact text primitive used to communicate a short piece of contextual information.

Badges are useful for statuses, categories, counts, labels, states, and other small pieces of information that benefit from visual emphasis.

The important architectural distinction is that a Badge is **presentation, not application state**. The application decides what the text means and when it changes. The Badge presents that information.

> **A Badge is text with an opinionated presentation.**

## Basic Usage

A Badge can be created directly from text:

```rust
badge("Active")
```

For application text, localization should normally be used:

```rust
badge(localize("user.status.active"))
```

Or the Badge can present application data directly:

```rust
badge(User::status)
```

The Model owns `User::status`. The Badge presents it.

This keeps the responsibility clear:

```text
Model
  ↓
Application State
  ↓
Badge
  ↓
Presentation
```

The Badge does not decide what the data means. It presents it.

---

## Default Badge Types

Beverly provides a set of default `BadgeType` values for common cases.

For example:

```rust
badge(BadgeType::Success, localize("user.active"))
```

```rust
badge(BadgeType::Warning, localize("user.pending"))
```

```rust
badge(BadgeType::Error, localize("user.failed"))
```

```rust
badge(BadgeType::Info, localize("user.new"))
```

These types provide a common vocabulary for applications that need familiar contextual treatments.

But they are **defaults, not a restriction**.

Beverly should not attempt to define every possible kind of badge an application could need.

A healthcare application, for example, may have:

```rust
PatientStatus::Stable
PatientStatus::Monitoring
PatientStatus::Critical
```

A deployment platform might have:

```rust
DeploymentStatus::Building
DeploymentStatus::Running
DeploymentStatus::Failed
DeploymentStatus::RolledBack
```

An enterprise application might have:

```rust
AccountTier::Free
AccountTier::Pro
AccountTier::Enterprise
```

These are different domains with different vocabularies.

Beverly should allow the application to define its own.

> **Beverly provides a vocabulary. It does not define your domain.**

---

## Your Data, Your Type System

The most useful Badge type is often the one that accurately represents the application's actual data.

For example:

```rust
enum DeploymentStatus {
    Building,
    Running,
    Failed,
    RolledBack,
}
```

The application can define how those values should be presented:

```rust
impl BadgePresentation for DeploymentStatus {
    fn badge(&self) -> Badge {
        match self {
            Self::Building =>
                badge("Building")
                    .style(building_style()),

            Self::Running =>
                badge("Running")
                    .style(running_style()),

            Self::Failed =>
                badge("Failed")
                    .style(failed_style()),

            Self::RolledBack =>
                badge("Rolled Back")
                    .style(rolled_back_style()),
        }
    }
}
```

Then the View can simply express the domain:

```rust
deployment.status().badge()
```

The important part is not the exact API. The architectural principle is:

**Your domain types should describe your domain. Beverly should make those types easy to present.**

This is preferable to forcing application data into a generic set of UI categories simply because those categories already exist in the framework.

---

## Default Types Are a Starting Point

The default types are useful when they accurately describe what the application needs.

```rust
badge(BadgeType::Success, "Running")
```

is perfectly reasonable when `Success` is the correct semantic representation.

But an application should not have to pretend that every status is `Success`, `Warning`, `Error`, or `Info`.

For example, a financial application might need:

```rust
TransactionType::Pending
TransactionType::Settled
TransactionType::Reversed
TransactionType::Disputed
```

A logistics application might need:

```rust
ShipmentStatus::PickedUp
ShipmentStatus::InTransit
ShipmentStatus::Delayed
ShipmentStatus::Delivered
```

Those types carry information that a generic `BadgeType` cannot.

The framework provides the common case. The application owns the specific case.

> **Use the default vocabulary when it fits. Define your own vocabulary when it doesn't.**

---

## Type Should Reflect Meaning

A Badge type should describe something meaningful about the data being presented.

For example:

```rust
badge(BadgeType::Warning, "Delayed")
```

communicates a generic presentation.

But:

```rust
badge(ShipmentStatus::Delayed)
```

can communicate a domain concept directly.

That distinction becomes especially valuable as applications become larger and more data-driven.

The application can reason about:

```rust
ShipmentStatus::Delayed
```

instead of passing around arbitrary strings such as:

```rust
"yellow"
"warning"
"delayed"
```

The UI remains strongly typed while the domain remains expressive.

This is one of the places where Rust provides a natural advantage: the application can define precise domain types, and Beverly can turn those types into visual components.

---

## Presentation Does Not Own the Domain

Even when a custom type determines how a Badge looks, the Badge still does not own the underlying state.

For example:

```rust
badge(DeploymentStatus::Failed)
```

does not mean the Badge knows why the deployment failed.

The Model owns the deployment.

The domain type describes its state.

The Badge determines how that state is presented.

Conceptually:

```text
Domain Type
    ↓
Presentation Mapping
    ↓
Badge
```

This keeps the boundary clean.

The Model can be tested without the UI.

The presentation mapping can be tested independently.

The Badge remains a simple reusable component.

---

## Text Is the Primitive

A Badge is fundamentally a form of text.

That means it should inherit the same basic composition philosophy as Beverly's other text-based primitives rather than introducing another abstraction for content.

A Badge can contain static text:

```rust
badge("New")
```

Localized text:

```rust
badge(localize("document.new"))
```

Or application data:

```rust
badge(Document::status)
```

It can also be composed with other content:

```rust
badge()
    .children([
        icon("check"),
        text(localize("user.active")),
    ])
```

The Badge does not require a separate content model.

> **If the primitive is text, use the text system.**

---

## Styling

Badges can be configured using Beverly's fluent styling model.

```rust
badge("Active")
    .padding_x(8)
    .padding_y(4)
    .radius(999)
```

More complete styling can be composed normally:

```rust
badge(localize("user.active"))
    .font_size(12)
    .font_weight(600)
    .padding_x(8)
    .padding_y(4)
    .radius(999)
```

The API remains familiar:

```text
Create → Configure → Compose
```

There is no separate styling system required to understand Badge.

---

## Data-Driven Badges

Badges become particularly useful when presenting collections of application data.

For example:

```rust
users.iter().map(|user| {
    badge(user.status())
})
```

Or:

```rust
documents.iter().map(|document| {
    badge(document.category())
})
```

The Badge remains unaware of where the data came from.

The data could come from SQLite, Parquet, an API, a stream, a data lake, or an AI-generated result.

The Badge only receives the value that should be presented.

This is another consequence of keeping the View thin:

> **The View presents data. The Model owns data.**

---

## Badges Are Not Controls

A Badge is not inherently interactive.

It does not represent a button, input, command, or action.

```rust
badge("Active")
```

is informational.

If the user needs to perform an action, use an interactive primitive:

```rust
button().text(localize("user.manage"))
    .on("click", Command::User::Manage)
```

The Badge and Button can be composed:

```rust
row()
    .children([
        text(User::name),
        badge(User::status),
        button().text(localize("user.manage"))
            .on("click", Command::User::Manage),
    ])
```

The responsibilities remain separate.

The Badge communicates.

The Button provides interaction.

The Command expresses application action.

---

## Accessibility

A Badge should communicate meaning through more than visual styling alone.

Color, shape, or contrast should not be the only indication of meaning.

For example:

```rust
badge(BadgeType::Error, "Error")
```

contains the semantic word `Error` rather than relying exclusively on an error color.

When additional accessible context is required, Beverly can expose the appropriate semantic/accessibility configuration.

The general principle remains:

> **Accessible by construction, not accessible by cleanup.**

---

## Badge vs Text

The distinction between `text()` and `badge()` should be simple.

```rust
text("Active")
```

means:

> Present this text.

```rust
badge("Active")
```

means:

> Present this text using Badge presentation.

The content remains text.

The difference is presentation.

A Badge is therefore not a second text system. It is a convenient, opinionated presentation of the existing one.

---

## Badge vs Alert

A Badge and an Alert may both communicate status, but they serve different purposes.

| Primitive     | Purpose                                     |
| ------------- | ------------------------------------------- |
| `text()`      | General text                                |
| `badge()`     | Compact contextual text                     |
| `alert(type)` | Prominent informational/status presentation |
| `button()`    | User interaction                            |

A Badge is appropriate for:

```text
Active
Pending
Beta
New
3
Admin
```

An Alert is appropriate for communicating something that deserves a larger, more prominent surface:

```text
Your profile was updated successfully.
```

The distinction is primarily presentation and context.

Neither primitive should become responsible for application state.

---

## The Principle

Badge demonstrates an important Beverly design principle:

**Framework defaults should make common cases easy without preventing applications from expressing their own domain.**

Beverly can provide:

```rust
BadgeType::Success
BadgeType::Warning
BadgeType::Error
BadgeType::Info
```

because these are useful, familiar defaults.

But an application can define:

```rust
DeploymentStatus
ShipmentStatus
AccountTier
PatientStatus
TransactionState
```

or any other type that accurately represents its own data.

The framework does not need to know what those things mean.

It only needs to make them easy to present.

> **The framework provides the primitives. Your application provides the vocabulary.**

This is particularly important for AI-generated applications. An agent should be able to see:

```rust
DeploymentStatus::RolledBack
```

and understand that it is a domain concept, rather than trying to infer meaning from a generic collection of UI strings and colors.

Strong types make the application's vocabulary explicit to both humans and machines.

---

## Why Badge Is Simple

There is a temptation to make every UI primitive responsible for state, business logic, events, data fetching, permissions, and application workflows.

Beverly intentionally avoids that.

A Badge does one thing well:

**It presents a small piece of information with a recognizable visual treatment.**

If the application needs state, the Model owns it.

If the application needs an application action, the Controller connects it.

If the application needs an Event, the Event defines it.

If the application needs a visual representation, the View provides it.

The Badge does not need to know about the rest.

> **A Badge is not a system. It is a primitive.**

That simplicity makes it reusable, predictable, easy for humans to understand, and easy for AI systems to generate correctly.

## Summary

The Badge follows Beverly's broader architecture:

- **Text** provides the content.
- **Badge** provides the presentation.
- **BadgeType** provides useful default presentation vocabulary.
- **Application types** can define domain-specific badge semantics.
- **Model** owns the underlying state.
- **Events** describe application activity.
- **Controller** connects application behavior.
- **Composition** determines where the Badge appears.

The common case stays simple:

```rust
badge(BadgeType::Success, localize("user.active"))
```

But the framework never requires the application to stop there.

Your application can define its own vocabulary when the domain demands it.

> **Use Beverly's defaults when they fit your data. Define your own types when your data deserves better names.**
