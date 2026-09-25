# 2. Quick Start

Get your first Beverly application running in minutes.

In this guide, we'll start with an empty Rust project and build a small interactive application from the ground up.

By the end, you'll have:

- A Bevy application
- Beverly UI components
- A reusable Button
- A Rust Model containing application data
- An application Event
- A Controller connecting the Event to the Model
- A UI that reacts to changing application state
- A distributable application binary

The complete flow we'll build is:

```text id="r4q7cy"
Button
  ↓
Event
  ↓
Controller
  ↓
Model
  ↓
Application Data
  ↓
View
```

Beverly keeps application state close to ordinary Rust: the Model owns the underlying data, Events describe what happens, and Controllers connect Events to application behavior.

---

# Installation

Beverly is a Rust framework built on top of Bevy, so you'll need a working Rust development environment.

## Install Rust

If you don't already have Rust installed, install it using `rustup`.

Verify your installation:

```bash id="7n0gup"
rustc --version
cargo --version
```

You should see the installed Rust and Cargo versions.

---

## Install Bevy

A Beverly application is a Bevy application, so Bevy is the runtime underneath Beverly.

Add Bevy to your project with Cargo:

```bash id="n6x0mw"
cargo add bevy
```

Or add it directly to `Cargo.toml`:

```toml id="b4ajq5"
[dependencies]
bevy = "..."
```

Use the version recommended by the Beverly release you're using.

---

## Install Beverly

Add Beverly to your project:

```bash id="l5l1ph"
cargo add beverly
```

Or add it directly to `Cargo.toml`:

```toml id="p6b5d1"
[dependencies]
bevy = "..."
beverly = "..."
```

Beverly provides the application UI layer while Bevy provides the underlying runtime and rendering infrastructure.

---

# Create a Project

Create a new Rust binary:

```bash id="gk3xfr"
cargo new hello_beverly
cd hello_beverly
```

Your project starts with:

```text id="p6v7wh"
hello_beverly/
├── Cargo.toml
└── src/
    └── main.rs
```

At this point you have a normal Rust application.

That's intentional.

Beverly doesn't require a special project format or separate application runtime.

---

# Add Beverly

Add Bevy and Beverly:

```bash id="9c2m2s"
cargo add bevy
cargo add beverly
```

Your `Cargo.toml` now contains the dependencies required by your application.

The exact versions will depend on the Beverly release you're using.

---

# Hello World

Let's start with the smallest possible Beverly application.

```rust id="j4lqf9"
use bevy::prelude::*;
use beverly::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(
        text("Hello, Beverly!")
    );
}
```

Run it:

```bash id="0j5wte"
cargo run
```

A window should appear with your first Beverly UI.

That's it.

You now have a native application using Rust, Bevy, and Beverly.

---

# Your First Component — A Button

Let's add an interactive Button.

```rust id="l7j8pn"
button("Click Me")
```

The Button is not a one-off piece of UI.

**Beverly components are reusable assets.**

Once you have a Button, you can use that same component throughout your application:

```rust id="0pmf2m"
button("Save")

button("Cancel")

button("Delete")

button("Create User")
```

You can also compose components into larger reusable UI structures:

```rust id="0j38qj"
card()
    .padding(16)
    .radius(12)
    .children([
        text("Welcome to Beverly"),
        button("Get Started"),
    ])
```

This is the foundation of Beverly's component model:

**Build a primitive once. Compose and reuse it everywhere.**

Components can encapsulate their own presentation and interaction behavior while remaining composable with the rest of the application.

---

# Your First Model

The Model is where your application's underlying data lives.

A Model is just Rust.

For this example, we'll keep track of how many times our Button has been clicked:

```rust id="4i3z9k"
struct AppModel {
    clicks: u32,
}
```

The Model owns this data.

Expose it through getters and setters:

```rust id="9c5h9v"
impl AppModel {
    fn clicks(&self) -> u32 {
        self.clicks
    }

    fn set_clicks(&mut self, clicks: u32) {
        self.clicks = clicks;
    }

    fn increment_clicks(&mut self) {
        self.set_clicks(self.clicks() + 1);
    }
}
```

The underlying data stays inside the Model.

The View reads it through getters:

```rust id="5v7j0x"
model.clicks()
```

Application behavior changes it through setters or Model methods:

```rust id="4p5d2s"
model.increment_clicks();
```

The Model is therefore the source of truth for application state.

---

# Your First Event

Now define the Event that represents the user's action:

```rust id="9f2k4m"
event! {
    App::Clicked
}
```

The Event represents:

> "The user clicked."

Events are the vocabulary of your application.

The same Event can be produced by different sources:

- A Button
- A keyboard shortcut
- An API
- Another application
- An AI agent

The source doesn't need to know what happens next. It emits the Event.

---

# Your First Interactive UI

Now connect the Button to the Event:

```rust id="q8s1nd"
button("Click Me")
    .on("click", Event::App::Clicked)
```

That's the Button's entire application-level interaction contract:

```text id="6w3t4k"
click
  ↓
Event::App::Clicked
```

The Controller decides what happens next:

```rust id="4k8d6p"
controller! {
    App::Clicked => Model::App::increment_clicks,
}
```

The complete flow is:

```text id="b8f5g1"
┌─────────────┐
│    Button   │
│             │
│  on click   │
└──────┬──────┘
       │
       ▼
Event::App::Clicked
       │
       ▼
┌─────────────┐
│ Controller  │
│             │
│ Event →     │
│ Action      │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│    Model    │
│             │
│ Owns Data   │
└─────────────┘
```

The Button doesn't need to know about the Model.

The Model doesn't need to know about the Button.

The Controller connects them through the Event.

---

# Reading Model Data

Now display the number of clicks in the UI.

The View reads the Model through its getter:

```rust id="u1m2c9"
text(format!("Clicks: {}", model.clicks()))
```

The Model remains the source of truth:

```text id="k3z8ha"
             Model
               │
          owns application
              data
               │
               ▼
             Getter
               │
               ▼
              View
```

When the Model changes, the View sees the updated state on the next frame.

---

# State Updates at 60 FPS

Beverly's UI is continuously updated as part of Bevy's rendering loop.

At 60 frames per second, the application is evaluating and rendering the current state of the UI roughly every 16.7 milliseconds.

That means a Model change can be reflected in the UI almost immediately.

```text id="x7h4kw"
Model changes
     ↓
Next update
     ↓
View reads current state
     ↓
Render
```

There is no need to manually synchronize a separate UI state store with your application state.

The UI is a projection of the current Model state.

This makes state changes feel immediate and keeps the mental model simple:

> **Change the Model. The UI reflects the Model.**

---

# Updating Model Data

When application behavior needs to change state, it goes through the Model's public API.

For example:

```rust id="v4m7q2"
impl AppModel {
    fn set_clicks(&mut self, clicks: u32) {
        self.clicks = clicks.min(100);
    }
}
```

Now the Model can enforce its own invariants.

Every caller gets the same behavior.

The UI doesn't need to know that clicks are limited to 100.

The Controller doesn't need to know.

The Model owns the rule because the Model owns the data.

---

# State Management

Beverly uses ordinary Rust for application state.

Your Model is the state layer:

```rust id="r5j2x8"
struct AppModel {
    clicks: u32,
}
```

There is no need to introduce another state-management abstraction simply to keep the UI synchronized with your application.

The basic relationship is:

```text id="n4v8xq"
        Model
          │
       owns data
          │
          ▼
         View
```

Events handle interaction:

```text id="s8k3jm"
View
 ↓
Event
 ↓
Controller
 ↓
Model
```

Together, these form Beverly's application state model.

---

# Putting It Together

A small Beverly application can therefore look like this:

```rust id="z3h7vn"
struct AppModel {
    clicks: u32,
}

impl AppModel {
    fn clicks(&self) -> u32 {
        self.clicks
    }

    fn set_clicks(&mut self, clicks: u32) {
        self.clicks = clicks;
    }

    fn increment_clicks(&mut self) {
        self.set_clicks(self.clicks() + 1);
        println!("Clicks: {}", self.clicks());
    }
}

event! {
    App::Clicked
}

controller! {
    App::Clicked => Model::App::increment_clicks,
}
```

And the View:

```rust id="6h5r9x"
card()
    .padding(16)
    .radius(12)
    .children([
        text(format!("Clicks: {}", model.clicks())),
        button("Click Me")
            .on("click", Event::App::Clicked),
    ])
```

The architecture is:

```text id="c4q9sv"
View
 │
 │ Event
 ▼
Controller
 │
 │ Action
 ▼
Model
 │
 │ owns
 ▼
Application Data
 │
 ▼
View
```

The Button is reusable.

The Model is ordinary Rust.

The Event defines the application's vocabulary.

The Controller connects Events to behavior.

The View projects the current state.

And because Beverly continuously renders the application at 60 FPS, changes to that state can be reflected immediately.

---

# Running the App

Run the application during development with:

```bash id="q8f4ml"
cargo run
```

For faster iteration, use:

```bash id="j5w8xp"
cargo check
```

when you only need to verify that the project compiles.

Build an optimized binary with:

```bash id="r2x9nk"
cargo build --release
```

The resulting application will be placed in Cargo's standard release directory:

```text id="8d3k5w"
target/release/
```

---

# Project Structure

As your application grows, keep the application organized around its responsibilities.

A simple project might look like:

```text id="v6p2qn"
hello_beverly/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── model/
│   │   └── app.rs
│   ├── view/
│   │   └── app.rs
│   ├── events/
│   │   └── app.rs
│   └── controller/
│       └── app.rs
└── assets/
```

For a larger application:

```text id="y9q4bk"
src/
├── main.rs
├── model/
│   ├── user.rs
│   ├── document.rs
│   └── application.rs
├── view/
│   ├── dashboard.rs
│   ├── users.rs
│   └── documents.rs
├── events/
│   ├── user.rs
│   └── document.rs
└── controller/
    └── application.rs
```

There is no requirement to structure every project this way.

For a small application, a single `main.rs` may be perfectly reasonable.

The architecture should scale with the application, not force ceremony onto it.

---

# Cargo Beverly

Beverly provides a Cargo-oriented workflow for developing and distributing applications.

The goal is for Beverly commands to feel like normal Cargo commands while adding Beverly-specific application packaging capabilities.

## Build

During development:

```bash id="f7m2qc"
cargo beverly build
```

This provides a Beverly-oriented build workflow while still producing a normal Rust application.

You can still use:

```bash id="k3n8vp"
cargo build
```

because Beverly is ultimately a Rust application.

---

# Building for Distribution

When you're ready to distribute your application:

```bash id="x4m7st"
cargo beverly publish
```

The publish command builds the application in release mode and produces the artifacts needed for distribution.

For example:

```bash id="h8q2vc"
cargo beverly publish --target macos
```

The workflow becomes:

```text id="p5j9rn"
cargo beverly build
        ↓
Development

cargo beverly publish
        ↓
Distribution
```

Platform support can be extended over time:

```bash id="n7v3kx"
cargo beverly publish --target macos
cargo beverly publish --target windows
cargo beverly publish --target linux
```

The application remains a normal Rust project throughout the process.

---

# What You Just Built

You've now built a complete Beverly application.

The architecture is:

```text id="w6k2qm"
                    ┌─────────────┐
                    │    View     │
                    │             │
                    │   Button    │
                    └──────┬──────┘
                           │
                           │ Event
                           ▼
                    ┌─────────────┐
                    │ Controller  │
                    │             │
                    │ Event →     │
                    │ Action      │
                    └──────┬──────┘
                           │
                           │ Action
                           ▼
                    ┌─────────────┐
                    │    Model    │
                    │             │
                    │  Owns Data  │
                    │             │
                    │  Getters    │
                    │  Setters    │
                    └──────┬──────┘
                           │
                           │ Current State
                           ▼
                    ┌─────────────┐
                    │    View     │
                    │             │
                    │  60 FPS     │
                    └─────────────┘
```

The core idea is simple:

> **The Model is your state. Events are your application vocabulary. Controllers connect Events to behavior. Views project the current state.**

The Button you built today can be reused throughout the application.

The Model you wrote is ordinary Rust.

And the same Event system that handles your first Button can eventually connect forms, data interfaces, background processes, APIs, and AI agents.

That's the foundation of Beverly.

---

## Next

Now that you have a working application, the next chapter explains the architecture behind it:

**Model → View → Controller → Events**

You'll learn how Beverly represents application state, how Events become application contracts, how Controllers connect Events to behavior, and how the same architecture enables human and AI interaction.
