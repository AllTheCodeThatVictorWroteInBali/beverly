# 3.5 Controller

The **Controller connects Events to application behavior**.

It answers a simple question:

> **When this Event occurs, what should happen?**

A Controller maps an Event to one or more Model methods or other application actions.

```rust
controller! {
    User::Create => Model::User::create,
    User::Delete => Model::User::delete,
}
```

The Event defines the contract.

The Controller defines what the application does with that contract.

The Model owns the data and behavior.

```text
Event
  ↓
Controller
  ↓
Action
  ↓
Model
```

Controllers are intentionally declarative.

You should be able to glance at one and understand the application's behavior without tracing through a large amount of framework code.

## Sequential Pipelines

Multiple actions can be composed into a **series** using an array:

```rust
controller! {
    Document::Process => [
        Document::load,
        Document::parse,
        Document::summarize,
        Document::save,
    ],
}
```

The actions execute in order.

Most importantly, **the return value from one action becomes the input to the next action**.

```text
DocumentId
    ↓
Document::load
    ↓
Document
    ↓
Document::parse
    ↓
ParsedDocument
    ↓
Document::summarize
    ↓
Summary
    ↓
Document::save
    ↓
()
```

There is no special pipeline object or custom data-passing mechanism.

The Controller composes ordinary function signatures.

Conceptually:

```rust
load(DocumentId) -> Result<Document, Error>

parse(Document) -> Result<ParsedDocument, Error>

summarize(ParsedDocument) -> Result<Summary, Error>

save(Summary) -> Result<(), Error>
```

The output of each step must be compatible with the input of the next.

If it isn't, Rust catches the problem at compile time.

> **If the pipeline doesn't type-check, it doesn't build.**

## Efficient Data Flow

Rust's ownership model also makes these pipelines extremely efficient.

A pipeline step receives the data it needs, performs its work, and passes the result to the next step.

The pipeline does not need to maintain a collection of long-lived objects waiting for a garbage collector to eventually clean them up.

Data can be moved through the pipeline with clear ownership and deterministic lifetime.

```text
Step A
  │
  │ owns value
  ▼
Step B
  │
  │ owns value
  ▼
Step C
  │
  │ owns value
  ▼
Step D
```

When a value is no longer needed, its lifetime ends according to Rust's ownership rules.

There is no tracing garbage collector periodically scanning the application's object graph.

That matters for high-throughput applications.

Instead of building a large managed object graph and waiting for garbage collection, Beverly pipelines can move data through a predictable sequence of operations with deterministic resource management.

This is particularly valuable when processing:

- Streaming data
- Large datasets
- Telemetry
- AI inference results
- Documents
- Images
- Financial data
- Device data

The framework can orchestrate the pipeline while Rust handles the underlying memory management.

> **Data flows through the pipeline. It doesn't have to live there.**

This is one of the reasons Beverly can pursue high-performance application infrastructure without exposing developers to complicated memory-management APIs.

## Errors Break the Chain

Controller pipelines use standard Rust `Result`.

There is no special Beverly error-flow abstraction to learn.

```rust
fn parse(document: Document)
    -> Result<ParsedDocument, DocumentError>
```

If an action returns:

```rust
Ok(value)
```

the value continues to the next action.

If it returns:

```rust
Err(error)
```

the current chain stops.

```text
Action A
   ↓
Action B
   ↓
Err(error)
   ↓
STOP
```

This uses a mechanism Rust developers already understand.

Beverly doesn't need to reinvent `Result`, error propagation, or failure semantics.

> **Successful values flow forward. Errors stop the chain.**

## Parallel Fan-Out

Not every action needs to happen in sequence.

A Controller can fan an input out to multiple actions in **parallel** using braces:

```rust
controller! {
    User::Created => {
        Notification::User::created,
        Analytics::User::created,
        Agent::User::created,
    },
}
```

Conceptually:

```text
             UserCreated
                  │
          ┌───────┼───────┐
          ↓       ↓       ↓
   Notification Analytics Agent
```

The actions can execute independently.

This is useful when multiple systems need to react to the same Fact and there is no reason for them to wait on one another.

The distinction is deliberately visual:

```rust
[ A, B, C ]    // series
{ A, B, C }    // parallel
```

One means **do these things in order**.

The other means **fan this work out**.

## Series and Parallel Can Be Combined

Series and parallel operations can be composed.

```rust
controller! {
    Document::Process => [
        Document::load,
        {
            Document::index,
            Document::analyze,
            Document::extract_metadata,
        },
        Document::save,
    ],
}
```

The conceptual flow is:

```text
              load
               │
               ▼
        ┌──────┼──────┐
        ↓      ↓      ↓
      index analyze metadata
        └──────┼──────┘
               ↓
              save
```

A parallel stage can produce a result that becomes the input to the next stage.

This means:

```text
Series
   ↓
Parallel
   ↓
Series
```

can remain part of the same straightforward data-flow model.

The framework handles the orchestration.

The developer describes the relationships.

## Type Safety Across the Pipeline

Because each action has a typed input and output, the entire pipeline can be checked before the application runs.

For example:

```text
load()
  → Document

parse(Document)
  → ParsedDocument

summarize(ParsedDocument)
  → Summary
```

is valid.

But:

```text
load()
  → Document

summarize(ParsedDocument)
  → Summary
```

cannot be connected directly.

The types don't match.

Rust catches the incompatible pipeline at compile time.

This is one of the places where Beverly's use of Rust becomes particularly powerful:

> **The compiler becomes a pipeline validator.**

Developers don't need to manually inspect every connection.

AI-generated pipelines don't have to be trusted blindly.

The type system verifies that the data actually flows between the operations as described.

## Async Is Explicit

Asynchronous operations are intentionally modeled differently.

An asynchronous operation is not treated as one magical Controller step that hides its lifecycle.

It is two operations:

1. **Issue the request.**
2. **Listen for the response.**

For example:

```text
Command: Document::Process
        ↓
    start work
        ↓
   ... async ...
        ↓
Fact: Document::Processed
```

The first operation initiates the work.

The resulting Fact represents the response.

Another Controller can listen for that Fact:

```rust
controller! {
    Document::Processed => Document::save,
}
```

This keeps asynchronous behavior consistent with Beverly's Event architecture.

There isn't a hidden callback chain buried inside a single pipeline expression.

The application can see the request.

The application can see the response.

The Event history can see both.

The AI can see both.

## Prefer Events for Long-Running Flows

Controllers are powerful, but they shouldn't become enormous chains of application logic.

For short, tightly coupled operations, a series is ideal:

```rust
[A, B, C]
```

For longer or conceptually separate workflows, prefer breaking the chain with a new Event.

Instead of:

```text
A → B → C → D → E → F → G → H
```

consider:

```text
A → B → Fact
          ↓
          C → D → Fact
                    ↓
                    E → F
```

The Event becomes a natural boundary between stages.

This provides:

- Smaller Controllers
- Easier testing
- Clearer application behavior
- Better observability
- Natural asynchronous boundaries
- Better AI discoverability
- Easier replay and debugging

> **Use pipelines for composition. Use Events for boundaries.**

## Controllers as Application Wiring

The Controller is intentionally not where application logic lives.

It connects the pieces.

```text
             Event
               │
               ▼
          Controller
               │
          ┌────┴────┐
          ↓         ↓
       Series    Parallel
          │         │
          └────┬────┘
               ↓
             Model
```

The Model owns the domain behavior.

The Event defines the contract.

The Controller determines how those pieces connect.

That separation makes the Controller easy to read, easy to generate, and easy to test.

And because the connections are typed, Rust can validate both the **logic and data flow** before the application ever runs.

> **The Controller is the wiring diagram for your application.**

You describe what connects to what.

Beverly handles the orchestration.

Rust makes sure the connections make sense.
