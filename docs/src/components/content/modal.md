# Modal

## Live Demo

<iframe src="../../assets/demos/index.html?demo=modal" title="Interactive Beverly modal demo" loading="lazy" allow="fullscreen" style="display: block; width: 100%; height: min(820px, 90vh); min-height: 580px; border: 0;"></iframe>

<a href="../../assets/demos/index.html?demo=modal" target="_blank" rel="noopener">Open the demo in a separate tab</a>.

## Header, Body, and Footer

`UiElement::modal(...)` implements `.header(...)`, `.body(...)`, and
`.footer(...)` as top, middle, and bottom regions inside the modal surface.
Each accepts a single element or an iterable, repeated calls append content,
and sections render in that order regardless of builder-call order.

```rust
use beverly::components::modal::{BasicModalContent, ModalStyle};
use beverly::prelude::*;

UiElement::modal(
    Modal::new(),
    ModalStyle::default(),
    BasicModalContent::new("Account", ""),
)
.header(text("Edit account details"))
.body(UiElement::input("Email").label("Email address"))
.footer(button().text("Save"))
```

The built-in dialog title and close button stay above custom sections. The
body grows into available surface height, leaving footer content at the
bottom. Custom sections replace the basic body and dismiss-button content;
without sections, `BasicModalContent` supplies the title, body, and dismiss
button. Sections support nested, resource-backed `UiElement` composition.
For specialized construction, `spawn_modal_with_surface(...)` remains available.

A Modal is a temporary surface that appears above the application's current interface.

It is a specialized container for content that requires the user's attention without navigating away from the current application context. A Modal can contain forms, confirmations, settings, information, workflows, or other composed Beverly components.

Like a Card, a Modal does not own application data or business logic. Its responsibility is to present temporary content, manage its UI interaction state, and expose UI Events. The application decides when a Modal should appear and what its interaction means.

> **The Modal controls the surface. The application controls the state.**

## Basic Modal

A Modal contains composed content just like a Card.

```rust id="m7q4bx"
modal()
    .children([
        text(localize("user.create")),
        text(localize("user.create.description")),
    ])
```

A more complete Modal might contain a header, body, and footer:

```rust id="p8r3kc"
modal()
    .header(
        text(localize("user.create"))
    )
    .body([
        text(localize("user.create.description")),
        input()
            .label(localize("user.name.label"))
            .aria(localize("user.name.description"))
            .bind(User::name),
    ])
    .footer(
        button().text(localize("user.create"))
    )
```

The Modal provides the temporary surface. Its children provide the actual interface.

There is no separate composition system for Modal. It uses the same `children()` model as the rest of Beverly.

## Modal as a Specialized Card

Conceptually, a Modal is a Card with additional behavior.

A Card provides:

- containment
- composition
- styling
- state-specific presentation

A Modal adds:

- visibility
- temporary presentation
- focus management
- dismissal behavior
- modal interaction Events
- optional backdrop
- accessibility semantics appropriate to a dialog

This means Modal should not introduce an entirely different UI model. It extends a familiar primitive with the behavior required for a temporary surface.

> **A Modal is a Card with a lifecycle.**

## Visibility

The application state determines whether a Modal is visible.

```rust id="w5k9td"
modal()
    .visible(User::is_create_modal_open)
    .children([
        text(localize("user.create")),
        input()
            .label(localize("user.name.label"))
            .aria(localize("user.name.description"))
            .bind(User::name),
    ])
```

The Modal does not decide why it is open. It simply reflects the state supplied by the application.

For example, the Model might contain:

```rust id="j3r8qm"
struct User {
    create_modal_open: bool,
    name: String,
}
```

The application can then change that state through its normal Event and Controller architecture.

The important boundary is:

```text
Application State
       ↓
Modal visibility
       ↓
Modal presentation
```

The Modal is therefore not a second state-management system.

## Opening a Modal

Opening a Modal is application behavior, not a special Modal command.

A Button might produce a UI Event:

```rust id="q6v2ns"
button().text(localize("user.create"))
    .on("click", Ui::CreateButton::Clicked)
```

The application can then connect that Event to a Command:

```rust id="c8m4ya"
Ui::CreateButton::Clicked
    → Command::User::OpenCreate
```

The Controller handles the relationship between the UI Event and the application action.

The Modal itself does not need to know that a Button opened it.

This allows the same Modal to be opened by a Button, keyboard shortcut, command palette, agent, API-driven state change, or any other application mechanism.

> **The Modal does not open itself. Application state makes it visible.**

## Closing a Modal

A Modal can expose UI Events for dismissal.

```rust id="n4x7pz"
modal()
    .on("close", Ui::CreateModal::CloseRequested)
```

The Event describes what happened to the UI. It does not directly mutate application state.

The application can translate that Event into a Command:

```rust id="r9k3wf"
Ui::CreateModal::CloseRequested
    → Command::User::CloseCreate
```

The Controller then performs the appropriate application action, which updates the Model.

The resulting flow is:

```text
User interaction
      ↓
UI Event
      ↓
Controller
      ↓
Command
      ↓
Model
      ↓
Modal visibility
```

This keeps opening and closing consistent with the rest of Beverly's architecture.

## Dismissal

A Modal may support several UI-level dismissal interactions.

For example:

```rust id="y2c8hs"
modal()
    .on("close", Ui::CreateModal::CloseRequested)
    .on("escape", Ui::CreateModal::EscapePressed)
    .on("backdrop_click", Ui::CreateModal::BackdropClicked)
```

These are UI Events. The application decides whether each one should actually close the Modal.

For example, an application might allow Escape to dismiss an informational Modal but require an explicit decision before dismissing a destructive workflow.

The Modal should not embed that business rule.

The component reports:

```text
Escape happened.
Backdrop was clicked.
Close was requested.
```

The application decides:

```text
Should this Modal close?
```

This distinction becomes particularly important for forms and destructive operations.

## Modal Interaction Events

Modal interaction can be expressed through abstract UI Events.

```rust id="k7m5qc"
modal()
    .on("open", Ui::CreateModal::Opened)
    .on("close", Ui::CreateModal::Closed)
    .on("escape", Ui::CreateModal::EscapePressed)
    .on("backdrop_click", Ui::CreateModal::BackdropClicked)
```

These Events describe the Modal's lifecycle and interaction.

They should not be confused with application Commands such as:

```rust id="x3p8vd"
Command::User::Create
Command::User::CloseCreate
Command::User::CancelCreate
```

The UI Events describe what happened to the interface. The Commands describe what the application should do.

This makes the Modal useful outside any particular domain.

> **Events describe the interface. Commands describe the application.**

## Modal Content

Modal content is ordinary Beverly composition.

```rust id="b6r2wm"
modal()
    .header([
        icon("user"),
        text(localize("user.create")),
    ])
    .body([
        input()
            .label(localize("user.name.label"))
            .aria(localize("user.name.description"))
            .bind(User::name),

        input()
            .label(localize("user.email.label"))
            .aria(localize("user.email.description"))
            .bind(User::email),
    ])
    .footer([
        button().text(localize("user.cancel"))
            .on("click", Command::User::CancelCreate),

        button().text(localize("user.create"))
            .on("click", Command::User::Create),
    ])
```

The Modal does not need to know that these controls represent user creation.

It provides the surface. The components inside it provide the interface.

## Forms Inside Modals

A Modal is often useful for temporary workflows such as creating or editing a record.

The Form remains responsible for collecting input. The Model remains responsible for application state and validation.

```rust
let create_user_form = UiElement::form(Form::post("/api/users"))
    .children([
        UiElement::input("Name")
            .name("name")
            .bind(User::name),
        UiElement::input("Email")
            .name("email")
            .bind(User::email),
    ])
    .submitting(|| [spinner(), text("Creating user…")]);
```

Place the resulting Form in the Modal's body. Its generated submit button emits
`FormSubmitted`; the app handles the request and updates `FormStatusState` on
the Form entity. The Modal does not become the Form, and the Form does not
become the Model. See [Submit](../forms/submit.md).

Each component retains its responsibility.

```text
Modal
 └── Form
      ├── Input
      ├── Input
      └── Submit Button
```

The Modal provides the temporary surface. The Form collects input. Bindings connect fields to the Model. Commands initiate application behavior.

## Focus

A Modal changes the user's interaction context, so focus is part of its UI behavior.

When a Modal opens, the application should be able to establish an appropriate focus target. When it closes, focus should return to the appropriate location in the underlying interface.

A Modal can express this through its configuration:

```rust id="v5n8rx"
modal()
    .focus(User::create_modal_focus_target)
    .children([
        text(localize("user.create")),
        input()
            .label(localize("user.name.label"))
            .aria(localize("user.name.description"))
            .bind(User::name),
    ])
```

The exact focus target is an application decision. Beverly's responsibility is to provide the mechanism for managing focus within the temporary surface.

This is especially important for keyboard users because opening a Modal should establish a predictable interaction context rather than leaving focus behind the overlay.

## Focus Containment

While a Modal is active, keyboard interaction should remain within the Modal until it is dismissed.

This prevents keyboard navigation from unexpectedly moving into the obscured application behind the Modal.

Conceptually:

```text
Underlying Application
        ↓
     inactive
        ↑
      Modal
        ↓
   active focus
```

The Modal therefore manages the UI mechanics of its temporary interaction boundary without owning the application's state.

## Backdrop

A Modal can provide a backdrop separating the temporary surface from the underlying application.

```rust id="q9w4kc"
modal()
    .backdrop(true)
    .children([
        text(localize("user.create")),
    ])
```

The backdrop is presentation. Whether clicking it dismisses the Modal is interaction policy.

Those can therefore remain separate:

```rust id="e6t2mv"
modal()
    .backdrop(true)
    .on("backdrop_click", Ui::CreateModal::BackdropClicked)
```

The Modal reports the interaction. The application decides what to do with it.

This distinction prevents presentation choices from becoming hidden application behavior.

## Modal State

Modal styles can vary with interaction state, while its success/submitting
callbacks add separate content beneath the Modal surface. The app supplies the
`FormStatusState`; the Modal doesn't infer it from a request or predicate.

```rust id="s3p7yd"
UiElement::modal(Modal::new(), ModalStyle::default(), BasicModalContent::new("Create user", ""))
    .hover(|style| style.opacity(0.96))
    .focused(|style| style.outline_width(2).outline_color("#2563eb"))
    .submitting_style(|style| style.opacity(0.72))
    .success_style(|style| style.border_color("#16a34a"))
    .submitting(|| [
        spinner(),
        text(localize("user.creating")),
    ])
    .success(|| text(localize("user.created")))
```

Set `FormStatusState::submitting()`, `success()`, or `idle()` on the spawned
Modal entity from application code. `submitting`/`success` callbacks append
supplementary content below the existing modal content; state-style callbacks
change only the listed style properties. For method details and precedence,
see [Styling DSL](../../foundations/styling.md) and
[Form status callbacks](../forms/overview.md#pending-and-success-presentations).

## Error State

A Modal can present an error without implementing the operation that produced it.

```rust id="u7n3qx"
modal()
    .error(|errors| text(errors.join("\n")))
```

The underlying error should come from application state and should normally be localized at presentation time.

The Modal should not contain domain validation such as:

```text
email is invalid
name is required
user already exists
```

Those rules belong to the Model or application layer.

The Modal presents the resulting state.

## Success State

The same principle applies to successful operations.

```rust id="m4k8zp"
modal()
    .success(|| [
        icon("check"),
        text(localize("user.created")),
    ])
```

The Modal does not decide whether the operation succeeded. It presents the success state supplied by the application.

This makes the same Modal reusable for many different workflows.

## Preventing Accidental Dismissal

Some Modal workflows should not disappear simply because the user interacts with the backdrop or presses Escape.

The Modal can expose the interaction, while the application decides what to do:

```rust id="r6v2kc"
modal()
    .on("escape", Ui::CreateModal::EscapePressed)
    .on("backdrop_click", Ui::CreateModal::BackdropClicked)
```

The Controller can then apply the application's policy.

For example, if a Form contains unsaved changes, the application might respond to a close request by showing a confirmation Modal rather than immediately closing the current one.

The Modal itself does not need to understand "unsaved changes." That information belongs to application state.

## Nested Modals

Beverly should make it possible to build layered workflows, but nested Modals should remain an explicit application decision rather than an implicit behavior of the component.

A confirmation surface can be composed independently:

```rust id="w8q5mz"
modal()
    .header(text(localize("user.discard_changes")))
    .body(text(localize("user.discard_changes.description")))
    .footer([
        button().text(localize("user.keep_editing")),

        button().text(localize("user.discard")),
    ])
```

The application determines when that Modal appears and what each interaction means.

The component remains the same primitive regardless of the workflow.

## Accessibility

A Modal is more than a visual box. When presented as a dialog, it needs an explicit accessible identity and description.

```rust id="n6r3vx"
modal()
    .label(localize("user.create.label"))
    .aria(localize("user.create.description"))
    .children([
        text(localize("user.create")),
        input()
            .label(localize("user.name.label"))
            .aria(localize("user.name.description"))
            .bind(User::name),
    ])
```

The Modal's accessible semantics describe the dialog itself. Its children retain their own accessibility declarations.

For example, the Input still needs:

```rust id="p3w7hk"
input()
    .label(localize("user.name.label"))
    .aria(localize("user.name.description"))
    .bind(User::name)
```

Accessibility is therefore compositional. The Modal provides dialog-level semantics, while each interactive child provides its own semantics.

A complete Modal should also provide predictable keyboard behavior, focus containment, and a clear mechanism for dismissal where appropriate.

> **Accessibility belongs to the interaction boundary, not just the pixels.**

## Styling

Modal styling follows the same fluent configuration model as Card.

```rust id="y4k8sq"
modal()
    .padding(24)
    .radius(16)
    .shadow(12)
    .max_width(640)
    .children([
        text(localize("user.create")),
    ])
```

The visual treatment can be customized without changing the Modal's underlying behavior.

This allows a design system to establish consistent Modal primitives while individual applications compose their own content.

## Complete Example

A complete create-user Modal brings the pieces together:

```rust id="f9m3qx"
modal()
    .label(localize("user.create.label"))
    .aria(localize("user.create.description"))
    .visible(User::is_create_modal_open)
    .padding(24)
    .radius(16)
    .backdrop(true)
    .header([
        icon("user"),
        text(localize("user.create")),
    ])
    .body([
        input()
            .label(localize("user.name.label"))
            .aria(localize("user.name.description"))
            .bind(User::name),

        input()
            .label(localize("user.email.label"))
            .aria(localize("user.email.description"))
            .bind(User::email),
    ])
    .footer([
        button().text(localize("user.cancel"))
            .on("click", Command::User::CancelCreate),

        button().text(localize("user.create"))
            .submitting_style(|style| style.opacity(0.5))
            .on("click", Command::User::Create),
    ])
    .submitting(|| [spinner(), text(localize("user.creating"))])
    .error(|errors| text(errors.join("\n")))
    .on("escape", Ui::CreateModal::EscapePressed)
    .on("backdrop_click", Ui::CreateModal::BackdropClicked)
```

There are several independent responsibilities here, but they remain easy to identify.

The Modal owns its presentation and temporary interaction boundary. Inputs bind
to Model state; Buttons produce UI interactions; Commands represent application
operations. The application sets `FormStatusState` on the spawned Modal entity
after deciding whether the operation is pending or complete. Callbacks render
supplementary content; they do not evaluate application predicates themselves.

Nothing requires the Modal to understand the User domain.

## The Modal Contract

The Modal has a deliberately small conceptual contract:

**Create. Configure. Compose. Present. Contain interaction. Emit UI Events.**

It provides:

- temporary visual presentation
- composition through `children()`
- semantic sections such as `header`, `body`, and `footer`
- visibility
- backdrop presentation
- focus management
- keyboard interaction
- dismissal Events
- state-specific presentation
- dialog accessibility semantics

It does not own:

- application data
- business rules
- domain validation
- application Commands
- database operations
- API operations
- agent behavior

Those concerns remain in the application's Model, Events, and Controller.

The result is a Modal that behaves like the rest of Beverly: familiar Rust, ordinary composition, explicit Events, and clear ownership.

> **The Modal is a temporary interface, not a second application state system.**
