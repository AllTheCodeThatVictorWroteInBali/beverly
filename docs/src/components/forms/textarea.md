# Textarea

`textarea()` is the multiline counterpart to `input()`.

It follows the same patterns:

- A label is required.
- An ARIA declaration is required.
- `.bind()` connects it to Model state.
- The Model setter is called every time the value changes.
- The Model validates and transforms the value.
- Setter errors become field-level errors through the binding.
- Errors are exposed to the `.error(...)` context as an array of strings.
- The Model can perform asynchronous validation.
- Pending state is represented by the component.
- Debouncing belongs in the Model.
- The Model remains the source of truth.

The only fundamental difference is that `textarea()` is designed for multiline content.

## The Basic Textarea

A basic textarea:

```rust id="5q8m3n"
textarea()
    .label(localize("profile.bio.label"))
    .aria(localize("profile.bio.description"))
```

A bound textarea:

```rust id="q4w8h2"
textarea()
    .label(localize("profile.bio.label"))
    .aria(localize("profile.bio.description"))
    .bind(User::bio)
```

The API should feel immediately familiar to anyone who has used an HTML textarea, while the underlying behavior follows Beverly's Model and binding architecture.

## Binding

`.bind()` connects the textarea to a Model property:

```rust id="7j3p5k"
textarea()
    .label(localize("profile.bio.label"))
    .aria(localize("profile.bio.description"))
    .bind(User::bio)
```

The textarea reads its initial value from the Model.

When the user types, the setter is invoked.

When the Model changes, the textarea changes.

```text id="1r8c4m"
Model → Binding → Textarea
Textarea → Binding → Setter → Model
```

There is no second authoritative copy of the text inside the textarea.

## The Setter Is Called on Every Change

Just like `input()`, the Model setter is called whenever the textarea value changes.

If a user types:

```text id="m5n8t2"
H
He
Hel
Hell
Hello
```

the setter receives those changes.

```text id="8f2k6p"
Textarea Change
       ↓
     Binding
       ↓
   Model Setter
       ↓
     Result
```

The Model can then decide what to do with each change.

It can:

- Accept the value
- Reject the value
- Normalize the value
- Transform the value
- Track editing state
- Perform asynchronous validation
- Debounce expensive work

The textarea does not need to know which behavior is being used.

## The Model Can Transform the Text

The Model can modify the value and send the resulting value back through the binding.

For example, suppose a description should never contain leading or trailing whitespace:

```rust id="9d7f4q"
impl User {
    fn set_bio(
        &mut self,
        bio: String,
    ) -> Result<(), UserError> {
        let bio = bio.trim().to_string();

        self.bio = bio;

        Ok(())
    }
}
```

Or the Model might normalize repeated whitespace, enforce a domain-specific format, or transform the content before storing it.

The resulting flow is:

```text id="4z7n2p"
User Text
   ↓
Setter
   ↓
Normalize
   ↓
Model State
   ↓
Binding
   ↓
Textarea
```

The Model determines the canonical value.

## Validation

Textarea validation works exactly like input validation.

Suppose a biography must contain at least 20 characters:

```rust id="k3v8q1"
impl User {
    fn set_bio(
        &mut self,
        bio: String,
    ) -> Result<(), UserError> {
        let bio = bio.trim().to_string();

        if bio.chars().count() < 20 {
            return Err(UserError::BioTooShort);
        }

        self.bio = bio;

        Ok(())
    }
}
```

The textarea does not implement this rule.

The Model does.

This means the same rule applies regardless of whether the value comes from a human, an agent, an API, or another application component.

## Field Errors

If the setter returns an error, the binding associates that error with the textarea.

```rust id="q8m4s2"
textarea()
    .label(localize("profile.bio.label"))
    .aria(localize("profile.bio.description"))
    .bind(User::bio)
    .error(
        self.children([
            text(|errors: &[String]| errors.join(" "))
        ])
    )
```

The error context is automatically exposed to the `.error(...)` content.

The errors are an array of strings.

The strings originate from the Model's validation response.

```text id="7p2c9v"
Model Setter
     ↓
   Errors
[String, String, ...]
     ↓
  Binding
     ↓
Textarea Error Context
     ↓
.error(...)
```

The textarea does not generate the validation message.

The Model does.

## Internationalization

Validation messages should never be hard-coded into the textarea.

Instead of:

```rust id="f8n2x6"
text("Biography is too short.")
```

the Model returns the appropriate structured error, and the error exposed by the binding contains the localized message.

```rust id="w6j4q9"
textarea()
    .label(localize("profile.bio.label"))
    .aria(localize("profile.bio.description"))
    .bind(User::bio)
    .error(
        self.children([
            text(|errors: &[String]| errors.join(" "))
        ])
    )
```

The textarea contains no language-specific validation copy.

The label and ARIA description come from the application's localization system.

The error comes from the Model.

> **The Model returns meaning. The UI presents language.**

## Multiple Errors

A Model may return more than one error for a field.

For example:

```text id="v7k2m5"
[
    "Biography is too short.",
    "Biography contains prohibited content."
]
```

The error context exposes the errors together.

The View decides how to present them.

For example:

```rust id="z3q6h1"
textarea()
    .label(localize("profile.bio.label"))
    .aria(localize("profile.bio.description"))
    .bind(User::bio)
    .error(
        self.children([
            text(|errors: &[String]| errors.join("\n"))
        ])
    )
```

The Model determines the errors.

The component determines their presentation.

## Asynchronous Validation

Textarea values can also require asynchronous validation.

For example, a content field might need to be checked by a remote service.

The Model owns that behavior:

```rust id="n4y8c2"
impl User {
    async fn set_bio(
        &mut self,
        bio: String,
    ) -> Result<(), UserError> {
        let bio = bio.trim().to_string();

        if bio.chars().count() < 20 {
            return Err(UserError::BioTooShort);
        }

        if !content_allowed(&bio).await? {
            return Err(UserError::ContentNotAllowed);
        }

        self.bio = bio;

        Ok(())
    }
}
```

The textarea does not need an async validation API.

The setter is still the state transition.

The difference is that the Model performs asynchronous work before accepting that transition.

## Pending State

While asynchronous validation is occurring, the textarea can expose a pending presentation:

```rust id="c7m2x9"
textarea()
    .label(localize("profile.bio.label"))
    .aria(localize("profile.bio.description"))
    .bind(User::bio)
    .submitting(
        self.opacity(0.5)
    )
```

Or:

```rust id="p8v4n6"
textarea()
    .label(localize("profile.bio.label"))
    .aria(localize("profile.bio.description"))
    .bind(User::bio)
    .submitting(
        self.children([
            spinner(),
            text(localize("profile.bio.checking"))
        ])
    )
```

Again, `self` refers to the textarea itself.

The async Event or setter does not directly manipulate the textarea.

The Model enters a pending state.

The textarea observes that state and presents itself accordingly.

> **State determines when. The component determines what.**

## Debouncing

The setter is called for every textarea change.

That means debouncing expensive work belongs in the Model.

Suppose content validation requires a remote request. A user typing a paragraph should not trigger a network request for every character.

The Model can debounce the work:

```rust id="h2q7m4"
impl User {
    async fn set_bio(
        &mut self,
        bio: String,
    ) -> Result<(), UserError> {
        self.bio = bio.clone();

        debounce(Duration::from_millis(300)).await;

        validate_content(&bio).await?;

        Ok(())
    }
}
```

The textarea continues to report every change.

The Model decides when the expensive operation should actually happen.

```text id="6p3v8n"
Every Change
     ↓
  Setter
     ↓
   Model
     ↓
 Debounce
     ↓
Async Validation
     ↓
   Result
     ↓
 Binding
     ↓
 Textarea
```

This keeps timing and network behavior out of the View.

The same behavior therefore applies if the Model transition is initiated by another application client.

## Touched State

The Model can also control when validation should occur.

For example, an application may not want to show a biography error until the user has interacted with the field.

If the validation rules depend on `touched`, that state participates in the Model's logic.

```rust id="r6m9q2"
if touched {
    validate_bio(&bio)
} else {
    Ok(())
}
```

The important principle is:

> **If validation behavior depends on state, that state belongs to the Model.**

The textarea presents the result.

It does not invent the rule.

## Styling

Textarea uses the same fluent component API:

```rust id="w3k8p5"
textarea()
    .label(localize("profile.bio.label"))
    .aria(localize("profile.bio.description"))
    .bind(User::bio)
    .padding(12)
    .radius(8)
    .width(400)
    .height(160)
```

Error presentation follows the same pattern:

```rust id="m7q2v4"
textarea()
    .label(localize("profile.bio.label"))
    .aria(localize("profile.bio.description"))
    .bind(User::bio)
    .error(
        self
            .color("red")
            .children([
                text(|errors: &[String]| errors.join(" "))
            ])
    )
```

Pending presentation:

```rust id="x9c3n7"
textarea()
    .label(localize("profile.bio.label"))
    .aria(localize("profile.bio.description"))
    .bind(User::bio)
    .submitting(
        self.opacity(0.5)
    )
```

The same component API handles normal, error, and pending states.

## The Complete Textarea

A production textarea can combine all of these capabilities:

```rust id="q5m8v2"
textarea()
    .label(localize("profile.bio.label"))
    .aria(localize("profile.bio.description"))
    .bind(User::bio)
    .padding(12)
    .radius(8)
    .width(400)
    .height(160)
    .error(
        self.children([
            text(|errors: &[String]| errors.join(" "))
        ])
    )
    .submitting(
        self.opacity(0.5)
    )
```

There is no manual change handler.

There is no second source of truth.

There is no UI-specific validation implementation.

There is no hard-coded validation language.

The input reports changes.

The binding connects those changes to the Model.

The setter validates and transforms the value.

The Model can perform asynchronous work.

The binding exposes the resulting errors.

The textarea presents the resulting state.

> **Textarea is not a special form system. It is the same Beverly binding contract applied to multiline text.**
