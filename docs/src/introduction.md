# Beverly

## What is Beverly?

Beverly is a UI framework written in Rust and built on top of the Bevy game engine.

It is designed for applications where the UI needs to be **fast, local, data-heavy, and increasingly intelligent**.

Beverly provides a native application runtime with:

- Small binary size
- Low resource overhead
- Fast, near-instant startup
- High-performance rendering
- Smooth 60 FPS interaction
- Local and offline execution
- Strong compile-time guarantees from Rust
- A UI architecture designed for both humans and AI agents

Beverly is not a web browser wrapped around an application. It is a UI runtime you can ship directly inside the product.

---

## The UI for AI

AI changes what a UI needs to be.

Traditional UI frameworks were designed primarily around a human sitting in front of a screen: click a button, fill out a form, navigate a page.

AI introduces another kind of user.

An agent can inspect application state, choose actions, invoke tools, respond to events, and operate an application alongside a human.

Beverly treats AI as a **first-class participant in the application**, rather than bolting an AI API onto an existing UI framework.

The same application events can be produced by:

- A human clicking a button
- A keyboard shortcut
- Another application
- An API
- An AI agent

This creates a common application language shared by humans and machines.

> **The UI is no longer just something an AI controls. It becomes part of the interface through which AI and humans work together.**

---

## Why Rust + Bevy?

Beverly combines Rust's systems programming model with Bevy's high-performance rendering architecture.

### Rust

Rust provides:

- Memory safety without a garbage collector
- Strong ownership and borrowing guarantees
- Compile-time type checking
- Predictable resource usage
- Safe concurrency
- Explicit application state and data flow

For an AI application, these properties have another advantage: **the UI consumes fewer resources that could otherwise be used by the application and its models.**

### Bevy

Bevy provides a high-performance foundation for:

- Rendering
- Animation
- Input
- Windows
- Assets
- GPU acceleration
- Cross-platform applications

Beverly builds the application UI layer on top of that foundation.

The goal is simple:

> **Use the game engine to build the application runtime, not a game.**

---

## Small Runtime, Large Surface Area

A UI framework shouldn't require a massive runtime just to render a button, a dashboard, or a data table.

Beverly aims to provide a surprisingly small runtime while still supporting the components required by serious applications.

That means:

- Small binaries
- Low memory overhead
- Fast startup
- Efficient rendering
- Minimal runtime dependencies
- Components for complex applications

The result is a UI runtime that can scale from a desktop application to a constrained device without bringing an entire browser stack along with it.

---

## Local-First, Offline, Sovereign

Beverly applications run locally.

They don't require:

- A browser
- A remote UI server
- A CDN
- Telemetry infrastructure
- A permanent internet connection

This makes Beverly well suited to applications where **data sovereignty, privacy, reliability, or deployment constraints matter**.

Applications can run:

- Offline
- On private networks
- In air-gapped environments
- On dedicated devices
- Inside enterprise infrastructure
- Alongside local AI models

Your UI can live where your data lives.

---

## Human + AI Interaction

Beverly does not create a separate interface for AI.

Instead, humans and agents interact with the same application primitives.

A button might emit:

```rust
Event::Document::Save
```

A keyboard shortcut can emit the same event.

An API can emit the same event.

An AI agent can emit the same event.

The Controller routes that event to application behavior, while the Model owns the underlying state.

This gives the application a single, explicit vocabulary that both humans and machines can understand.

## Rewindable AI

When application behavior is represented as explicit Events, AI actions don't have to disappear into an opaque execution log.

The application can record the sequence of events, the resulting state transitions, and the relevant context around an agent's actions.

That makes AI behavior observable, rewindable, and replayable.

A human can inspect what the agent did:

```
User::Open(8472)
    ↓
Document::Read(8472)
    ↓
Document::Summarize(8472)
    ↓
Document::Create("Quarterly Report")
    ↓
Document::Save(9124)
```

They can then replay the sequence to understand what happened, inspect individual decisions, or rewind the application to an earlier point.

This creates a fundamentally different relationship between humans and AI:

The AI doesn't just act. Its actions become part of the application's observable history.

Instead of asking "What did the AI do?", the application can show you.

Instead of asking "Why is the application in this state?", you can trace the events that produced it.

And instead of treating AI behavior as something that happened somewhere inside an opaque model, Beverly makes the application-level consequences of that reasoning inspectable and reproducible.

This is the foundation for AI visibility: humans remain able to observe, understand, rewind, and replay what their AI systems are doing.

---

## Design Philosophy

Beverly is built around a few core principles.

### Explicitness over magic

The implementation can be sophisticated. The application API should remain understandable.

> **Magic in implementation. Explicitness at the API.**

### Ordinary Rust whenever possible

If something can be solved cleanly with ordinary Rust, Beverly should not introduce another abstraction for it.

### Model, View, Controller, Events

Beverly uses a lightweight MVC architecture:

- **Model** — owns application state and behavior
- **View** — represents and renders the UI
- **Controller** — connects application events to behavior
- **Events** — define the application's vocabulary and contracts

Events provide the connection between the UI, application logic, external systems, and AI agents.

### Composition over complexity

Beverly components should be small, composable primitives.

```rust
card()
    .padding(16)
    .radius(12)
    .children([
        text("Hello"),
        button("Save"),
    ])
```

The goal is to make applications easy for both humans **and AI systems** to understand and construct.

---

## When to Use Beverly

Beverly is particularly well suited to applications where the UI itself is part of the product's infrastructure.

Consider Beverly when you need:

### Local applications

Applications that should run directly on the user's machine or device.

### Resource-constrained environments

Applications where CPU, memory, storage, startup time, or bandwidth matter.

### Data-heavy interfaces

Applications dealing with:

- Large datasets
- Streaming data
- Data visualization
- High-frequency updates
- Tables and grids
- Telemetry
- Scientific or operational data

### AI-native applications

Applications where AI agents need to interact with the same state, events, and tools as human users.

### Sovereign applications

Applications that need to operate:

- Offline
- Inside private infrastructure
- In air-gapped environments
- With sensitive local data
- Alongside local AI models

### Dedicated devices

Applications where a browser runtime is unnecessary overhead.

---

## When Not to Use Beverly

Beverly is not intended to replace the web.

A traditional web application may be the better choice when:

- The application is primarily web-based
- Browser deployment is the primary requirement
- HTML and CSS are already a strong fit
- You need the existing web ecosystem
- You want users to access the application without installing anything
- Your application benefits heavily from existing JavaScript libraries and browser APIs

If a conventional web stack solves the problem well, use it.

Beverly exists for the cases where you want something different:

> **A native, local, high-performance UI runtime designed for data, devices, and AI.**
