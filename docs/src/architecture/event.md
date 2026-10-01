# 3.4 Events

**Events are the vocabulary of a Beverly application.**

They describe what can happen and what has happened.

`event!` declares ordinary Bevy `Message` structs in namespaces. Registration
and handling remain explicit application responsibilities:

```rust
use bevy::prelude::*;
use beverly::prelude::*;

event! {
    User::Create {
        name: String
    }
    User::Delete {
        id: u64
    }
}

fn setup(mut app: App) {
    app.add_message::<User::Create>()
        .add_message::<User::Delete>();
}
```

Write and read them with Bevy's normal message APIs:

```rust
fn create_user(mut messages: MessageWriter<User::Create>) {
    messages.write(User::Create {
        name: "Alice".to_string(),
    });
}

fn handle_users(mut messages: MessageReader<User::Create>) {
    for message in messages.read() {
        info!(name = %message.name, "creating user");
    }
}
```

The macro declares types only. It does not register messages, route commands,
or emit events automatically; those behaviors stay explicit in the systems and
plugins that own the application contract.

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

## How Events Are Stored

The application state you see at any moment is not the source of truth.

The Event history is.

```text id="a1e9tv"
Event Log (append-only)
   │
   ├── 001  Fact: DocumentCreated   { id: 1 }
   ├── 002  Command: Document::Summarize { id: 1 }
   ├── 003  Fact: DocumentSummarized { id: 1 }
   ├── 004  Command: Document::Publish { id: 1 }
   └── 005  Fact: DocumentPublished { id: 1 }
```

Every Event is written to this log in the order it occurred, and it is never edited or removed once written.

Each entry carries the information required to reproduce it later:

```text id="y3w7pk"
Sequence   → position in history
Timestamp  → when it happened
Event      → the typed Command or Fact
Payload    → its strongly typed data
```

The Model's current state is simply what you get from folding the log from the beginning:

```text id="q8m4rd"
State(0) + Event 001 + Event 002 + ... + Event N = Current State
```

Because the log is append-only, this is the same guarantee databases call an event-sourced or write-ahead log: history is immutable, and current state is a derived, disposable projection of it.

> **The log is the truth. The current state is just a cache of it.**

## Saving Event History

Saving an application's history means saving its Event log, not its in-memory Models.

```text id="n7c2xh"
Event
  ↓
Serialize (typed payload → bytes)
  ↓
Append to log (disk, database, or stream)
  ↓
Acknowledge
```

Because Events are strongly typed Rust values, they serialize the same way any other typed Rust data does. There is no bespoke save format to invent per feature.

The log can live wherever it makes sense for the application:

```text id="s2b6mp"
Event Log
   │
   ├── a local file
   ├── SQLite / Postgres
   ├── an append-only object store
   └── a distributed log (e.g. Kafka-style stream)
```

Saving does not require freezing the whole application. Only the new Events since the last save need to be appended, which is what keeps this cheap even for long-running applications.

> **Persistence is just "keep appending." Nothing about the Model needs a separate save/load path.**

## Replaying

Replaying means reading the Event log from the start and re-applying every Event, in order, to rebuild state.

```text id="e5t1zq"
Empty State
   │
   ├── apply Event 001
   ├── apply Event 002
   ├── apply Event 003
   ├── ...
   └── apply Event N
   ↓
Reconstructed State (identical to the original)
```

Because Facts describe what already happened, replaying them is deterministic: applying the same Events in the same order always produces the same resulting state.

This is useful in several concrete situations:

```text id="f9d4kw"
Restarting the application
    → replay the log to rebuild state from nothing

Debugging a production issue
    → replay a copied log locally to reproduce it exactly

Testing
    → replay a recorded scenario as a regression test

Migrating a Model
    → replay old Events through new logic to backfill new state
```

> **Nothing about "restoring the application" is special-cased. It's the same replay used everywhere else.**

## Rewinding

Rewinding moves state *backward* by replaying only a prefix of the log, rather than the whole thing.

```text id="j2p6vc"
Event 001 → Event 002 → Event 003 → Event 004 → Event 005
                              ▲
                    rewind target: after Event 003
```

To rewind to a point in time, the application discards nothing. It simply re-derives state by folding the log up to that Event and stopping:

```text id="h4k9wx"
State(0) + Event 001 + Event 002 + Event 003 = State at that point in time
```

The Events after the rewind point are **not deleted**. They remain in the log. Rewinding is a read operation over history, not a mutation of it.

```text id="c8r3nd"
Rewind ≠ Undo-by-deleting
Rewind = Recompute state as of an earlier Event
```

This distinction matters. Because the full log still exists, an application (or a developer, or an agent) can rewind, inspect an earlier state, and then choose to either:

```text id="w6q1zf"
Resume forward
    → continue applying the remaining Events as normal

Branch
    → issue new Commands from that point, creating a new
      continuation of history instead of the original one
```

> **Rewinding shows you the past. It doesn't erase it.**

## Stepping Through

Because the log is just an ordered list, moving through it one Event at a time is a simple index operation, not a special debugging feature bolted on afterward.

```text id="t4v8mn"
            ◀── step back            step forward ──▶

Event 001  Event 002  [ Event 003 ]  Event 004  Event 005
                            ▲
                      current cursor
```

A step forward applies the next Event in the log to the current state:

```text id="i9y5qk"
State(cursor) + Event(cursor + 1) = State(cursor + 1)
```

A step back recomputes state as of the previous Event, the same way rewinding does, just one Event at a time:

```text id="l3n7ub"
State(cursor − 1) = fold(Event 001 ..= Event(cursor − 1))
```

This gives an application (or a developer tool built on top of it) the same experience as stepping through a debugger, except the "breakpoints" are the application's own Commands and Facts:

```text id="o7a2ic"
Step back  → Fact: DocumentSummarized
Step back  → Command: Document::Summarize
Step back  → Fact: DocumentCreated
```

Because every step is just a Command or Fact, stepping through history is also stepping through an audit trail. There is nothing else to reconstruct.

> **A step-through debugger for application behavior isn't a separate feature. It's the same Event log, read one entry at a time.**

## Snapshots

Replaying an entire log from the beginning is simple and correct, but for a long-running application it can become slow.

A snapshot solves this without changing anything about how replay, rewind, or stepping work conceptually:

```text id="z1x5me"
State(0) + Events 001..500  = State at Event 500
                                     │
                                     ▼
                             Snapshot("at 500")

Snapshot("at 500") + Events 501..N = Current State
```

A snapshot is just a cached checkpoint: a Model's state at a known Event, saved so replay doesn't need to start from empty state every time.

```text id="v9b3ho"
Replay without a snapshot:
   Empty State + Events 001..10,000

Replay with a snapshot:
   Snapshot(9,900) + Events 9,901..10,000
```

Snapshots are an optimization, not a change to the model of history. The Event log remains the single source of truth; a snapshot can always be regenerated by replaying from the beginning.

> **A snapshot is a shortcut through history, never a replacement for it.**

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
