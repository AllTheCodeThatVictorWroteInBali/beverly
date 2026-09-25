# Application Structure

Beverly applications are organized around a simple idea:

> **Every concern has a place, and every place has a clear responsibility.**

The goal is not architectural complexity. It is **separation without ceremony**.

A developer should be able to open an unfamiliar Beverly application and quickly answer:

- Where does the data live?
- Where can that data change?
- What does the interface look like?
- What actions can the application perform?
- What happens when an event occurs?
- Where should I make this change?

The architecture makes those answers predictable.

```text
Application
│
├── Model
│   └── Owns application data and domain rules
│
├── View
│   └── Presents data and collects interaction
│
├── Events
│   └── Define what happened and what can happen
│
└── Controller
    └── Connects events to application actions
```

This is the application's vocabulary.

## One Responsibility, One Place

Beverly deliberately separates **data, presentation, behavior, and communication**.

The Model owns the data.

The View presents the data.

Events describe what happened or what should happen.

The Controller connects those events to actions.

That means a change should usually have an obvious destination.

**Changing how data works?** Look in the Model.

**Changing how something looks?** Look in the View.

**Adding an application action?** Define a Command.

**Responding to something that happened?** Listen for its Fact.

**Changing what happens in response?** Look in the Controller.

You don't need to search through a giant application file to discover where behavior lives.

## A Structure LLMs Can Understand

Modern applications are increasingly written with AI.

That changes what good architecture means.

A codebase may be edited by one developer, several developers, or multiple LLMs working simultaneously. The architecture should make it difficult for one part of the system to accidentally interfere with another.

Beverly uses explicit boundaries to make this possible.

For example, an LLM working on the View should primarily be concerned with:

```text
View
 ↓
Components
 ↓
Events
```

An LLM working on the Model should primarily be concerned with:

```text
Model
 ↓
Application Data
 ↓
Domain Rules
```

An LLM working on Controllers should primarily be concerned with:

```text
Events
 ↓
Actions
 ↓
Model / Services
```

The boundaries give both humans and machines a smaller area to reason about.

> **The smaller the surface of a change, the easier the change is to understand, review, and verify.**

## Multiple LLMs, One Application

Beverly is designed for a development environment where multiple agents may work on the same application.

One LLM might be building a new component.

Another might be implementing a database integration.

Another might be adding an agent action.

Another might be improving accessibility.

They should not all need to understand the entire application.

Because concerns are separated, their work can remain localized.

```text
             Application
                  │
      ┌───────────┼───────────┐
      │           │           │
     View        Model      Controller
      │           │           │
   Components   Data       Behavior
      │           │           │
      └─────── Events ────────┘
```

The important property is that **one agent does not need to modify another agent's area of responsibility to accomplish its own task**.

A View agent should not need to rewrite Model state.

A Model agent should not need to restructure the UI.

A Controller agent should not need to duplicate application logic inside event handlers.

The architecture creates boundaries between pieces of work.

This reduces accidental overwrites, duplicated logic, and changes made in the wrong place.

## Explicit State Ownership

State ownership is especially important when multiple agents are generating code.

Beverly does not encourage every component to invent its own source of truth.

The Model owns application data.

The Controller determines how application actions are connected.

The View reflects the resulting state.

```text
          Model
        owns state
            │
            ▼
          View
      reflects state
            │
            ▼
          Event
      expresses intent
            │
            ▼
       Controller
      connects action
            │
            ▼
          Model
```

This creates a predictable direction of travel.

The View does not quietly become another Model.

A component does not secretly become the owner of application state.

A Controller does not become a second database.

There is one obvious place to look for the authoritative state.

> **One source of truth. Clear ownership. Predictable flow.**

That matters even more when code is being generated or modified by machines.

## Easy to Find. Easy to Change.

A good architecture should reduce the amount of code you need to read before making a change.

Suppose you want to change the way users are displayed.

You should primarily work in the View.

```text
users/
├── model.rs
├── view.rs
├── events.rs
└── controller.rs
```

Suppose you want to change how users are stored.

Look at `model.rs`.

Suppose you want to add:

```text
User::Delete
```

Look at the Events and Controller.

Suppose you want deleting a user to trigger an audit record.

Add the appropriate event/action connection rather than embedding auditing logic inside an unrelated component.

The structure itself becomes a map of the application.

## Local Reasoning

Beverly favors **local reasoning**.

You should be able to understand a component without understanding the entire application.

You should be able to understand a Model without loading the renderer into your head.

You should be able to understand a Controller without understanding every screen.

You should be able to understand an Event without knowing which View generated it.

This makes code review dramatically simpler.

It also makes AI-generated code easier to evaluate.

An LLM can generate a change inside a constrained boundary.

A human can inspect that boundary and determine whether the change makes sense.

The architecture doesn't eliminate mistakes.

It makes mistakes **easier to locate**.

## A Predictable Project

A Beverly project can remain simple even as the application grows.

```text
src/
├── main.rs
│
├── model/
│   ├── user.rs
│   ├── document.rs
│   └── settings.rs
│
├── view/
│   ├── users.rs
│   ├── documents.rs
│   └── settings.rs
│
├── events/
│   └── mod.rs
│
└── controller/
    └── mod.rs
```

The exact directory structure is not the important part.

The important part is that the conceptual structure remains stable.

```text
Data       → Model
Interface  → View
Vocabulary → Events
Wiring     → Controller
```

A small application and a large application can use the same mental model.

## Architecture as a Contract

Beverly's architecture is intentionally boring in the best possible way.

There are no hidden state machines that developers have to learn.

No giant application controller.

No requirement that every piece of data pass through a proprietary state-management abstraction.

No reason for every component to know about every other component.

Instead, Beverly establishes a small set of contracts:

```text
Model
  owns data

View
  presents data

Event
  describes intent or reality

Controller
  connects intent to behavior
```

Once those contracts are understood, the rest of the application becomes composition.

This is particularly important for AI-native development.

An LLM doesn't need to understand an entire codebase to make a useful change.

It needs to understand the relevant contract, make the change inside that boundary, and follow the existing vocabulary.

## The Application Becomes a Map

The ultimate goal is **predictability**.

When something doesn't work, you know where to look.

When something needs to change, you know where to make the change.

When an LLM generates code, you know where that code belongs.

When multiple LLMs work simultaneously, their responsibilities can remain separated.

When a new developer joins the project, the architecture itself explains how the application works.

> **The codebase should tell you where to look.**

That's the point of Beverly's application structure.

Not abstraction for its own sake.

Not architecture as ceremony.

**Clear concerns. Clear ownership. Clear boundaries. Clear code.**

### The Beverly Rule

If you are unsure where something belongs, ask what the code is responsible for:

**Does it own data?** → Model

**Does it present data?** → View

**Does it describe something happening or something you want to happen?** → Event

**Does it connect an event to behavior?** → Controller

If the answer is still unclear, the code may be doing too much.

> **Simple enough for a human to navigate. Structured enough for multiple machines to work on simultaneously.**
