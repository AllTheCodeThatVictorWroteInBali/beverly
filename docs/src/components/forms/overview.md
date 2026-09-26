# Forms

A Beverly form is a typed interface that transforms user input into application state.

The form handles the **editing experience**. The Model handles the **truth**.

This distinction is important. A form should not become a second state-management system, and validation should not exist only in the UI. The same Model mechanics should work whether a value comes from a human through a form, an AI agent, an API, or another part of the application.

The basic flow is:

```text
Input → Binding → Setter → Model
```

The setter is the authoritative state-transition boundary.

---

## The Model

A form ultimately edits application data, so start with ordinary Rust.

```rust id="on6d0d"
struct User {
    name: String,
    email: String,
    age: u32,
    active: bool,
}
```

Nothing special is required here.

You can implement the accessors yourself:

```rust id="9lh5z6"
impl User {
    fn name(&self) -> &str {
        &self.name
    }

    fn set_name(&mut self, name: String) {
        self.name = name;
    }
}
```

This is intentionally ordinary Rust. In a real application, getters and setters can become repetitive boilerplate, so Beverly can generate the common case with `#[model]`.

```rust id="hzh3pl"
#[model]
struct User {
    name: String,
    email: String,
    age: u32,
    active: bool,
}
```

The macro generates ordinary Rust accessors for the fields.

Conceptually, you get the same thing you would have written yourself:

```rust id="3annbd"
impl User {
    fn email(&self) -> &str {
        &self.email
    }

    fn set_email(&mut self, email: String) {
        self.email = email;
    }
}
```

There is no second model language to learn.

> **The Model is just Rust — and that's exactly the point.**

---

## Implicit and Explicit Setters

Beverly defaults to the obvious behavior.

If a field has no special requirements, Beverly can infer its setter from the field itself:

```rust id="i3vsxe"
input()
    .label("Email")
    .bind(User::email)
```

The generated setter simply updates the field.

When a field needs domain-specific behavior, you can explicitly tell Beverly which setter to use:

```rust id="rehc9s"
#[model]
struct User {
    name: String,

    #[setter = validate_email]
    email: String,

    age: u32,
    active: bool,
}
```

Now changes to `email` go through `validate_email`.

```rust id="oymvvq"
impl User {
    fn validate_email(&mut self, email: String) -> Result<(), UserError> {
        let email = email.trim().to_lowercase();

        if !is_valid_email(&email) {
            return Err(UserError::InvalidEmail);
        }

        self.email = email;

        Ok(())
    }
}
```

The form binding does not need to know about the validation:

```rust id="b4xbzl"
input()
    .label("Email")
    .bind(User::email)
```

The binding remains simple because the Model owns the rules.

This gives Beverly two modes:

**Implicit when the behavior is obvious. Explicit when the behavior matters.**

The generated behavior handles the common case. Ordinary Rust handles the exceptional case.

> **Magic in implementation. Explicitness at the API.**

---

## Binding

`.bind()` connects an input to a Model field.

```rust id="g9st5i"
input()
    .label("Email")
    .bind(User::email)
```

The binding handles the translation between interface input and typed application state.

Conceptually, it:

1. Reads the current Model value.
2. Displays that value in the input.
3. Receives user input.
4. Transforms the input into the Model's type.
5. Calls the appropriate setter.
6. Reacts to the result.
7. Keeps the input synchronized with application state.

The Model remains the source of truth.

If another part of the application changes the user's email, the bound input reflects that change. The form does not maintain an authoritative duplicate of the value.

```text
Model → Binding → Input
Input → Binding → Setter → Model
```

This is two-way binding without requiring developers to manually maintain synchronization code.

---

## Validation

Validation belongs at the state transition.

Consider an email field.

The UI can tell the user that an email _looks_ invalid, but the application must ultimately decide whether the value is valid. That decision belongs with the Model.

```rust id="jh5y7y"
impl User {
    fn validate_email(&mut self, email: String) -> Result<(), UserError> {
        let email = email.trim().to_lowercase();

        if !is_valid_email(&email) {
            return Err(UserError::InvalidEmail);
        }

        self.email = email;

        Ok(())
    }
}
```

If validation fails, the Model does not change.

This means the same rule applies everywhere:

```text
Human → Form ───────┐
Agent ──────────────┤
API ────────────────┼→ Setter → Model
Application ────────┤
Other system ───────┘
```

The UI is one client of the application. It is not the authority over the application.

This is especially important for agentic applications. An agent should not have to reproduce the application's validation rules. It should attempt the state transition and handle the result.

> **Validation protects application state, not the screen.**

---

## Async Validation

Some validation requires external information.

For example, an email address may need to be checked against an existing account system:

```rust id="esw1c7"
impl User {
    async fn set_email(
        &mut self,
        email: String,
    ) -> Result<(), UserError> {
        let email = email.trim().to_lowercase();

        if !is_valid_email(&email) {
            return Err(UserError::InvalidEmail);
        }

        if email_exists(&email).await? {
            return Err(UserError::EmailAlreadyRegistered);
        }

        self.email = email;

        Ok(())
    }
}
```

The important architectural point is that the validation still belongs to the state transition.

The form does not need to know how the validation works. It only needs to represent the state of the operation:

- pending
- accepted
- rejected

Beverly's Event architecture handles the asynchronous boundary.

> **Async does not create a second architecture. It uses the same architecture.**

---

## Editing State

Application state and editing state are different things.

The **Model owns application state**.

The **Form owns editing state**.

A field may need to know whether it has been:

- touched
- changed
- focused
- validated
- submitted
- rejected
- validated asynchronously
- waiting for a response

For example:

**Touched** means the user has interacted with the field.

**Dirty** means the current editing value differs from the original value. If the user changes a value and then restores the original value, the field can become clean again.

**Pending** means an asynchronous operation is currently in progress.

This information belongs to the form because it describes the **editing experience**, not the application's underlying truth.

The Model does not need to know whether a user has clicked into an input three times.

---

## Building a Form

With the Model defined, the form itself stays small.

```rust id="i4wfn2"
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
    .on("submit", Command::User::Create)
```

Notice what is _not_ here.

There are no manual change handlers.

There is no form-specific copy of the `User`.

There is no email validation function attached to the input.

There is no synchronization code.

The form describes the interface. The bindings connect it to the Model. The Model defines what is allowed to happen.

---

## Submission

Editing and submission are separate concerns.

The fields update the Model through their bindings.

The submit control expresses an intent:

```rust id="kmltri"
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

The form emits the Command.

The Controller decides what that Command does.

For example:

```rust id="q3r8v1"
controller! {
    User::Create => Api::users::create,
}
```

The API receives typed application state rather than raw UI input.

```rust id="lvmgi6"
async fn create(user: User) -> Result<User, ApiError> {
    api
        .post("/users", &user)
        .await
}
```

The conceptual flow is:

```text
Form
  ↓
Model
  ↓
Command
  ↓
Controller
  ↓
API
  ↓
Fact
```

For example, a successful request can produce:

```text
User::Created
```

while a failure can produce:

```text
User::CreateFailed
```

The form can respond to those Events by showing success, displaying an error, resetting its editing state, or preserving the user's inputs.

Again, there is no separate form architecture for asynchronous work. It is the same Event architecture used everywhere else in Beverly.

---

## Forms and AI

A Beverly form is not a privileged path into application state.

It is simply one interface.

A human can change a user's email.

An agent can request the same change.

An API can request the same change.

An internal application service can request the same change.

They all eventually encounter the same Model rules.

That gives an agent a much cleaner interface to the application.

Instead of learning:

> “If the user enters an email here, run this validation function, then update this piece of state…”

the agent works with the application's actual vocabulary:

```text
User::SetEmail
User::Created
User::CreateFailed
```

The Model enforces the rules.

The Controller coordinates the action.

Events communicate what happened.

The form simply gives humans a way to interact with the same system.

---

## Form Contract

A Beverly form follows a small set of rules:

- **Model owns application state.**
- **Form owns editing state.**
- **Bindings connect inputs to typed Model fields.**
- **Setters are the authoritative state-transition boundary.**
- **Validation belongs with the state transition.**
- **`#[model]` generates common accessor boilerplate.**
- **Custom setters can be ordinary Rust.**
- **`#[setter = ...]` makes custom field behavior explicit when needed.**
- **Inputs track editing concerns such as touched, dirty, focused, and pending.**
- **Submit controls emit Commands.**
- **Controllers coordinate actions.**
- **Events carry asynchronous results and application facts.**
- **Humans, agents, APIs, and other systems use the same underlying application rules.**

The result is a form that is simple at the surface without being simplistic underneath.

> **The form handles the editing experience.
> The Model handles the truth.
> The setter handles the rules.
> The Command expresses intent.
> Events carry the result through the application.**

A Beverly form is a **typed interface that transforms user input into validated application state.**
