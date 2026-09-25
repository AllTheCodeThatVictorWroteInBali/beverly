# 2. Quick Start

Get your first Beverly application running in minutes.

In this guide, we'll start with an empty Rust project and build a small interactive application from the ground up.

By the end, you'll have:

- A Bevy application
- Beverly UI components
- A button
- A Rust Model
- An application Event
- A Controller connecting the Event to the Model
- A working interactive UI
- A distributable application binary

The complete flow we'll build is:

```text
Button
  ↓
Event
  ↓
Controller
  ↓
Model
  ↓
Application State
```

The important part is that there is very little framework-specific machinery involved. Beverly is built to let you use ordinary Rust wherever possible.

---

## Installation

Beverly is a Rust framework built on top of Bevy, so you'll need a working Rust development environment.

### Install Rust

If you don't already have Rust installed, install it using `rustup`.

Verify your installation:

```bash
rustc --version
cargo --version
```

You should see the installed Rust and Cargo versions.

---

## Install Bevy

A Beverly application is a Bevy application, so Bevy is the runtime underneath Beverly.

Add Bevy to your project with Cargo:

```bash
cargo add bevy
```

Or add it directly to `Cargo.toml`:

```toml
[dependencies]
bevy = "..."
```

Use the version recommended by the Beverly release you're using.

---

## Install Beverly

Add Beverly to your project:

```bash
cargo add beverly
```

Or add it directly to `Cargo.toml`:

```toml
[dependencies]
bevy = "..."
beverly = "..."
```

Beverly provides the application UI layer while Bevy provides the underlying runtime and rendering infrastructure.

---

# Create a Project

Create a new Rust binary:

```bash
cargo new hello_beverly
cd hello_beverly
```

Your project starts with:

```text
hello_beverly/
├── Cargo.toml
└── src/
    └── main.rs
```

At this point you have a normal Rust application.

That's intentional.

Beverly doesn't require a special project format or a separate application runtime.

---

# Add Beverly

Add Bevy and Beverly:

```bash
cargo add bevy
cargo add beverly
```

Your `Cargo.toml` now contains the dependencies required by your application.

The exact versions will depend on the Beverly release you're using.

---

# Hello World

Let's start with the smallest possible Beverly application.

```rust
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

```bash
cargo run
```

A window should appear with your first Beverly UI.

That's it.

You now have a native application using Rust, Bevy, and Beverly.

---

# Your First Component

Let's make the application interactive.

Create a button:

```rust
button("Click Me")
```

A complete example might look like:

```rust
fn setup(mut commands: Commands) {
    commands.spawn(
        button("Click Me")
    );
}
```

Beverly components are designed to be composable.

For example:

```rust
card()
    .padding(16)
    .radius(12)
    .children([
        text("Welcome to Beverly"),
        button("Click Me"),
    ])
```

Components describe the UI.

Behavior comes from Events and Controllers.

---

# Your First Model

A Model is just Rust.

You don't need a special state-management framework to create one.

Start with a simple struct:

```rust
struct AppModel {
    clicks: u32,
}
```

We can give it ordinary Rust methods:

```rust
impl AppModel {
    fn click(&mut self) {
        self.clicks += 1;

        println!("Clicks: {}", self.clicks);
    }
}
```

The Model owns its state.

When something needs to modify that state, it receives mutable access:

```rust
fn click(&mut self)
```

This is one of Beverly's fundamental principles:

> **Your application state is ordinary Rust state.**

You can use structs, enums, methods, `Result`, `Option`, traits, iterators, and everything else Rust provides.

Beverly doesn't need to replace Rust's type system with another state-management abstraction.

---

# Your First Event

Now we need a way for the UI to communicate that something happened.

Define an Event:

```rust
event! {
    App::Clicked
}
```

Events are the vocabulary of your application.

The Event represents:

> "The user clicked."

An Event can be produced by many different sources:

- A button
- A keyboard shortcut
- An API
- Another application
- An AI agent

The producer doesn't need to know what happens next.

It only needs to emit the Event.

---

# Your First Interactive UI

Now we'll connect everything together.

Our application has four pieces:

```text
View
 ↓
Event
 ↓
Controller
 ↓
Model
```

The View contains the button.

The button emits:

```rust
Event::App::Clicked
```

The Controller connects that Event to the Model:

```rust
controller! {
    App::Clicked => Model::App::click,
}
```

The Model handles the action:

```rust
impl AppModel {
    fn click(&mut self) {
        self.clicks += 1;

        println!("Clicks: {}", self.clicks);
    }
}
```

The important distinction is:

**The View doesn't call the Model directly.**

Instead:

```text
Button
  ↓
Event::App::Clicked
  ↓
Controller
  ↓
Model::App::click
  ↓
State changes
```

This gives the application a clear separation between:

- What happened
- What should happen
- Who owns the state

---

# Putting It Together

A small Beverly application can therefore look conceptually like this:

```rust
struct AppModel {
    clicks: u32,
}

impl AppModel {
    fn click(&mut self) {
        self.clicks += 1;
        println!("Clicks: {}", self.clicks);
    }
}

event! {
    App::Clicked
}

controller! {
    App::Clicked => Model::App::click,
}
```

And the View:

```rust
card()
    .padding(16)
    .children([
        text("Hello, Beverly"),
        button("Click Me")
            .on("click", Event::App::Clicked),
    ])
```

The exact runtime/bootstrap syntax may vary by Beverly version, but the architecture remains the same:

```text
View → Event → Controller → Model
```

---

# Running the App

Run the application during development with:

```bash
cargo run
```

For faster iteration, use:

```bash
cargo check
```

when you only need to verify that the project compiles.

Build an optimized binary with:

```bash
cargo build --release
```

The resulting application will be placed in Cargo's standard release directory:

```text
target/release/
```

---

# Project Structure

As your application grows, keep the application organized around its responsibilities.

A simple project might look like:

```text
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

```text
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

```bash
cargo beverly build
```

This should behave like a normal build while providing Beverly-specific defaults and tooling.

You can still use:

```bash
cargo build
```

because Beverly is ultimately a Rust application.

---

# Building for Distribution

When you're ready to distribute your application:

```bash
cargo beverly publish
```

The publish command builds the application in release mode and produces the artifacts needed for distribution.

For example:

```bash
cargo beverly publish --target macos
```

The goal is to make the difference between development and distribution explicit:

```text
cargo beverly build
    ↓
Development build

cargo beverly publish
    ↓
Distribution build
```

Platform support can be extended over time:

```bash
cargo beverly publish --target macos
cargo beverly publish --target windows
cargo beverly publish --target linux
```

The application remains a normal Rust project throughout the process.

---

# What You Just Built

You've now built a complete Beverly application.

The architecture is:

```text
                    ┌─────────────┐
                    │    View     │
                    │             │
                    │   Button    │
                    └──────┬──────┘
                           │
                           ▼
                    ┌─────────────┐
                    │    Event    │
                    │             │
                    │ App::Clicked│
                    └──────┬──────┘
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
                    │ clicks += 1 │
                    └─────────────┘
```

This pattern scales.

A button click can become:

```text
Event → Model
```

Or a more complex application pipeline:

```text
Event
  ↓
Validate
  ↓
Load Data
  ↓
Process
  ↓
AI
  ↓
Save
  ↓
Emit Result Event
```

The same Event system can be used by humans, APIs, background processes, and AI agents.

That is the foundation of Beverly.

---

## Next

Now that you have a working application, the next chapter explains the architecture behind it:

**Model → View → Controller → Events**

You'll learn how Beverly represents application state, how Events become application contracts, how Controllers connect Events to behavior, and how the same architecture enables human and AI interaction.
