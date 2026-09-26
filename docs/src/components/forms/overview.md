### Implicit and Explicit Setters

In the common case, Beverly can infer the setter directly from the field.

```rust
#[model]
struct User {
    name: String,
    email: String,
    age: u32,
    active: bool,
}
```

Beverly can generate the ordinary Rust accessors automatically:

```rust
impl User {
    fn email(&self) -> &str {
        &self.email
    }

    fn set_email(&mut self, email: String) {
        self.email = email;
    }
}
```

So a binding can simply say:

```rust
input()
    .label("Email")
    .bind(User::email)
```

When the field needs custom behavior, the setter can be called out explicitly in the struct:

```rust
#[model]
struct User {
    name: String,

    #[setter = validate_email]
    email: String,

    age: u32,
    active: bool,
}
```

The binding remains exactly the same:

```rust
input()
    .label("Email")
    .bind(User::email)
```

But Beverly now knows that changing `email` should go through `validate_email` rather than the generated default setter.

```rust
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

This gives Beverly two modes:

**Implicit when the behavior is obvious. Explicit when the behavior matters.**

You don't have to write configuration for ordinary fields. When a field has domain-specific rules, the struct itself can make that relationship explicit.

This follows a broader Beverly principle:

> **Magic in implementation. Explicitness at the API.**

The generated behavior handles the common case. Ordinary Rust handles the exceptional case.

And because the setter is the state-transition boundary, the same validation applies regardless of where the change originates: a form, an agent, an API, or another part of the application.

**One field. One setter. One source of truth.**
