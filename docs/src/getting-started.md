# 2. Quick Start

Beverly is designed to get you from an empty Rust project to a complete interactive application quickly.

This tutorial builds a tiny application with:

- A reusable Button component
- A Rust Model that owns application data
- A typed Bevy Message
- A Controller plugin connecting the Message to application behavior
- A View that reflects the Model
- Built-in accessibility
- A structure that is straightforward to test

The complete flow is:

```text
View
  ↓
Message
  ↓
Controller
  ↓
Model
  ↓
Application Data
  ↓
View
```

## 2.1 Installation

Create a new Rust project:

```bash
cargo new hello_beverly
cd hello_beverly
```

Add Bevy and Beverly:

```bash
cargo add bevy
cargo add beverly
```

Beverly is a Rust crate, so you work with it the same way you work with other Rust dependencies.

---

# 2.2 Hello, Beverly

Start with a simple application:

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn main() {
    app()
        .window_size(960, 540)
        .title("Hello, Beverly")
        .theme(light_theme())
        .children([text("Hello, Beverly!")])
        .run();
}
```

That's a Beverly application.

There is no browser, JavaScript runtime, HTML document, or separate frontend server.

Beverly runs as part of your native Rust application.

`app()` owns the global application configuration. Configure the window, theme,
font asset, and root children before calling `.run()`:

```rust
app()
    .window_size(1280, 720)
    .title("Workspace")
    .font("fonts/Inter-Regular.ttf")
    .theme(dark_theme())
    .children([text("Workspace")])
    .run();
```

---

# 2.3 Your First Component

Beverly provides reusable UI primitives.

Create a button:

```rust
button().text("Click Me")
```

The same Button component can be reused throughout your application:

```rust
button().text("Save")
button().text("Cancel")
button().text("Delete")
button().text("Create User")
```

Build a larger interface by composing components:

```rust
app()
  .children([button().text("Get Started")])
  .run();
```

For button content, use `.children([ButtonChild::icon(...), ButtonChild::text(...)])`.
Other Beverly components use the shared `app().children([...])` composition API.

Build a primitive once. Compose and reuse it everywhere.

---

# 2.4 Accessibility Is Part of the Component

Accessibility is not an afterthought in Beverly.

It is part of the component contract.

For example, a button gets its accessible name from its text content:

```rust
button().text("Click Me")
```

Beverly treats accessibility as a first-class runtime concern rather than something added during a final QA pass. The button plugin projects the text content into its semantic node; custom content should include a text child so assistive technology has a meaningful name.

The goal is simple:

> **Accessible by construction, not accessible by cleanup.**

This applies to Beverly's complex UI primitives as well — forms, navigation, dialogs, trees, data interfaces, keyboard interaction, and other components where accessibility cannot be reduced to a single visual label.

Accessibility should be something the framework helps you get right automatically, while still making the important semantics explicit.

---

# 2.5 Your First Model

The Model is ordinary Rust.

It owns the application's underlying data.

You don't need a separate state-management system just to keep your UI synchronized.

For our application:

```rust
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
    }
}
```

The important distinction is that the Model owns the data and exposes the operations that make sense for that data.

Getters read it.

Setters and Model methods change it.

The Model can also enforce application rules:

```rust
fn set_clicks(&mut self, clicks: u32) {
    self.clicks = clicks.min(100);
}
```

Now the invariant lives with the data rather than being scattered throughout the UI.

---

# 2.6 Your First Message

Messages define the vocabulary of your application.

Declare one:

```rust
event! {
    App::Clicked
}
```

The macro generates a Bevy `Message` type. Register it explicitly:

```rust
app.add_message::<App::Clicked>();
```

---

# 2.7 Connect the Button

Attach an explicit callback to the Button:

```rust
fn click_button(commands: &mut Commands, _button: Entity) {
  commands.write_message(App::Clicked);
}

button()
  .text("Click Me")
  .on("click", click_button)
```

The Button does not emit application messages automatically. The callback is
the explicit attachment point; other inputs may write the same message through
their own application systems.

---

# 2.8 Your First Controller

The Controller connects Messages to behavior:

```rust
controller! {
  AppController {
    App::Clicked => handle_click,
  }
}

fn handle_click(_message: &App::Clicked) {
  // Call the model's domain method here.
}
```

The Controller does not define the Message.

The Message was already defined by `event!`.

The Controller simply says:

> When this Message occurs, run this handler.

This keeps the application's vocabulary separate from its wiring.

The basic relationship is:

```text
Message = Contract
Controller = Connection
Model = Owner
View = Projection
```

---

# 2.9 The View Reflects the Model

The View reads the current state of the Model:

```rust
text(format!("Clicks: {}", model.clicks()))
```

The View does not own the application's underlying data.

It projects the current Model state into the UI.

The relationship is straightforward:

```text
Model changes
      ↓
View reads current state
      ↓
Bevy updates/rendering
      ↓
UI reflects the change
```

Beverly runs as part of Bevy's rendering/update loop. At 60 FPS, a new frame is available roughly every 16.7 milliseconds.

That means a Model change can be reflected essentially immediately in the UI.

> **Change the Model. The UI reflects the Model.**

---

# 2.10 Why Beverly Is Easy to Test

Beverly's architecture makes testing straightforward because each major part has a clear responsibility.

### Test the Model as Rust

The Model contains your application logic and data.

You can test it without starting a window, rendering a frame, or interacting with the UI:

```rust
#[test]
fn increments_clicks() {
    let mut model = AppModel { clicks: 0 };

    model.increment_clicks();

    assert_eq!(model.clicks(), 1);
}
```

This is just a Rust test.

### Test the View as Bevy

The View is built from Bevy components.

That means View behavior can be tested using the same Bevy-oriented testing techniques used elsewhere in the application.

You don't need a second UI architecture or a browser automation layer simply to exercise your component tree.

### Test Events as contracts

Events are typed.

You can test that an interaction produces the expected Event and that the Event contains the expected payload.

### Test Controllers as wiring

Controllers are explicit mappings:

```rust
App::Clicked => Model::App::increment_clicks
```

That makes the application's behavior easy to reason about and easy to test.

The result is a useful property:

> **The application is testable because its architecture is explicit.**

There isn't one enormous UI layer that has to be tested as a black box.

The Model, View, Events, and Controllers each have a clear boundary.

---

# 2.11 Project Structure

A small application can start with a few files.

As it grows, Beverly's architecture maps naturally onto your Rust project:

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

A larger application might look like:

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

This isn't a requirement.

It is simply a natural way to organize a Beverly application.

---

# 2.12 Running Your Application

Run your application normally:

```bash
cargo run
```

Check it without running:

```bash
cargo check
```

Build an optimized release:

```bash
cargo build --release
```

---

# 2.13 Beverly Commands

Beverly can also provide application-oriented Cargo commands.

For example:

```bash
cargo beverly build
```

Build a development application.

For distribution:

```bash
cargo beverly publish
```

The goal of `publish` is to feel like a natural extension of `cargo build`, while producing the artifacts you actually distribute.

Target-specific publishing can then be extended modularly:

```bash
cargo beverly publish --target macos
cargo beverly publish --target windows
cargo beverly publish --target linux
```

The initial implementation can target macOS while leaving the command structure open for additional platforms.

---

# 2.14 What You Just Built

You started with an empty Rust project and ended with the basic architecture of a Beverly application:

```text
             ┌───────────┐
             │    View   │
             └─────┬─────┘
                   │
                 Event
                   │
             ┌─────▼─────┐
             │ Controller│
             └─────┬─────┘
                   │
                 Action
                   │
             ┌─────▼─────┐
             │   Model   │
             └─────┬─────┘
                   │
             Application
                Data
                   │
                   ▼
                  View
```

The Button is reusable.

The Model is ordinary Rust.

The View is built from Bevy.

The Event defines the application's vocabulary.

The Controller connects Events to behavior.

Accessibility is part of the component contract.

And because each part has a clear responsibility, each part can be tested independently.

This is the foundation Beverly builds on:

> **The Event is the contract. The Controller is the connection. The Model is the owner. The View is the projection.**
