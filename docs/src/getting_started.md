# Getting Started

Welcome to Beverly.

Beverly is a Rust-native UI system for [Bevy](https://bevyengine.org/), designed for high-performance, accessible, AI-native, and local-first applications.

This guide will get you from a new Rust project to your first Beverly application.

> **Development status**
>
> Beverly is currently in early development. APIs and architecture may change as the project evolves. The examples in this documentation reflect the current development API and may change between releases.

---

## Prerequisites

Before working with Beverly, you should have:

- Rust and Cargo
- A working Bevy development environment
- Basic familiarity with Rust
- Basic familiarity with Bevy

Install Rust using [rustup](https://rustup.rs/) if you don't already have it.

Verify your installation:

```bash
rustc --version
cargo --version
```

You should see a Rust compiler and Cargo version printed in your terminal.

---

## Create a Bevy Project

Create a new Rust application:

```bash
cargo new my-beverly-app
cd my-beverly-app
```

Run the default application:

```bash
cargo run
```

Once your basic Bevy environment is working, you're ready to add Beverly.

---

## Add Beverly

Add Beverly to your project's dependencies.

During early development, Beverly may be consumed directly from the Git repository:

```toml
[dependencies]
bevy = "..."
beverly = { git = "https://github.com/AllTheCodeThatVictorWroteInBali/beverly.git" }
```

Once Beverly has stable crates published to crates.io, this documentation will use the released crate version instead.

> **Note**
>
> Because Beverly is under active development, the exact dependency configuration may change. Check the repository's `Cargo.toml` and the current API documentation if the example above does not match the latest version.

---

## Initialize Beverly

A Beverly application is built on top of Bevy.

The intended application structure is:

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .run();
}
```

The `BeverlyPlugin` provides the core Beverly systems needed by the application.

As Beverly's architecture develops, additional plugins and configuration options may become available.

---

## Your First UI

Once Beverly is initialized, UI components can be added to your Bevy application.

The basic model is:

```text
Bevy Application
       │
       ▼
Beverly Plugin
       │
       ├── UI Components
       ├── Layout
       ├── Interaction
       ├── Animation
       └── Rendering
```

A simple application might eventually look like:

```rust
fn setup(mut commands: Commands) {
    commands.spawn(
        BeverlyButton::new("Hello, Beverly")
    );
}
```

The exact component API is still evolving, so refer to the component documentation for the current implementation.

---

## Run Your Application

Start the application with:

```bash
cargo run
```

For development builds, Cargo will compile the application and launch the Bevy window.

As you develop your UI, you can use Bevy's normal application and ECS workflow alongside Beverly's UI primitives.

---

## Project Structure

A simple Beverly application can be organized like this:

```text
my-beverly-app/
├── src/
│   └── main.rs
├── assets/
├── Cargo.toml
└── Cargo.lock
```

As your application grows, you may want to separate UI components from application logic:

```text
my-beverly-app/
├── src/
│   ├── main.rs
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── navigation.rs
│   │   └── components.rs
│   └── systems/
│       └── mod.rs
├── assets/
├── Cargo.toml
└── Cargo.lock
```

The important distinction is that Beverly UI remains part of the same Bevy application rather than requiring a separate browser-based frontend.

---

## Development Workflow

Beverly follows the normal Rust development workflow.

### Check the project

```bash
cargo check
```

### Run the application

```bash
cargo run
```

### Run tests

```bash
cargo test
```

### Format the code

```bash
cargo fmt
```

### Run Clippy

```bash
cargo clippy
```

For contributions to Beverly itself, see [Contributing](../CONTRIBUTING.md) in the repository.

---

## Working With Components

Beverly components are designed to be composable.

Instead of building an entire application as a collection of specialized widgets, applications should be constructed from reusable primitives.

For example:

```text
Application
│
├── Navigation
│   ├── Button
│   └── Badge
│
├── Content
│   ├── Card
│   ├── Input
│   └── Table
│
└── Actions
    ├── Button
    └── Dropdown
```

This makes the component system reusable across different applications while keeping application-specific behavior in the application layer.

See the [Components](./components/overview.md) documentation for the current component catalog.

---

## Themes

Beverly is designed around a reusable theme system.

Themes will provide shared definitions for things such as:

- Colors
- Typography
- Spacing
- Borders
- Radii
- Shadows
- Motion
- Component states

The goal is to allow an application to establish a visual language once and have that language propagate throughout its components.

Theme APIs are currently under development.

---

## Accessibility

Accessibility should be considered from the beginning of an application rather than added after the UI is complete.

When building Beverly interfaces, consider:

- Keyboard navigation
- Focus management
- Visible focus states
- Clear interaction states
- Sufficient contrast
- Reduced motion
- Semantic relationships
- Screen-reader compatibility where applicable

Beverly's accessibility system is actively evolving.

See [Accessibility](./principles/accessibility.md) for the project's accessibility goals and implementation guidance.

---

## AI-Native Interfaces

Beverly is designed with AI-native applications in mind.

An AI-enabled application may need UI primitives for things such as:

```text
User
 │
 ▼
Command
 │
 ▼
Agent
 │
 ├── Tool call
 ├── Streaming response
 ├── Confirmation
 └── Result
       │
       ▼
      UI
```

Rather than creating an entirely separate interface for AI activity, Beverly aims to make these interactions composable with the same UI system used by human users.

AI interaction primitives are currently experimental.

See [AI Interaction](./principles/ai-interaction.md) for the broader design direction.

---

## Local-First Applications

Beverly does not require a web server or external service simply to render an interface.

This makes it suitable for applications that need to operate:

- Offline
- Locally
- On private networks
- In self-hosted environments
- In restricted or air-gapped environments

Network services and AI providers can still be integrated when an application requires them. Beverly's goal is to avoid making those services a requirement of the UI layer itself.

See [Sovereignty](./principles/sovereignty.md) for more on this design principle.

---

## What's Next?

Once your application is running, the best place to continue is the component documentation.

Recommended path:

1. [Components](./components/overview.md)
2. [Architecture](./architecture.md)
3. [Themes](./foundations/themes.md)
4. [Accessibility](./principles/accessibility.md)
5. [Performance](./principles/performance.md)
6. [AI Interaction](./principles/ai-interaction.md)
7. [Sovereignty](./principles/sovereignty.md)

---

## Building Beverly Itself

If you're interested in contributing to Beverly rather than simply using it, clone the repository:

```bash
git clone https://github.com/AllTheCodeThatVictorWroteInBali/beverly.git
cd beverly
```

Then run:

```bash
cargo check
cargo test
```

For documentation development:

```bash
mdbook serve docs
```

The documentation site will be available at:

```text
http://localhost:3000
```

See [CONTRIBUTING.md](https://github.com/AllTheCodeThatVictorWroteInBali/beverly/blob/main/CONTRIBUTING.md) for contribution guidelines.

---

## Where Beverly Is Going

Beverly is being built around four principles:

**Performance** — UI should scale with the application rather than becoming a bottleneck.

**Accessibility** — Accessible interaction should be part of the primitive itself.

**AI Interaction** — Humans and AI agents should be able to operate the same underlying application.

**Sovereignty** — Applications should remain usable and controllable without mandatory external infrastructure.

These principles guide the architecture as the project develops.

**Welcome to Beverly.**
