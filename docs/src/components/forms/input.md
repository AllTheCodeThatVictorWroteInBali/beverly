# Text Input

The text input is the most common form element in Beverly.

It looks simple:

```rust
input()
```

But a production input has several responsibilities:

- Present a value
- Identify itself with a label
- Provide accessible semantics
- Bind to application state
- Convert user input into the Model's type
- Invoke the appropriate setter
- Receive validation results
- Present field-level errors
- Reflect changes made by the Model
- Represent editing state such as touched, dirty, focused, and pending

Beverly keeps these responsibilities together without requiring the developer to manually wire them up.

## A Valid Input Has a Label and ARIA

Every Beverly form element requires both a label and an ARIA declaration.

```rust
input()
    .label(localize("user.email.label"))
    .aria(localize("user.email.description"))
```

These are part of the form element's contract, not optional accessibility enhancements.

The label identifies the field.

The ARIA declaration provides additional accessible semantics or descriptive information.

Beverly can therefore treat missing required accessibility metadata as a compile-time error.

> **Accessible by construction, not accessible by cleanup.**

This is particularly important for AI-generated interfaces. An agent should not have to remember accessibility requirements as a separate cleanup step. The component API makes the required structure explicit.

## The Basic Input

A useful input normally has three fundamental pieces:

```rust
input()
    .label(localize("user.email.label"))
    .aria(localize("user.email.description"))
    .bind(User::email)
```

The label identifies the field.

The ARIA declaration provides accessible context.

The binding connects the field to application state.

Everything else builds on these three pieces.

## Binding

`.bind()` connects the input to a Model field.

```rust
input()
    .label(localize("user.email.label"))
    .aria(localize("user.email.description"))
    .bind(User::email)
```

The binding handles the translation between interface input and typed application state.

Conceptually:

```text
Model → Binding → Input
Input → Binding → Setter → Model
```

The Model remains the source of truth.

The developer does not need to manually write a change handler, synchronize state, or copy the input value into another form-specific state object.

The binding handles that connection.

## The Setter Is Called on Every Change

A bound input invokes the Model setter every time the input changes.

For example, if a user types:

```text
v
vi
vic
vict
victor
```

the corresponding setter is invoked for each change.

```text
Input Change
     ↓
  Binding
     ↓
Model Setter
     ↓
Result
```

This is intentional.

The Model sees the stream of values and decides what to do with them.

For a simple field, that might mean validating immediately.

For another field, it might mean normalizing the value.

For an asynchronous field, it might mean starting or scheduling a request.

The Form does not need to know which behavior is appropriate.

> **The input reports every change. The Model decides what every change means.**

## The Model Can Change the Input

Binding is not simply a one-way mechanism for sending text into the Model.

The Model can transform the value and send the canonical value back to the input.

Phone numbers are a good example.

A user might enter:

```text
4155551234
```

The Model may normalize that value to:

```text
(415) 555-1234
```

For example:

```rust
impl User {
    fn set_phone(
        &mut self,
        phone: String,
    ) -> Result<(), UserError> {
        let phone = format_phone_number(phone)?;

        self.phone = phone;

        Ok(())
    }
}
```

The resulting flow is:

```text
User Input
    ↓
Binding
    ↓
Model Setter
    ↓
Normalize / Validate
    ↓
Model State
    ↓
Binding
    ↓
Formatted Input
```

The input does not need to know how phone numbers are formatted.

The Model owns the canonical representation.

This applies to:

- Phone numbers
- Currency
- Dates
- Identifiers
- Capitalization
- Whitespace
- Units
- Domain-specific formats

The user edits the value.

The Model decides what the application's value actually becomes.

> **The input collects the value. The Model defines the value.**

## The Setter Owns Validation

Validation belongs to the Model.

For example:

```rust
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

The input does not know what makes an email valid.

The Form does not know either.

The Model knows.

That means the same validation rule applies whether the value comes from:

- A human using the Form
- An AI agent
- An API
- Another component
- Another internal application action

There is one validation rule and one place to maintain it.

## From Setter Error to Field Error

Suppose the user enters an invalid email.

The binding passes the value to the Model:

```text
Input
  ↓
Binding
  ↓
User::set_email(...)
  ↓
Err(UserError::InvalidEmail)
```

The setter rejects the state transition.

The Model does not change.

The binding receives the error and associates it with the field that produced it.

Field-level errors are therefore **derived from the form binding**.

```text
Setter Result
     ↓
  Binding
     ↓
 Field Errors
     ↓
.error(...)
```

The UI does not independently validate the email.

It does not need to duplicate the Model's rules.

The setter returned an error.

The binding knows which field invoked the setter.

The field therefore receives the error.

## Errors Are Exposed to the Error Context

When `.error(...)` is configured, the current field errors are automatically exposed to the content inside the error hook.

Just as `self` refers to the component itself, the error context exposes the field's errors.

The error context is an array of strings:

```rust
input()
    .label(localize("user.email.label"))
    .aria(localize("user.email.description"))
    .bind(User::email)
    .error(
        self.children([
            text(|errors: &[String]| errors.join(" "))
        ])
    )
```

The important part is that the strings come from the Model's error response.

The form element does not invent them.

The Model produces the errors.

The binding associates those errors with the field.

The `.error(...)` content presents them.

A field can therefore expose multiple errors when the Model returns multiple validation messages:

```text
errors = [
    "Email address is invalid.",
    "Email domain is not allowed."
]
```

The error presentation can decide how those messages should be displayed.

For example:

```rust
.error(
    self.children([
        text(|errors: &[String]| errors.join("\n"))
    ])
)
```

The exact presentation is the View's responsibility.

The contents of the error are the Model's responsibility.

> **The Model produces the errors. The binding exposes them. The field presents them.**

## Internationalization

Validation text should not be hard-coded into the form element.

Avoid:

```rust
.error(
    self.children([
        text("Invalid email address.")
    ])
)
```

The Model should return the semantic validation error, and the resulting error context should provide the appropriate localized message.

Conceptually:

```text
Model
  ↓
Validation Error
  ↓
Localized Error Message
  ↓
Binding
  ↓
Field Error Context
  ↓
.error(...)
```

This means the reusable input does not contain English validation messages—or validation messages in any particular language.

Labels and ARIA descriptions should likewise come from the application's localization system:

```rust
input()
    .label(localize("user.email.label"))
    .aria(localize("user.email.description"))
    .bind(User::email)
```

The component describes the structure of the interface.

The localization system provides the language.

The Model provides the meaning of validation failures.

> **The Model returns meaning. The UI presents language.**

## Asynchronous Validation

Some setters need to perform asynchronous work.

For example, an email address might need to be checked against a server to determine whether it is already registered.

The setter can represent that transition asynchronously:

```rust
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

The input does not need a special asynchronous validation framework.

It still calls the setter on every change.

The Model determines that this particular transition requires asynchronous work.

## Pending State

While the Model is performing asynchronous work, the field can expose its pending state.

For example:

```rust
input()
    .label(localize("user.email.label"))
    .aria(localize("user.email.description"))
    .bind(User::email)
    .submitting(
        self.opacity(0.5)
    )
```

Or the pending presentation can contain additional content:

```rust
input()
    .label(localize("user.email.label"))
    .aria(localize("user.email.description"))
    .bind(User::email)
    .submitting(
        self.children([
            spinner(),
            text(localize("user.email.checking"))
        ])
    )
```

The same principle applies:

**The state determines when the presentation is active. The component determines what that state looks like.**

The async request does not directly manipulate the UI.

The Model enters a pending state.

The input reflects that state.

## Debouncing Belongs in the Model

Because the setter is called on every input change, applications that perform expensive or remote work should debounce that work in the Model layer.

For example, typing:

```text
v
vi
vic
vict
victor
```

should not necessarily produce five network requests.

The Model can debounce the request:

```rust
impl User {
    async fn set_username(
        &mut self,
        username: String,
    ) -> Result<(), UserError> {
        self.username = username.clone();

        debounce(Duration::from_millis(300)).await;

        if username_exists(&username).await? {
            return Err(UserError::UsernameTaken);
        }

        Ok(())
    }
}
```

The important architectural point is that the Form still reports every change.

The Model decides that remote validation should wait until the user pauses.

```text
Every Input Change
        ↓
      Setter
        ↓
      Model
        ↓
    Debounce
        ↓
 Async Validation
        ↓
    Model Result
        ↓
      Binding
        ↓
       Input
```

This keeps debounce behavior out of the UI.

If an agent, API, or another application component invokes the same state transition, the same Model behavior applies.

### Avoiding Stale Requests

For remote validation, a production implementation should also prevent an older request from overwriting a newer result.

Conceptually:

```rust
impl User {
    async fn set_username(
        &mut self,
        username: String,
    ) -> Result<(), UserError> {
        self.username = username.clone();

        let request_id = self.next_validation_request();

        sleep(Duration::from_millis(300)).await;

        let available = username_available(&username).await?;

        if request_id != self.current_validation_request() {
            return Ok(());
        }

        if !available {
            return Err(UserError::UsernameTaken);
        }

        Ok(())
    }
}
```

The exact implementation can be hidden behind Beverly's runtime primitives.

The important rule is that **the Model owns the lifecycle of the asynchronous validation**.

The Form should not have to implement debounce timers, request cancellation, or stale-response handling.

## Validation Timing

The Model also controls when a field should be considered invalid.

For example, an application may not want to display an error before a user has interacted with a field.

If validation behavior depends on whether the field has been touched, that state must participate in the Model's state and logic.

Conceptually:

```rust
if touched {
    validate(value)
} else {
    Ok(())
}
```

The exact implementation can vary, but the architectural principle is important:

> **If validation behavior depends on state, that state belongs to the Model.**

The Form can present the resulting state, but it should not secretly redefine the application's validation rules.

## Successful Updates

When the setter returns:

```rust
Ok(())
```

the Model has accepted the state transition.

The binding reflects the accepted value back into the input.

If the Model transformed the value, the transformed value is what the input displays.

For example:

```text
User types:       4155551234
                       ↓
                  set_phone()
                       ↓
Model stores:    (415) 555-1234
                       ↓
Input displays:  (415) 555-1234
```

The Model is always the authoritative state.

## Error Updates

When the setter returns an error:

```rust
Err(...)
```

the Model rejects the transition.

The previous valid Model value remains authoritative.

The binding exposes the resulting errors to the field.

The field's `.error(...)` presentation becomes active.

```text
Input
  ↓
Setter
  ↓
Err
  ↓
Field Error Context
  ↓
.error(...)
```

The UI does not have to manually set an error flag.

The error is a consequence of the failed Model transition.

## Styling the Input

Inputs use the same fluent API as every other Beverly component.

```rust
input()
    .label(localize("user.email.label"))
    .aria(localize("user.email.description"))
    .bind(User::email)
    .padding(12)
    .radius(8)
    .width(320)
```

State-specific presentation uses the same API:

```rust
input()
    .label(localize("user.email.label"))
    .aria(localize("user.email.description"))
    .bind(User::email)
    .error(
        self.color("red")
    )
```

Or:

```rust
input()
    .label(localize("user.email.label"))
    .aria(localize("user.email.description"))
    .bind(User::email)
    .error(
        self.children([
            error_icon(),
            text(|errors: &[String]| errors.join(" "))
        ])
    )
```

The component API does not change because the input is in an error state.

## The Complete Input

A production input might therefore look like:

```rust
input()
    .label(localize("user.email.label"))
    .aria(localize("user.email.description"))
    .bind(User::email)
    .padding(12)
    .radius(8)
    .width(320)
    .error(
        self.children([
            text(|errors: &[String]| errors.join(" "))
        ])
    )
    .submitting(
        self.opacity(0.5)
    )
```

The input has:

- A required label
- A required ARIA declaration
- A Model binding
- Typed application state
- Model-owned validation
- Model-owned normalization
- Automatic synchronization
- Setter error propagation
- An automatically exposed error context
- Localized error presentation
- Async/pending state
- Fluent styling

And there is no manual `on_change` handler.

## The Text Input Contract

The text input can be understood as a contract between the Form and the Model:

```text
              TEXT INPUT

 Label ──────────────── Identity
 ARIA ───────────────── Accessibility
 Bind ───────────────── State connection
 Input ──────────────── User intent
 Setter ─────────────── Validation + transformation
 Result ─────────────── Accepted value or errors
 Model ──────────────── Source of truth
```

The input reports every change.

The binding connects that change to the Model.

The setter validates and transforms it.

The Model can synchronously or asynchronously change application state.

The binding reflects the resulting state back into the input.

Errors come from the Model and are exposed through the binding.

The field presents those errors.

> **The input collects. The binding connects. The setter validates and transforms. The Model owns the state. The field presents the result.**
