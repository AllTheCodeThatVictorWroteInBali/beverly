# AI Workloads, Background Workers & Channels

Beverly does not need a special AI programming model. An AI workload is simply another piece of application work. It may take a long time to complete, it may produce results incrementally, and it may need to communicate with the rest of the application while it is running, but the underlying problem is not unique to AI. Rust, Bevy, and ordinary application architecture already provide the tools needed to handle it.

The most important rule is to keep long-running work away from the main UI loop. Inference, RAG retrieval, embedding generation, document processing, network requests, and tool calls can all take enough time to make a UI unresponsive if they are performed directly as part of the main application work. Instead, run that work in the background and communicate the results back to the application.

A simple architecture looks like this:

```text id="77sde9"
AI Worker → Channel → Model → View
```

The worker performs the expensive work, a standard Rust channel carries the results back to the application, and the Model becomes the owner of those results. On the next Bevy tick, the View reflects the updated Model state automatically. The AI code does not need to know how the UI is rendered, and the UI does not need to know how inference was performed.

## Background Workers

Bevy already provides mechanisms for running work outside the main application loop, while Rust provides standard concurrency primitives such as channels. Beverly should use these existing tools rather than recreate them as another abstraction. Bevy provides the runtime, Rust provides the concurrency primitives, and the application can use whatever AI library, local model, remote service, vector database, or other technology makes sense for the workload.

For example, an AI worker might perform inference and send partial results through a channel:

```rust id="up9r97"
enum AiMessage {
    Token(String),
    Complete,
    Error(String),
}
```

As the worker receives tokens, it can send them back to the application:

```rust id="tccvml"
tx.send(AiMessage::Token(token))?;
```

The application receives those messages and updates the Model:

```rust id="kd4x2r"
while let Ok(message) = rx.try_recv() {
    match message {
        AiMessage::Token(token) => {
            chat.append_response(&token);
        }

        AiMessage::Complete => {
            chat.set_status(ChatStatus::Complete);
        }

        AiMessage::Error(error) => {
            chat.set_error(error);
        }
    }
}
```

The important part is what happens after the message is received. The response is immediately saved into the Model, and on the next tick the View sees the new state and updates automatically. There is no need to manually find a text component and update it every time a token arrives. Streaming is simply the normal Model → View relationship happening repeatedly as new data becomes available.

## Streaming AI Responses

Streaming therefore does not require a special UI architecture. It is simply a case where the Model changes incrementally instead of all at once.

Suppose the Model contains the current response:

```rust id="voh2jc"
struct Chat {
    response: String,
}
```

The Model can expose ordinary getters and setters:

```rust id="fbj2qt"
impl Chat {
    fn get_response(&self) -> &str {
        &self.response
    }

    fn set_response(&mut self, response: String) {
        self.response = response;
    }

    fn append_response(&mut self, token: &str) {
        self.response.push_str(token);
    }
}
```

The straightforward getter/setter approach makes the architecture particularly clear. Conceptually, each new token becomes part of the existing response:

```rust id="1rg4xd"
chat.set_response(
    chat.get_response().to_owned() + &token
);
```

For an efficient implementation, the Model can instead expose an append operation:

```rust id="wruan7"
chat.append_response(&token);
```

Both approaches express the same architectural principle: the AI response belongs to the Model. The append method is simply a more efficient way of implementing the same state transition without repeatedly constructing a new String.

The View only needs to read the current state:

```rust id="aiu1x9"
text(chat.get_response())
```

The AI worker should not reach into the UI and call `set_text()` every time it receives a token. `set_text()` is a normal component-level setter and is useful when configuring or directly modifying a text component:

```rust id="y76bjd"
text("Hello")
    .set_text("Hello, world!");
```

That is different from application state. For application data, the response belongs to the Model, and the normal path remains AI result → Model setter → View. This distinction becomes increasingly valuable as an application grows because the same response can be displayed in multiple places, persisted, inspected, or consumed by other parts of the application without being trapped inside a UI component.

## AI Work Is Still Application Work

Once this pattern is established, more complicated AI workloads do not require a different architecture. A RAG workflow might generate an embedding, search a vector index, retrieve documents, construct context, invoke an LLM, and stream the resulting response. Those steps can all happen in ordinary Rust code, with the channel carrying meaningful results back to the application as they become available.

For example:

```rust id="dcvuir"
enum AiMessage {
    Searching,
    ContextFound(Vec<Document>),
    Token(String),
    Complete,
    Error(String),
}
```

The application can translate those messages into Model state:

```rust id="o9err9"
while let Ok(message) = rx.try_recv() {
    match message {
        AiMessage::Searching => {
            chat.set_status(ChatStatus::Searching);
        }

        AiMessage::ContextFound(documents) => {
            chat.set_context(documents);
        }

        AiMessage::Token(token) => {
            chat.append_response(&token);
        }

        AiMessage::Complete => {
            chat.set_status(ChatStatus::Complete);
        }

        AiMessage::Error(error) => {
            chat.set_error(error);
        }
    }
}
```

The View remains ordinary Beverly code:

```rust id="9zhhww"
chat_view(chat)
```

Nothing about RAG requires the UI to become an AI-specific UI. The AI performs the work and produces results; the application decides how those results become state and how that state is presented.

This is an important Beverly principle: **AI should fit into the application, rather than forcing the application to become an AI framework.**

## AI Tool Calls

The same principle applies when an AI needs to take an action. Suppose the application already exposes a Command for retrieving orders:

```rust id="6ue8d7"
Command::Orders::Get {
    user_id: user.get_id(),
}
```

A human interface might trigger that Command through a Button. An API could trigger the same Command from an external request, and an AI could determine that retrieving the orders is the appropriate next step. There should not need to be three different implementations of "get orders" simply because there are three different sources of the request.

The Command is the application's existing vocabulary for that action, so every participant should use it.

This is where JEV fits naturally into the architecture.

## JEV Chooses From Existing Commands

JEV does not create application behavior. The application defines its available Commands, and JEV provides a structured way for an AI to choose between those Commands.

For example, an application might already define:

```rust id="hufhh8"
Command::Orders::Get
Command::Orders::Cancel
Command::Orders::Refund
Command::User::Notify
```

JEV can choose from those existing actions:

```rust id="cdxbfz"
Jev::choice("next_action", [
    Command::Orders::Get,
    Command::Orders::Cancel,
    Command::Orders::Refund,
])
```

The AI is therefore operating inside the application's existing vocabulary. It is not generating arbitrary code, inventing an API, or creating new application capabilities at runtime. The developer defines what the application is capable of doing, while JEV provides the structured decision that determines which of those capabilities should be used.

This distinction is particularly useful for reasoning about agents. An agent does not necessarily need its own set of tools, execution framework, or application architecture. The application already has actions. The AI needs a way to observe what is happening, decide what should happen next, and request one of the actions that the application already understands.

That gives us a simple principle:

> **Commands define what the application can do. JEV chooses what to do.**

Once JEV has selected a Command, the request follows the same Controller path as any other Command:

```text id="2pgriq"
JEV
 ↓
Command
 ↓
Controller
 ↓
Model
 ↓
View
```

This shared path is also what makes the permission model powerful. The Model is the authoritative owner of application state and the rules governing whether that state is allowed to change. Because requests from humans, APIs, automations, and AI all ultimately reach the same Model, they all encounter the same permission and domain rules.

If the Model determines that an operation is not allowed, it can reject the state transition regardless of who requested it. JEV cannot grant itself additional capabilities simply by choosing a Command, because choosing a Command is not the same thing as being authorized to perform it. The Command expresses the requested action; the Model decides whether that action is valid and permitted against the current application state.

For example, an AI may choose:

```rust id="x8k4m2"
Command::Orders::Refund {
    order_id: order.get_id(),
}
```

The Controller connects that Command to the appropriate Model operation. The Model then makes the authoritative decision about whether the refund is allowed:

```rust id="q5n7cz"
orders.refund(order.get_id())?;
```

The Model can consider whatever rules belong to the application's state and domain: whether the order exists, whether it has already been refunded, whether it is in a refundable state, whether the operation is permitted for the current actor, or any other invariant the application requires.

This means the permission layer does not need to trust the source of the request. It only needs to enforce the rules at the point where state changes.

That gives Beverly an important security property: **the same Model rules apply regardless of whether the request came from a human or an AI.**

The Controller can still handle application-level concerns such as orchestration, confirmation flows, rate limiting, auditing, and other policies that belong outside the Model. But the Model remains the final authority over its own state and the domain rules governing how that state can change.

JEV therefore does not become a privileged pathway around the application. It is simply another way to request an existing Command, and that Command must still pass through the same application machinery as every other request.

## AI and the Event Vocabulary

This becomes even more powerful when combined with Beverly Events. The application already has a vocabulary describing things that happened and things that can be requested.

Facts describe things that have happened:

```rust id="x3u3pr"
User::Created
Order::Cancelled
Document::Indexed
```

Commands describe things that can be requested:

```rust id="9i457j"
User::Create
Order::Cancel
Document::Index
```

An AI can listen to Facts, reason about the current state, and choose an existing Command through JEV. The resulting Command then enters the same Controller that handles Commands from every other source.

Because JEV operates through this existing Event system, AI decisions are automatically part of the application's event history. Beverly does not need a separate AI logging mechanism, a separate replay system, or additional instrumentation just to understand what an agent did. The JEV decision, the resulting Command, and the subsequent application Events can participate in the same history as everything else.

This makes rewinding and replaying AI behavior a natural consequence of the architecture. If the application records its Events, an AI decision that resulted in a Command is already part of that Event stream. Replaying the history can therefore reproduce the sequence of decisions and resulting actions without requiring a second AI-specific development effort.

The same mechanism also makes AI behavior inspectable. A developer can look at the event history and see what happened, which Command JEV selected, what state the application was in, and what happened afterward. When the event system supports rewind and replay, those capabilities apply to AI behavior automatically because the AI is participating in the same event system.

In that sense, observability and replay are not features that have to be bolted onto the agent later. They are inherited from the application's existing architecture.

An agent can therefore be understood as a participant in the application's existing event vocabulary. It listens to what happened, makes a decision, and requests something that the application already knows how to do. The Model remains the authority over whether the requested state transition is actually allowed.

```text id="yahscf"
Listen → Decide → Command
```

JEV provides the structured decision, the Controller provides the application wiring, the Model provides the state and enforces its rules, and the Event system provides the history through which those decisions can be observed, replayed, and rewound. Nothing about this requires a second application architecture for AI.

## Tool Results Become Model State

Once a Command runs, its result follows the same application path as any other application data. If an AI requests the user's orders, for example, the resulting data can simply become Model state:

```rust id="8upbbs"
orders.set_items(result);
```

The View can then render those orders:

```rust id="00vzbq"
table(orders.get_items())
```

The AI does not need to update the table directly. It updates application state, and the View follows that state.

This means AI-generated data, database data, user-entered data, and data returned by external services can all participate in the same Model → View relationship. The source of the data may be completely different, but once it enters the Model, the rest of the application does not need a special mechanism to display it.

The same principle applies to authorization. If the result would cause a state transition, it goes through the Model's rules rather than creating a separate AI-specific path. The Model remains the place where the application decides what is valid and what is permitted.

## AI, JEV, and Beverly

The resulting architecture deliberately keeps each responsibility small. Rust provides concurrency, channels, types, and resource management. Bevy provides the runtime and mechanisms for background execution. AI code performs inference, RAG, retrieval, tool calls, or whatever workload the application requires. Channels carry results from that background work into the application.

The Model owns application state and the rules governing how that state may change. Commands define the actions the application can request, while the Controller connects those actions to the rest of the application. JEV sits alongside these pieces as the structured decision layer for AI: it can choose from the Commands that the application already exposes, but it does not bypass the Model's rules.

Because JEV participates in the same Event system as the rest of the application, its decisions also inherit the application's existing observability, history, replay, and rewind capabilities. There is no separate AI event log to maintain and no additional replay architecture to build simply because an AI is making decisions.

This means a chatbot, RAG system, tool-using agent, streaming inference pipeline, or JEV-driven agent can all be built using the same underlying architecture.

There is no need to turn Beverly into an AI framework to accomplish this. The AI simply becomes another participant in the application.

The worker does the work, the channel carries the result, the Model owns the state and decides whether state changes are allowed, JEV chooses from existing Commands, the Controller connects the action, the Event system records the decision and its consequences, and Beverly reflects the resulting state.

That is the goal: **AI should use the application's architecture rather than replacing it.**
