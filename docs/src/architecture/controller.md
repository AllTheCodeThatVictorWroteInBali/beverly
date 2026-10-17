# 3.5 Controller

The **Controller connects Messages to application behavior**.

It answers a simple question:

> **When this Event occurs, what should happen?**

A Controller maps an Event to one or more Model methods or other application actions.

```rust
controller! {
  UserController {
    User::Create => create_user,
    User::Delete => delete_user,
  }
}
```

The macro generates a Bevy `Plugin`. Each handler receives a reference to the
message type it handles:

```rust
fn create_user(message: &User::Create) {
  info!(name = %message.name, "creating user");
}

fn delete_user(message: &User::Delete) {
  info!(id = message.id, "deleting user");
}
```

Register the messages and install the generated plugin explicitly:

```rust
app.add_message::<User::Create>()
  .add_message::<User::Delete>()
  .add_plugins(UserController);
```

`controller!` routes messages only. It does not make a view emit messages or
invent application state transitions.

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

`controller!` supports three composable route forms:

| Route | Behavior |
|---|---|
| `Message => handler` | Calls `handler(&Message)`; its return value is ignored, preserving single-handler behavior. |
| `Message => [first, next, last]` | Runs actions sequentially, passing each successful output to the next action. |
| `Message => {left, right}` | Runs independent branches through Bevy's scoped compute pool, waits for all, and returns their outputs as a tuple. |

Sequences and fan-out groups can nest recursively. Groups must contain at least
one action; trailing commas are accepted. Composed actions return `Result`.
The first action receives `&Message`; later actions receive their predecessor's
owned output. Fan-out outputs form a tuple in declaration order, not completion
order. Even a one-branch group produces a one-element tuple `(value,)`.

## Complete Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

event! { Document::Process { id: u32 } }

#[derive(Clone)]
struct LoadedDocument {
  id: u32,
  text: String,
}

fn load(message: &Document::Process) -> Result<LoadedDocument, &'static str> {
  Ok(LoadedDocument { id: message.id, text: "Report content".to_owned() })
}

fn summarize(document: LoadedDocument) -> Result<String, &'static str> {
  Ok(document.text)
}

fn index(document: LoadedDocument) -> Result<u32, &'static str> {
  Ok(document.id)
}

fn save((summary, id): (String, u32)) -> Result<(), &'static str> {
  info!(id, summary, "document processed");
  Ok(())
}

controller! {
  DocumentController {
    Document::Process => [load, { summarize, index }, save],
  }
}

fn main() {
  let mut app = App::new();
  app.add_message::<Document::Process>().add_plugins(DocumentController);
  app.world_mut().write_message(Document::Process { id: 42 });
  app.update();
}
```

Here `load` receives the message, the parallel branches receive independent
`LoadedDocument` values, and `save` receives `(String, u32)` after both succeed.
The compute pool is initialized on first use if the application has not
initialized it through Bevy's normal task-pool setup.

## Sequential Pipelines

Multiple actions can be composed into a **series** using an array:

```rust
controller! {
  DocumentController {
    Document::Process => [load, parse, summarize, save],
  }
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
fn load(message: &Document::Process) -> Result<LoadedDocument, Error>
fn parse(document: LoadedDocument) -> Result<ParsedDocument, Error>
fn summarize(document: ParsedDocument) -> Result<Summary, Error>
fn save(summary: Summary) -> Result<(), Error>
```

The output of each step must be compatible with the input of the next.

Sequential steps move values; intermediate values do not need to implement
`Clone`. Use ordinary functions or associated function paths, such as
`DocumentModel::load`. These are function calls, not Bevy systems with injected
`Res`, `ResMut`, or `Commands` parameters. Obtain application dependencies
explicitly through the message or values passed between actions.

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

Sequential stages can use a common error type or convert earlier errors into
the last stage's error type through `From`, as with ordinary Rust `?`.
Fan-out branches must share one error type. The composed route's error must
implement `Display`: Beverly logs it with the controller and message type,
then continues processing later messages. The final successful value is
discarded; emit a follow-up message explicitly when the application needs a
new workflow boundary. A plain `Message => handler` route still ignores its
return value; use `[handler]` for a one-stage route with composed error logging.

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
  UserController {
    User::Created => { notify_user_created, record_analytics, inform_agent },
  }
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

Each branch above receives `&User::Created` and returns `Result<Output, Error>`.
Borrowing the same message does not require the message itself to be `Clone`.
For an owned intermediate value, fan-out clones the input for all but one
branch, so that input must implement `Clone`. Use `Arc` for shared read-only
data when cloning a large payload would be expensive.

Fan-out uses Bevy's scoped compute pool and waits for every branch to finish.
Captured values and results must be `Send`; shared borrows must be `Sync`.
Native builds support simultaneous execution when pool workers are available.
Single-threaded browser builds preserve tuple and error semantics but do not
execute synchronous branch bodies simultaneously. Branches must not rely on
another branch running concurrently to make progress.

Branch completion order is unspecified. Outputs are joined in declaration
order. If branches fail, all siblings are still allowed to finish; there is no
rollback or sibling cancellation. The first error in declaration order is
propagated and any following sequential stage is skipped. Protect shared
mutable state explicitly and keep branch side effects independent.

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
  DocumentController {
    Document::Process => [load, { index, analyze, metadata }, save],
  }
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

In this example, `save` receives `(IndexOutput, AnalysisOutput, MetadataOutput)`
as one tuple argument. For a sequence inside one branch, write:

```rust
controller! {
  DocumentController {
    Document::Process => [load, { [parse, summarize], index }, save],
  }
}
```

That `save` receives `(Summary, IndexOutput)`. Fan-out inside fan-out produces
nested tuples; it does not flatten them. Each nested sequence preserves its
own order while independent branches can overlap.

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

Controller composition is synchronous from the route's perspective: the
controller's `Update` system waits for the pipeline and all parallel branches.
It accepts functions returning `Result`, not `async fn` or detached jobs. Long
CPU work can still block a UI update even when it uses multiple worker threads.
For long-running inference, network I/O, or cancellable work, use a worker and
explicit request/response messages rather than treating fan-out as background
execution.

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
  DocumentController {
    Document::Processed => save_document,
  }
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
