# Submit

> **Current status:** `.bind(...)` examples describe planned model-setter
> integration and are not currently executable against the crate.

`submit()` represents the action that submits a form.

It is intentionally simple.

The Submit control does not know how the application should create a user, save a document, call an API, or otherwise process the form.

It emits intent.

> **Submit expresses intent. The Event carries that intent. The Controller decides what happens next.**

## Basic Submit

A Submit control can be added directly to a Form:

```rust id="h4k8q2"
form()
    .children([
        input()
            .label(localize("user.name.label"))
            .aria(localize("user.name.description"))
            .bind(User::name),

        input()
            .label(localize("user.email.label"))
            .aria(localize("user.email.description"))
            .bind(User::email),

        submit(localize("user.create.submit")),
    ])
```

The Submit control does not directly call an API.

It does not directly mutate the Model.

It does not contain business logic.

It tells the Form that the user wants to submit.

## Connecting Submit to an Event

The Form declares what submission means using `.on()`:

```rust id="q7m3vx"
form()
    .children([
        input()
            .label(localize("user.name.label"))
            .aria(localize("user.name.description"))
            .bind(User::name),

        input()
            .label(localize("user.email.label"))
            .aria(localize("user.email.description"))
            .bind(User::email),

        submit(localize("user.create.submit")),
    ])
    .on("submit", Command::User::Create)
```

The flow is straightforward:

```text id="k5r9tp"
User clicks Submit
       ↓
Form emits submit
       ↓
Command::User::Create
       ↓
Controller
       ↓
Model / API
       ↓
Fact
```

The Submit control does not need to know anything about the rest of the flow.

## Submit Is Intent

This distinction is important.

The Submit control represents:

> **"I want to submit this form."**

It does not represent:

> **"Create this user."**

The latter is application intent and belongs in the Event vocabulary.

For example:

```rust id="v8c2nm"
event! {
    User::Create
    User::Created
    User::CreateFailed
}
```

The same `User::Create` Command can therefore be issued by:

- A human using the Form
- An AI agent
- An API
- Another application component
- An automated workflow

The UI is simply one way of issuing the Command.

> **Humans and agents can use the same application vocabulary.**

## Validation Before Submission

The Form works with the Model's existing validation mechanics.

The Form does not need a separate validation framework.

For example:

```rust id="m3q7yx"
impl User {
    fn set_email(
        &mut self,
        email: String,
    ) -> Result<(), UserError> {
        let email = email.trim().to_lowercase();

        if !is_valid_email(&email) {
            return Err(UserError::InvalidEmail);
        }

        self.email = email;

        Ok(())
    }
}
```

By the time the Submit Command reaches the Controller, the bound Model represents application state governed by the Model's rules.

Submission can then perform additional application-level validation where appropriate.

This creates a useful separation:

- **Field validation** — can this value enter Model state?
- **Form/application validation** — can this collection of state be submitted?
- **Controller policy** — is this action permitted?
- **API/domain processing** — can the requested operation actually be completed?

The Submit control does not need to know any of these rules.

## Submitting State

Submission can be asynchronous.

While the Command is being processed, the Form can present its submitting state:

```rust id="z6p4wk"
form()
    .children([
        // form fields...
        submit(localize("user.create.submit")),
    ])
    .on("submit", Command::User::Create)
    .submitting(
        self.opacity(0.5)
    )
```

Or provide a richer presentation:

```rust id="r2n8vq"
form()
    .submitting(
        self.children([
            spinner(),
            text(localize("user.create.submitting")),
        ])
    )
```

Here, `self` refers to the **Form itself**.

It does not refer to the Event that triggered submission.

The Command changes application state.

The Form observes that state and presents it.

> **State determines when. Components determine what.**

## Global Errors

If submission fails, the Form can present a global error:

```rust id="c7m4xp"
form()
    .error(
        self.children([
            text(|errors: &[String]| errors.join(" "))
        ])
    )
```

The error content is only presented while the Form has an active global error state.

The UI does not need to know which Event produced the error.

The application state determines whether the error exists.

The component determines how that state is presented.

## Success

Successful submission can use the same pattern:

```rust id="n5q8yr"
form()
    .success(
        self.children([
            text(localize("user.create.success")),
        ])
    )
```

Success is state, not a special submission template.

The Form simply declares what it should look like when that state is active.

## Reset After Submission

Resetting follows Beverly's normal state architecture.

The Form requests a reset through an Event.

The Model performs the reset.

The bindings then cause the Form to reflect the new Model state.

```text id="p4x7mn"
Form
 ↓
Reset Event
 ↓
Model resets
 ↓
Bindings observe state
 ↓
Form reflects Model
```

For example:

```rust id="y8k3vq"
form()
    .children([
        // fields...
        submit(localize("user.create.submit")),
    ])
    .on("submit", Command::User::Create)
    .on("reset", User::Reset)
```

The Form does not directly clear its fields.

The Model remains the source of truth.

> **The Form requests the reset. The Model performs the reset. The bindings make the UI follow the state.**

## Styling

Submit follows the same fluent component API:

```rust id="w6r2kp"
submit(localize("user.create.submit"))
    .padding(12)
    .radius(8)
    .width(160)
```

Because Submit is a normal component, it can participate in the same state-specific presentation system:

```rust id="q3m7vx"
submit(localize("user.create.submit"))
    .submitting(
        self.opacity(0.5)
    )
```

The Form can also control the presentation of the entire submission state:

```rust id="k9p4yt"
form()
    .submitting(
        self.opacity(0.5)
    )
```

This gives developers control at either level without creating separate styling APIs.

## Accessibility

Submit is a form element and requires explicit accessible semantics.

```rust id="f8q2mw"
submit(localize("user.create.submit"))
    .label(localize("user.create.submit.label"))
    .aria(localize("user.create.submit.description"))
```

The visible text and accessible semantics can come from the localization layer.

Beverly treats accessibility metadata as part of the component contract rather than something added later.

> **Accessible by construction, not accessible by cleanup.**

## Complete Form

Putting the form primitives together:

```rust id="v6k3rq"
form()
    .padding(24)
    .radius(12)
    .gap(16)
    .width(400)

    .children([
        input()
            .label(localize("user.name.label"))
            .aria(localize("user.name.description"))
            .bind(User::name),

        input()
            .label(localize("user.email.label"))
            .aria(localize("user.email.description"))
            .bind(User::email)
            .error(
                self.children([
                    text(|errors: &[String]| errors.join(" "))
                ])
            ),

        select()
            .label(localize("user.plan.label"))
            .aria(localize("user.plan.description"))
            .bind(User::plan)
            .children([
                option("free")
                    .label(localize("user.plan.free.label")),

                option("pro")
                    .label(localize("user.plan.pro.label")),

                option("enterprise")
                    .label(localize("user.plan.enterprise.label")),
            ]),

        checkbox()
            .label(localize("user.active.label"))
            .aria(localize("user.active.description"))
            .bind(User::active),

        submit(localize("user.create.submit"))
            .label(localize("user.create.submit.label"))
            .aria(localize("user.create.submit.description")),
    ])

    .on("submit", Command::User::Create)
    .on("reset", User::Reset)

    .error(
        self.children([
            text(|errors: &[String]| errors.join(" "))
        ])
    )

    .success(
        self.children([
            text(localize("user.create.success"))
        ])
    )

    .submitting(
        self.opacity(0.5)
    )
```

There is no custom form state-management system.

There are no manual synchronization handlers.

There is no UI-specific validation framework.

There is no API call hidden inside the button.

The pieces each have one responsibility.

**Form** — composes the fields.

**Input / Select / Checkbox** — collect interaction.

**Option** — represents a choice.

**Binding** — connects controls to Model state.

**Model** — owns and validates application state.

**Submit** — expresses submission intent.

**Event** — carries that intent.

**Controller** — connects intent to application actions.

## Submit API

| API                                   | Purpose                                                |
| ------------------------------------- | ------------------------------------------------------ |
| `submit(...)`                         | Creates a Submit control                               |
| `.label(...)`                         | Defines the accessible name                            |
| `.aria(...)`                          | Defines additional accessible semantics or description |
| `.padding(...)`, `.radius(...)`, etc. | Applies styling                                        |
| `.submitting(...)`                    | Defines the pending presentation                       |
| `.on(...)`                            | Connects Events or Commands                            |

## The Form System

With Submit, the core Beverly form primitives now fit together into one consistent model:

> **Form → Elements → Binding → Model → Events → Controller → Application**

Every piece has a clear responsibility.

The result is a form system that is simple enough for a human to understand at a glance, structured enough for an LLM to generate reliably, and powerful enough to connect directly to the rest of the application.

> **Collect. Bind. Validate. Submit. Observe.**
