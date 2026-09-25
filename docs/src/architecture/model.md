# 3.2 Model

The **Model owns the application's data**.

In Beverly, a Model is just Rust.

There is no special Model language to learn and no separate state-management framework required. A Model is a normal Rust struct with familiar methods for reading and changing its data.

```rust id="7x5j2k"
struct UserModel {
    name: String,
    email: String,
}

impl UserModel {
    fn name(&self) -> &str {
        &self.name
    }

    fn email(&self) -> &str {
        &self.email
    }

    fn set_name(&mut self, name: String) {
        self.name = name;
    }

    fn set_email(&mut self, email: String) {
        self.email = email;
    }
}
```

That's a Beverly Model.

## Familiar by Design

Beverly deliberately uses a familiar getter/setter pattern.

Getters read the data.

Setters change the data.

Domain methods perform operations.

```rust id="nq2p4r"
impl UserModel {
    fn rename(&mut self, name: String) {
        self.set_name(name);
    }
}
```

This is intentionally boring.

That's a feature.

A developer can look at a Model and immediately understand what it owns and how it can change.

An LLM can recognize the pattern immediately and reliably generate new Models from it.

If an application already has a `UserModel`, an AI can predict what a `DocumentModel`, `AccountModel`, or `ProjectModel` should look like without having to learn a proprietary state-management system.

> **The Model should be obvious.**

## Rust Safety Without Rust Complexity

Because Models are ordinary Rust, they inherit Rust's safety guarantees.

Data has a clear owner.

Access to that data is controlled through the Model's interface.

Rust's compiler handles the underlying memory-safety guarantees without requiring the developer to build their application around complicated ownership patterns.

Beverly does not ask developers to fill their application code with `Arc<Mutex<T>>`, elaborate borrowing patterns, or other advanced Rust techniques simply to manage application state.

The framework can handle complexity internally.

The application code stays simple.

```text id="v6zj9s"
        Model
          │
          ├── owns data
          │
          ├── getters → read
          │
          ├── setters → change
          │
          └── methods → behavior
```

This gives Beverly an important combination:

**Familiar application architecture + Rust's safety guarantees.**

## Models Own Their Access Control

A Model should also own the rules governing access to its data.

The UI should not be responsible for deciding whether an operation is permitted.

An AI agent should not be responsible for deciding whether an operation is permitted.

An API should not have to duplicate the same business rules.

The Model is the authoritative owner of the data and its domain rules.

For example:

```rust id="q1c7ma"
impl UserModel {
    fn set_email(&mut self, email: String) -> Result<(), UserError> {
        if !is_valid_email(&email) {
            return Err(UserError::InvalidEmail);
        }

        self.email = email;
        Ok(())
    }
}
```

The rule travels with the data.

Every path that ultimately changes the Model goes through the same domain logic.

This becomes particularly important in AI-native applications.

An AI agent may request:

```text
User::Delete
```

But the agent does not get to bypass the application's rules.

The application decides whether that operation is authorized.

The Controller can enforce application-level policy, while the Model enforces the integrity and domain rules of the data itself.

> **The Model owns the data. It should also own the rules that keep that data valid.**

## Models Can Talk to Anything

A Model isn't limited to in-memory data.

It can connect to whatever the application needs.

For example:

```text id="q7r1x4"
Model
 │
 ├── SQLite
 ├── PostgreSQL
 ├── APIs
 ├── Parquet
 ├── Files
 ├── Streams
 ├── Data Lakes
 └── AI-generated data
```

A `DocumentModel` might load documents from a database.

A `TelemetryModel` might consume a high-throughput stream.

A `DataModel` might query Parquet files.

A `UserModel` might communicate with an API.

The View doesn't need to know where the data came from.

It simply reads the Model.

```text id="x2m8qc"
Database / Parquet / API / Stream
                ↓
              Model
                ↓
               View
```

This keeps the UI independent from the underlying data source.

It also makes Beverly suitable for applications that work with large, local, streaming, or heterogeneous datasets.

## Models Are Easy to Test

Because a Model is ordinary Rust, testing one doesn't require launching the application.

You can write a standard Rust unit test:

```rust id="8p4z1w"
#[test]
fn user_email_can_be_changed() {
    let mut user = UserModel {
        name: "Alice".into(),
        email: "alice@example.com".into(),
    };

    user.set_email("new@example.com".into());

    assert_eq!(user.email(), "new@example.com");
}
```

Domain rules can be tested the same way:

```rust id="3f6w2n"
#[test]
fn invalid_email_is_rejected() {
    let mut user = UserModel {
        name: "Alice".into(),
        email: "alice@example.com".into(),
    };

    assert!(user.set_email("not-an-email".into()).is_err());
}
```

No window.

No renderer.

No UI automation.

No browser.

Just Rust.

This is one of Beverly's biggest architectural advantages:

> **If your application logic is Rust, test your application logic with Rust.**

The same simplicity extends to AI-generated code.

An LLM can generate a Model.

A developer can read it.

A developer can modify it.

A standard Rust test can verify it.

And Rust's compiler can enforce its safety.

## The Model's Role

The Model has a deliberately narrow responsibility:

**Own the data. Define how that data can change. Enforce its domain rules. Connect to the underlying data sources.**

It does not render the UI.

It does not decide how a Button looks.

It does not need to know whether a human or AI initiated an operation.

Those responsibilities belong elsewhere.

```text id="a4y9kc"
             Event
               ↓
          Controller
               ↓
             Model
          ↙    ↓    ↘
       Rules  Data  Sources
               ↓
             State
               ↓
              View
```

The result is a Model that is familiar to programmers, predictable for AI code generation, straightforward to test, and backed by Rust's safety guarantees.

> **The Model is just Rust — and that's exactly the point.**
