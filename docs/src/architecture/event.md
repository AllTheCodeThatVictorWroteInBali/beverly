# 3.4 Events

**Events are the vocabulary of a Beverly application.**

They describe what can happen and what has happened.

Beverly uses two kinds of Events:

- **Commands** — things you want to happen.
- **Facts** — things that happened.

```text id="q7m2kx"
Command
    │
    │ "I want this to happen."
    ▼
 Application
    │
    │ "This happened."
    ▼
Fact
```

This distinction gives applications a simple, predictable language for describing behavior.

## Commands

A Command expresses an intention.

It tells the application what you want it to do.

```rust id="8x4m1p"
event! {
    User::Create {
        name: String
    }

    User::Delete {
        id: UserId
    }
}
```

These Events describe actions the application understands:

```text id="w3p9ra"
User::Create
User::Delete
```

A human can trigger them.

An API can trigger them.

Another application can trigger them.

An AI agent can trigger them.

The Event is the common interface.

There is no separate API for AI agents to learn.

## Facts

A Fact describes something that has already happened.

```rust id="j5k8vd"
event! {
    User::Created {
        id: UserId
    }

    User::Deleted {
        id: UserId
    }
}
```

The distinction is simple:

```text id="n4c7fz"
CreateUser
    = Command
    = "Create this user."

UserCreated
    = Fact
    = "This user was created."
```

Commands express **intent**.

Facts express **reality**.

This makes it possible for different parts of the application to react to the same facts without needing to know which component originally caused them.

## Events Are Strongly Typed

Events are defined as typed Rust contracts.

```rust id="r1v6cm"
event! {
    User::Create {
        name: String
    }
}
```

The compiler knows what `User::Create` requires.

An invalid payload is not a runtime mystery.

It is a type error.

Rust catches malformed Events at compile time.

This gives Events a property that is particularly valuable for AI-generated code:

> **The compiler can tell the AI when it generated something that doesn't match the application's contract.**

The same type system that protects ordinary Rust code protects the application's event vocabulary.

## Events as Self-Documenting Code

An Event definition tells you what the application can do without requiring you to read the implementation first.

```rust id="m2q9wv"
event! {
    Document::Summarize {
        id: DocumentId
    }

    Document::Delete {
        id: DocumentId
    }

    Document::Publish {
        id: DocumentId
    }
}
```

You can glance at this and immediately understand three capabilities of the application.

That is intentional.

Beverly aims to reduce boilerplate by making the code itself the documentation.

Instead of maintaining separate lists of:

- API endpoints
- AI tools
- event names
- documentation
- schemas
- permissions
- telemetry definitions

the Event becomes the authoritative definition from which these things can be derived.

> **The best documentation is the code that cannot lie.**

## Events Are an Interface for AI

This becomes especially powerful when an AI agent participates in the application.

The agent doesn't need to understand the entire application implementation.

It needs to understand the application's vocabulary.

For example:

```text id="z8c3pk"
User::Create
    name: String

User::Delete
    id: UserId

Document::Summarize
    id: DocumentId
```

The agent can discover these Commands and determine what actions are available.

To perform an action, it simply issues the corresponding Command.

```text id="v4m1qs"
Agent
  │
  │ User::Create { name: "Alice" }
  ▼
Controller
  │
  ▼
Model
```

The agent doesn't need a special `Agent::CreateUser` interface.

It uses the same application vocabulary as everything else.

Agents can also listen for Facts:

```text id="c2x7mn"
UserCreated
DocumentPublished
PaymentCompleted
```

An agent can respond when the Fact it cares about occurs.

```text id="s6r8wp"
UserCreated
    ↓
Agent observes Fact
    ↓
Agent decides what to do
    ↓
Agent issues Command
```

This produces a very simple agent loop:

> **Listen for Facts. Decide. Issue Commands.**

## Events and Observability

Beverly Events are also observable by design.

Under the hood, Beverly's Event system integrates with OpenTelemetry so that Events can carry standardized telemetry information as part of their normal lifecycle.

That means application activity can be observed without developers having to build a separate logging architecture around every action.

Conceptually:

```text id="h9k3vd"
Event
  ↓
Action
  ↓
State Change
  ↓
Telemetry
  ↓
Event History
```

The important idea is that **the application already knows what happened**.

The Event is the natural unit of application activity.

Logging doesn't need to be a separate thing that developers remember to add everywhere.

## Playback and Rewind

Because Events form an ordered history of application activity, playback becomes remarkably straightforward.

Instead of trying to reconstruct what happened from scattered logs, you can step through the application's Event history:

```text id="p5x8rc"
Event 001
   ↓
Event 002
   ↓
Event 003
   ↓
Event 004
   ↓
Current State
```

Replaying the application means replaying its Events.

Rewinding means moving back through that history and reconstructing the corresponding state.

This creates an important capability for AI-native applications.

When an agent changes something, its actions become part of the same observable application history as human actions.

A developer can inspect:

```text id="k6r2nz"
Fact: DocumentCreated
Command: Document::Summarize
Fact: DocumentSummarized
Command: Document::Publish
Fact: DocumentPublished
```

The AI's behavior is no longer hidden behind a collection of opaque tool calls.

It becomes part of the application's history.

> **If the application records what happened, the application can show you what happened.**

## Less Boilerplate

Traditional applications often maintain multiple representations of the same capability.

A feature might require:

```text id="d3f7mx"
Implementation
API endpoint
Schema
Documentation
AI tool definition
Logging
Permissions
Tests
```

Beverly's Event system is designed to collapse much of that duplication.

One strongly typed Event definition can become the source for the representations the rest of the application needs.

```text id="u8m4qp"
             Event Definition
                    │
       ┌────────────┼────────────┐
       ↓            ↓            ↓
    Rust Types    AI Tools    Documentation
       │            │            │
       ├────────────┼────────────┤
       ↓            ↓            ↓
   Validation   Permissions   Telemetry
```

The goal isn't to generate more code.

It is to **stop writing the same information multiple times**.

## Events Are the Contract

Events create a shared vocabulary between every participant in the application.

```text id="b7n5kx"
Human ────────┐
              │
Keyboard ─────┤
              │
API ──────────┤
              ├──► Command ──► Controller ──► Model
              │
AI Agent ─────┘

Model ──► Fact ──► View
              │
              ├──► Agent
              │
              ├──► Other Systems
              │
              └──► Event History
```

Commands tell the application what someone wants to happen.

Facts tell the application what actually happened.

The compiler makes sure those messages conform to their contracts.

OpenTelemetry makes their activity observable.

The Event history makes that activity replayable.

And the same vocabulary can be understood by both humans and AI agents.

> **Events turn application behavior into typed, observable, self-documenting vocabulary.**

That is the foundation for Beverly's larger idea:

**One application. One vocabulary. Humans and AI use the same interface.**
