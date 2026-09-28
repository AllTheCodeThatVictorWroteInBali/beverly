# Building Forms

Once the Model, Events, and Controller are in place, building a form should be straightforward.

The Form is a component. Configure it with chained function calls, compose its contents with `.children()`, bind fields to the Model, and declare what should happen when the user submits or resets it.

The basic pattern is:

> **Create. Configure. Compose. Bind. Declare state. Emit intent.**

## Styling a Form

Forms use the same fluent component API as everything else in Beverly.

```rust
form()
    .padding(24)
    .radius(12)
    .gap(16)
    .width(400)
```

Because styling is just component configuration, it can be composed naturally with the rest of the form.

## Adding Form Elements

`.children()` defines the contents of the form.

```rust
form()
    .children([
        input()
            .label("Name")
            .bind(User::name),

        input()
            .label("Email")
            .bind(User::email),

        input()
            .label("Age")
            .bind(User::age),

        checkbox()
            .label("Active")
            .bind(User::active),

        submit("Create User"),
    ])
```

A form can contain any Beverly component. Components can themselves contain other components, so there is no special form-specific composition system.

## Binding Fields

`.bind()` connects a form element to a Model field.

```rust
input()
    .label("Email")
    .bind(User::email)
```

The binding handles the translation between interface input and typed application state.

Conceptually:

```text
Model → Binding → Input
Input → Binding → Setter → Model
```

The Model remains the source of truth.

When the Model changes, the bound input reflects the new value. When the user changes the input, the binding transforms the input into the appropriate type and invokes the Model's setter.

There is no need to manually synchronize the input with the Model.

## Field Errors Come From the Binding

Field-level errors are not independently managed by the UI.

They are **derived from the form binding**.

When a setter rejects a value, the binding receives that error and propagates it back to the field that produced it.

For example:

```rust
impl User {
    fn set_email(&mut self, email: String) -> Result<(), UserError> {
        let email = email.trim().to_lowercase();

        if !is_valid_email(&email) {
            return Err(UserError::InvalidEmail);
        }

        self.email = email;

        Ok(())
    }
}
```

The form does not need to understand what makes an email invalid.

The setter does.

If the setter returns an error, the binding associates that error with the corresponding field:

```rust
input()
    .label("Email")
    .bind(User::email)
    .error(
        self.children([
            text("Invalid email address.")
        ])
    )
```

The important distinction is that the `.error(...)` presentation belongs to the **field**, while the decision that an error exists belongs to the **Model's state transition**.

This keeps validation in one place.

> **The field displays the error. The Model decides whether the error exists.**

## Validation Timing Belongs to the Model

The Model is also responsible for deciding **when** a field should be considered invalid.

For example, an application may not want to display an error for an untouched field.

That distinction is application state.

If the application needs to know whether a field has been touched, that information must be represented in the Model rather than hidden inside the UI.

Conceptually:

```rust
if touched {
    validate(value)
} else {
    Ok(())
}
```

The exact implementation can vary, but the important principle does not:

> **If validation behavior depends on state, that state belongs to the Model.**

This means the same validation behavior applies regardless of where the state transition originated.

A human editing a form, an agent issuing a Command, an API request, or another application component all encounter the same Model rules.

The Form is simply one client of those rules.

## Form State Hooks

Forms can also declare presentations for their own state.

For example:

```rust
form()
    .error(
        self.children([
            text("Unable to create user.")
        ])
    )
```

Or:

```rust
form()
    .success(
        self.children([
            text("User created successfully.")
        ])
    )
```

And:

```rust
form()
    .submitting(
        self.opacity(0.5)
    )
```

These hooks are important to understand:

**`self` refers back to the Form being configured.**

The hook does not attach a visual change to the Event that caused the state transition. It declares what the **Form itself should present when it is in that state**.

For example:

```rust
form()
    .error(
        self
            .color("red")
            .children([
                text("Unable to create user.")
            ])
    )
```

The Event may cause the application to enter an error state, but the presentation is still owned by the Form.

This creates a clean separation:

```text
Event / Model State
        ↓
      Form
        ↓
   State-specific
   presentation
```

The Event causes state to change.

The Form decides how that state is presented.

## Global Errors

`.error(...)` defines what the Form should display when it has a global error.

```rust
form()
    .error(
        self.children([
            text("Unable to create user.")
        ])
    )
```

The content is only presented while the Form has an applicable error state.

It can contain arbitrary components:

```rust
form()
    .error(
        self
            .color("red")
            .children([
                text("Unable to create user."),
                text("Please try again."),
            ])
    )
```

The Form does not need a special error-template system. Error presentation is simply another component composition.

## Success State

Success works the same way.

```rust
form()
    .success(
        self.children([
            text("User created successfully.")
        ])
    )
```

The Form owns the presentation of its success state.

The Controller and Model determine what happened; the Form determines how that state is presented.

## Submitting State

A Form can also declare how it should appear while an operation is pending.

```rust
form()
    .submitting(
        self.opacity(0.5)
    )
```

Or:

```rust
form()
    .submitting(
        self.children([
            spinner(),
            text("Creating user..."),
        ])
    )
```

Again, `self` refers to the Form.

This is not an Event-specific visual effect. It is the Form's presentation while the Form is submitting.

The same state-specific hooks can be used by form elements where appropriate:

```rust
input()
    .submitting(
        self.opacity(0.5)
    )
```

The API remains consistent across components.

> **State determines when. Components determine what.**

## Field-Level Error Presentation

Field errors use the same component API.

```rust
input()
    .label("Email")
    .bind(User::email)
    .error(
        self
            .color("red")
            .children([
                text("Invalid email address.")
            ])
    )
```

But unlike a global Form error, the field error is derived from that field's binding.

The chain is:

```text
Input
  ↓
Binding
  ↓
Model Setter
  ↓
Result
  ↓
Field Error
```

If the setter succeeds, there is no field error.

If the setter returns an error, the binding associates that error with the field and the field can present its `.error(...)` content.

The UI does not need to duplicate the Model's validation logic.

## Submitting the Form

Submission is an Event.

```rust
form()
    .children([
        input()
            .label("Name")
            .bind(User::name),

        input()
            .label("Email")
            .bind(User::email),

        submit("Create User"),
    ])
    .on("submit", Command::User::Create)
```

The Form emits intent.

The Controller decides what that intent means.

```rust
controller! {
    User::Create => Api::users::create,
}
```

The resulting flow is:

```text
Form → Command → Controller → API → Fact
```

The Form does not need to know how the API works.

## Resetting a Form

Reset follows the same architecture.

The Form requests a reset:

```rust
form()
    .on("reset", User::Reset)
```

The Model handles the reset.

The bindings then reflect the new Model state back into the Form.

```text
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

The Form requests the reset. The Model performs the reset. The bindings make the UI follow the state.

This avoids creating a second, hidden source of truth inside the Form.

## Accessibility

Form accessibility is declared as part of the component definition.

Required form elements must have labels:

```rust
input()
    .label("Email")
    .bind(User::email)
```

Additional accessible descriptions can be provided with `.aria(...)`:

```rust
input()
    .label("Email")
    .aria("Enter the email address associated with your account.")
    .bind(User::email)
```

`.label()` names the control. `.aria()` provides additional accessible semantics or descriptive information when needed.

Required accessibility metadata is declared up front, allowing Beverly to catch missing requirements during compilation rather than relying entirely on runtime cleanup.

> **Accessible by construction, not accessible by cleanup.**

This makes known, preventable accessibility mistakes harder to express and helps reduce expensive usability, compliance, and legal risk.

## A Complete Form

Putting the pieces together:

```rust
form()
    .padding(24)
    .radius(12)
    .gap(16)
    .width(400)

    .error(
        self
            .color("red")
            .children([
                text("Unable to create user.")
            ])
    )

    .success(
        self.children([
            text("User created successfully.")
        ])
    )

    .submitting(
        self.opacity(0.5)
    )

    .children([
        input()
            .label("Name")
            .bind(User::name),

        input()
            .label("Email")
            .aria("Enter the email address associated with your account.")
            .bind(User::email)
            .error(
                self
                    .color("red")
                    .children([
                        text("Invalid email address.")
                    ])
            ),

        input()
            .label("Age")
            .bind(User::age),

        checkbox()
            .label("Active")
            .bind(User::active),

        submit("Create User"),
    ])

    .on("submit", Command::User::Create)
    .on("reset", User::Reset)
```

There is no form-specific state-management framework hiding underneath this.

The Model owns application state and validation.

The binding connects state to fields.

Events express intent.

The Controller connects intent to actions.

The Form presents the resulting state.

That is the whole system.
