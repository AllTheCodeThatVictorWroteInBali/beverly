# Toast

A Toast is a transient notification presented as a floating Card.

It is a composite component built from familiar Beverly primitives: a Card, content, a close Button, positioning, animation, and a timed lifecycle.

A Toast appears with an entrance animation, remains visible for a configurable period, and then fades away.

It can also be dismissed immediately by the user.

**A Toast is a Card with a lifecycle.**

## Basic Usage

Create a Toast with `toast()` and choose its type:

```rust id="p7m3q8"
toast(ToastType::Success)
    .children([
        text(localize("user.saved")),
    ])
```

Other predefined types follow the same semantic vocabulary as Alert:

```rust id="v4k8n2"
toast(ToastType::Info)
    .children([
        text(localize("user.updated")),
    ])
```

```rust id="m6q2r9"
toast(ToastType::Warning)
    .children([
        text(localize("user.unsaved_changes")),
    ])
```

```rust id="c8w3p5"
toast(ToastType::Error)
    .children([
        text(localize("user.save_failed")),
    ])
```

The type determines the Toast's presentation.

It does not create application state.

**The type describes the presentation. The application owns the meaning.**

## Toast vs Alert

Toast and Alert share a semantic vocabulary, but they serve different presentation models.

| Alert                          | Toast                           |
| ------------------------------ | ------------------------------- |
| Persistent/in-context          | Transient/floating              |
| Remains visible                | Automatically dismisses         |
| Communicates within the layout | Appears above the layout        |
| Primarily contextual           | Primarily immediate feedback    |
| No inherent entrance lifecycle | Enters and exits with animation |
| Card presentation              | Composite animated Card         |

An Alert might communicate:

```rust id="j3r7v1"
alert(AlertType::Error)
    .children([
        text(localize("user.save_failed")),
    ])
```

A Toast communicates the same kind of information differently:

```rust id="q5m8n2"
toast(ToastType::Error)
    .children([
        text(localize("user.save_failed")),
    ])
```

The semantic type is similar. The presentation and lifecycle are different.

**Alert communicates in context. Toast communicates temporarily above the interface.**

## Content and Named Sections

`children()` is the general composition primitive.

```rust id="r8p4m6"
toast(ToastType::Success)
    .children([
        text(localize("user.saved")),
        text(localize("user.saved_description")),
    ])
```

For more structured content, Toast provides three named sections:

- `header`
- `body`
- `footer`

These sections are semantic conveniences, not a replacement for `children()`.

A simple Toast can use `children()`:

```rust id="b2v9q5"
toast(ToastType::Info)
    .children(
        text(localize("user.profile_updated"))
    )
```

A structured Toast can use named sections:

```rust id="k7m2q8"
toast(ToastType::Success)
    .header(
        text(localize("user.saved"))
    )
    .body(
        text(localize("user.saved_description"))
    )
    .footer(
        text(localize("user.saved_time"))
    )
```

Each named section can accept one child or many children:

```rust id="f4r9v3"
toast(ToastType::Success)
    .header([
        icon("check"),
        text(localize("user.saved")),
    ])
```

```rust id="u8p3m6"
toast(ToastType::Success)
    .body([
        text(localize("user.saved")),
        text(localize("user.saved_description")),
    ])
```

```rust id="w5n1q7"
toast(ToastType::Success)
    .footer([
        text(localize("user.saved_time")),
        text(localize("user.saved_location")),
    ])
```

The named sections provide useful structure without introducing another composition model.

**`children()` is the general primitive. `header`, `body`, and `footer` add useful semantics.**

## Header

The Header is useful for a short title or primary message.

```rust id="e2m8r4"
toast(ToastType::Error)
    .header(
        text(localize("user.save_failed"))
    )
```

It can contain multiple components:

```rust id="a9q3v6"
toast(ToastType::Error)
    .header([
        icon("error"),
        text(localize("user.save_failed")),
    ])
```

The Header does not own the error state. It simply presents content.

## Body

The Body is useful for supporting information.

```rust id="g7m2p5"
toast(ToastType::Error)
    .body(
        text(localize("user.save_failed_description"))
    )
```

It can contain arbitrary components:

```rust id="q4n8m2"
toast(ToastType::Error)
    .body([
        text(localize("user.save_failed_description")),
        text(localize("user.retry_available")),
    ])
```

The Toast does not interpret the content.

**The application provides the meaning. The Toast provides the presentation.**

## Footer

The Footer is useful for secondary actions or supporting information.

```rust id="y6p1r7"
toast(ToastType::Info)
    .footer([
        button(localize("user.undo"))
            .label(localize("user.undo.label"))
            .aria(localize("user.undo.description"))
            .on("click", Command::User::UndoDelete),
    ])
```

The Footer can contain multiple components:

```rust id="w5n1q7"
toast(ToastType::Info)
    .footer([
        text(localize("user.deleted")),
        button(localize("user.undo"))
            .label(localize("user.undo.label"))
            .aria(localize("user.undo.description"))
            .on("click", Command::User::UndoDelete),
    ])
```

The Button remains an ordinary Beverly Button. The Toast does not redefine its behavior.

## Close Button

A Toast includes a close control so the user can dismiss it before its timer expires.

The close control is part of the Toast's presentation rather than application content.

Its accessible name should clearly communicate its purpose:

```rust id="s4r9m1"
toast(ToastType::Info)
    .close_label(localize("toast.dismiss"))
    .children([
        text(localize("user.profile_updated")),
    ])
```

The user can therefore either wait for the Toast to expire or dismiss it immediately.

The application does not need to construct a separate close-button architecture for every Toast.

**The Toast manages its presentation lifecycle. The application decides when the Toast exists.**

## Position

A Toast can be positioned at the top or bottom of the interface and aligned left, center, or right.

```rust id="u7p3k5"
toast(ToastType::Success)
    .position(ToastPosition::TopRight)
    .children([
        text(localize("user.saved")),
    ])
```

Other positions include:

```rust id="e2m8r4"
toast(ToastType::Info)
    .position(ToastPosition::TopLeft)
```

```rust id="a9q3v6"
toast(ToastType::Info)
    .position(ToastPosition::TopCenter)
```

```rust id="g7m2p5"
toast(ToastType::Info)
    .position(ToastPosition::BottomLeft)
```

```rust id="q4n8m2"
toast(ToastType::Info)
    .position(ToastPosition::BottomCenter)
```

```rust id="y6p1r7"
toast(ToastType::Info)
    .position(ToastPosition::BottomRight)
```

The position is a property of the Toast's presentation, not its application meaning.

**Same Toast. Different place in the interface.**

## Duration

A Toast remains visible for a configurable period before beginning its exit animation.

```rust id="w5n1q7"
toast(ToastType::Success)
    .duration(5000)
    .children([
        text(localize("user.saved")),
    ])
```

The duration is expressed in milliseconds.

A longer duration can be used when the content requires more time to read:

```rust id="r8p4m6"
toast(ToastType::Info)
    .duration(8000)
    .children([
        text(localize("user.import_complete")),
    ])
```

The duration belongs to the Toast's presentation lifecycle, not the application's Model.

## Entrance and Exit Animation

Animation is a fundamental part of the Toast primitive.

When the Toast appears, it enters the interface using its entrance animation.

When its duration expires, it exits before being removed.

Conceptually:

```text id="b2v9q5"
Create
  ↓
Entering
  ↓
Visible
  ↓
Timer / Dismiss
  ↓
Exiting
  ↓
Removed
```

The animation can naturally reflect the Toast's position.

A Toast entering from the bottom can move upward into place, while a Toast entering from the top can move downward.

The exact animation remains an implementation detail of the Toast primitive.

**The Toast owns the animation. The application does not have to orchestrate it.**

## Lifecycle

The Toast has presentation state internally:

- opening
- visible
- closing
- closed

The application does not need to manage these states manually.

This is UI state, not application state.

The Model continues to own application state while the Toast manages its own temporary presentation lifecycle.

**Presentation state belongs to the component. Application state belongs to the Model.**

## Application Events

A Toast can be triggered by an application Event:

```rust id="k7m2q8"
controller! {
    User::Saved => [
        Toast::Success,
    ],
}
```

The Event describes what happened. The Toast determines how that fact is presented.

The Toast does not need to know why the application produced it.

**Events describe facts. Toast describes presentation.**

## Data-Driven Toasts

Toast content can come directly from application data:

```rust id="f4r9v3"
toast(notification.toast_type())
    .children([
        text(notification.message()),
    ])
```

An application can also map its own domain types to Toast presentation types:

```rust id="u8p3m6"
enum DeploymentStatus {
    Running,
    Failed,
    RolledBack,
}
```

Then ordinary Rust determines how those domain states should be presented:

```rust id="w5n1q7"
fn toast_type(status: DeploymentStatus) -> ToastType {
    match status {
        DeploymentStatus::Running => ToastType::Success,
        DeploymentStatus::Failed => ToastType::Error,
        DeploymentStatus::RolledBack => ToastType::Warning,
    }
}
```

The framework provides useful defaults without defining the application's domain vocabulary.

**Your data, your type system.**

## Higher-Level Toasts

Building a higher-level Toast is just an ordinary Rust function that returns a Toast.

```rust id="e2m8r4"
fn saved_toast() -> Toast {
    toast(ToastType::Success)
        .header(
            text(localize("user.saved"))
        )
        .body(
            text(localize("user.saved_description"))
        )
}
```

Use it like any other component:

```rust id="a9q3v6"
saved_toast()
```

There is no nested Toast component or special extension system.

**Higher-level components are ordinary Rust functions that return lower-level components.**

## Accessibility

A Toast communicates information that may appear outside the user's current reading context, so its announcement behavior is part of the component's implementation.

The close control must be keyboard accessible and have an accessible name.

```rust id="g7m2p5"
toast(ToastType::Success)
    .close_label(localize("toast.dismiss"))
    .children([
        text(localize("user.saved")),
    ])
```

The Toast should not depend on color or animation alone to communicate meaning.

The semantic type and accessible announcement provide the meaning; visual treatment reinforces it.

**Accessible by construction, not accessible by cleanup.**

## What Toast Does Not Do

Toast does not:

- own application state
- replace the Model
- become a notification database
- manage application business logic
- define domain types
- replace Alert
- replace Modal
- require a separate state-management system

It owns its presentation lifecycle:

- positioning
- entrance animation
- visibility duration
- exit animation
- dismissal
- close-button presentation
- content composition

The application owns the reason the Toast exists.

## Alert, Toast, and Modal

These components can share underlying primitives while serving different purposes.

| Component     | Purpose                                         |
| ------------- | ----------------------------------------------- |
| `alert(type)` | Persistent communication within the interface   |
| `toast(type)` | Transient floating feedback                     |
| `modal()`     | Temporary surface requiring focused interaction |

The distinction is primarily lifecycle and presentation.

**Alert communicates. Toast appears and fades. Modal interrupts.**

## Design Principle

Toast is deliberately a composite component rather than a new UI system.

Conceptually, it combines:

```text id="q4n8m2"
Toast
├── Card
├── Header / Body / Footer
├── Close Button
├── Position
├── Entrance Animation
├── Duration
└── Exit Animation
```

Each piece already exists conceptually in Beverly.

Toast simply combines them into a useful, reusable pattern.

**The Toast is a specialized Card with a lifecycle.**

Create it with a semantic type. Configure its position and duration. Compose its content with `children()` or `header`, `body`, and `footer`. Let Beverly manage the presentation lifecycle.

**Create. Configure. Compose. Animate. Dismiss.**
