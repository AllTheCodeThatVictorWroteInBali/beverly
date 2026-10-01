# Beverly

### The UI for AI.

Beverly is a small, native UI runtime for AI-native applications — fast for humans, understandable by agents, and safe by default.

Built with Rust, Bevy, and GPU-native rendering, Beverly is for interfaces where conversations, tools, workflows, approvals, data, files, telemetry, and real-time state are part of the product — not an afterthought.

- **Website:** [beverlyui.com](https://beverlyui.com/)
- **Documentation:** [beverlyui.com/docs/](https://beverlyui.com/docs/)
- **Crate:** [crates.io/crates/beverly](https://crates.io/crates/beverly)

---

## Why Beverly?

AI can generate code faster than humans ever could. The bottleneck is the substrate.

AI-native applications need interfaces that can represent continuously changing state, structured actions, tool calls, permissions, streamed output, and complex operational data. Beverly provides a reliable, composable substrate for building those interfaces.

Rather than placing the UI behind a browser boundary, Beverly runs as part of the application:

```text
Rust
  ↓
Bevy ECS
  ↓
Beverly UI
  ↓
GPU
  ↓
Application
```

The result is a native interface layer that can live alongside models, files, databases, devices, sensors, GPUs, and private data.

---

## Design principles

### Small runtime

Beverly is designed to be small enough to ship: fast startup, low overhead, local execution, and no browser required. It is intended to live inside the product, from desktop software to embedded systems.

### Human + agent friendly

Predictable APIs, strongly typed primitives, explicit state, deterministic layouts, and semantic components give people and AI agents a shared vocabulary for building interfaces.

The same primitives should support direct human interaction, automation, and agent-driven workflows.

### Safe by default

AI can generate; Rust can verify.

Rust's ownership model, type system, exhaustive matching, and compile-time guarantees constrain broad classes of mistakes before software ships. Beverly pairs probabilistic generation with a predictable systems substrate.

### Local by default

Your AI's interface can run next to your AI.

Beverly prioritizes local execution and developer control: no mandatory cloud UI layer, no external CDN dependency, and no requirement that every interaction crosses a network boundary.

---

## The Best of the Web, Without the Web Runtime

Beverly brings the best ideas from the web into a native Rust UI runtime — without bringing along the browser.

You get familiar composition, semantic components, accessibility, screen reader support, ARIA, keyboard navigation, and a rich vocabulary for building interfaces.

But you don't have to deal with the parts of the web that make complex applications painful:

- No CSS quirks or specificity battles
- No JavaScript jank
- No browser latency
- No slow DOM updates
- No DOM reconciliation overhead
- No browser runtime
- No JavaScript dependency
- No CSS engine
- No CDN or network dependency

### Familiar Composition

Beverly uses a familiar chaining and composition model that makes the transition from web development feel natural.

If you've built interfaces with jQuery, HTML, and component-based web frameworks, the basic idea is immediately recognizable: start with a component, compose it with other components, configure it, bind state, and attach behavior.

```rust
MyButton()
    .disabled(model.saving())
    .on("click", Command::Document::save)
```

The syntax is familiar. The runtime is fundamentally different.

Instead of manipulating a DOM through JavaScript and asking a browser to continuously interpret HTML, CSS, and scripts, Beverly gives you **typed Rust components rendered directly through a native UI runtime**.

### Accessibility Is Part of the Runtime

Accessibility isn't something you bolt onto the application at the end.

Beverly provides built-in support for:

- Screen readers
- Complex ARIA trees
- Keyboard navigation
- Focus management
- Semantic UI primitives
- Accessible forms
- Dialogs, menus, tabs, trees, and other composite components

The result is a native UI that retains the accessibility conventions and interaction vocabulary developers already know from the web.

### A Faster Transition From the Old Web

Beverly is not asking developers to forget everything they learned building for the web.

**Keep the good parts. Lose the baggage.**

You get the web's mature interaction model and accessibility vocabulary with the performance, determinism, type safety, and distribution characteristics of a native Rust application.

**The web's best ideas. None of the browser's baggage.**

## Primitive layer

Beverly focuses on a small set of composable, strongly typed primitives with a large practical surface area. These primitives can form interfaces for conversations, dashboards, inspectors, workflows, tools, and real-time systems.

Current and planned interface capabilities include:

- Virtualized data views for large datasets
- Command surfaces for human and programmatic actions
- Agent tool cards for calls, results, states, approvals, and autonomous work
- Telemetry interfaces for live system state, events, and logs
- Data grids and trees for structured information
- Streaming states for loading, partial, live, stale, error, and updating data
- GPU-native surfaces, gradients, blur, glass, borders, and themes
- Interaction feedback for focus, hover, selection, validation, transitions, and motion
- Semantic controls shared by people, automation, and AI agents

---

## Status

Beverly is under active development.

The project currently provides a standalone Bevy component library with a `BeverlyPlugin`, built-in UI components, theming, animation, accessibility primitives, and GPU shader/material rendering. APIs may change as Beverly evolves around AI-native application workflows.

---

## Getting started

The executable API uses Bevy systems, messages, and Beverly's fluent
`App::new().ui(...)` composition. Some architecture prose below describes
future AI-oriented concepts; those sketches are intentionally separate from
the current compiling API.

### Installation

```toml
[dependencies]
bevy = "0.19"
beverly = "0.1"
```

Beverly targets Bevy `0.19`.

### Basic setup

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn main() {
    App::new().ui(my_ui()).run();
}

fn my_ui() -> Ui {
    ui()
        .theme(light_theme())
        .children([text("Hello, World!")])
}
```

`App::ui(...)` wires up Beverly's component systems, rendering/material pipeline, theming, animation, and accessibility primitives, and spawns the `Ui` tree as the application's root. It also creates a default 2D UI camera when the app has not created one.

### Prelude

```rust
use beverly::prelude::*;
```

The prelude re-exports `BeverlyPlugin`, theming, rendering and styling primitives, icons, and the primary types for Beverly's built-in components.

---

## Architecture

Beverly keeps UI state, rendering, interaction, animation, and application state close to Bevy's ECS execution model.

```text
┌─────────────────────────────────────┐
│           Application               │
├─────────────────────────────────────┤
│           Beverly UI                │
│                                     │
│  Components · Layout · Interaction  │
│  Animation · Accessibility          │
├─────────────────────────────────────┤
│              Bevy ECS               │
├─────────────────────────────────────┤
│              wgpu                   │
├─────────────────────────────────────┤
│               GPU                   │
└─────────────────────────────────────┘
```

Beverly's rendering system supports GPU-native gradients, borders, shadows, glass and backdrop-blur effects, noise, and skeleton shimmer. Its crate assets are embedded so consuming projects do not need to copy a Beverly `assets/` directory or configure a custom asset path.

---

## Development

```bash
git clone https://github.com/AllTheCodeThatVictorWroteInBali/beverly.git
cd beverly
cargo check
cargo test
cargo fmt --check
```

For documentation development:

```bash
mdbook serve docs
```

Then open `http://localhost:3000`.

---

## Publishing

Beverly ships a developer CLI, `cargo-beverly`, for packaging Beverly (Bevy) applications for
distribution:

```bash
cargo install --path tools/cargo-beverly
cargo beverly publish
```

`cargo beverly publish` builds your application in release mode using Cargo metadata (package
name, version, binary target), collects the release binary and your `assets/` directory, and
produces a runnable application bundle in `dist/`.

**macOS** is supported today, producing a standard `.app` bundle:

```
dist/
└── MyApp.app/
    └── Contents/
        ├── MacOS/MyApp
        ├── Resources/assets/...
        └── Info.plist
```

**Windows and Linux packaging are coming soon.** The publishing pipeline already separates
platform-independent steps (metadata, release build, asset collection) from a per-platform
packager, so adding `WindowsPackager`/`LinuxPackager` implementations won't require redesigning
`cargo beverly publish` itself.

See [tools/cargo-beverly/README.md](tools/cargo-beverly/README.md) for full details, flags, and
`[package.metadata.beverly]` overrides.

---

## Contributing

Beverly is open source and contributions are welcome. The project is early, so discussion and experimentation are encouraged.

Before submitting a pull request, run:

```bash
cargo fmt --check
cargo check
cargo test
```

For larger architectural changes, please open an issue first. See [`CONTRIBUTING.md`](CONTRIBUTING.md) for additional guidance.

---

## License

Licensed under the [Apache License 2.0](LICENSE).

---

## Links

- **Website:** [beverlyui.com](https://beverlyui.com/)
- **Documentation:** [beverlyui.com/docs/](https://beverlyui.com/docs/)
- **Issues:** [GitHub Issues](https://github.com/AllTheCodeThatVictorWroteInBali/beverly/issues)

<p align="center">
  Built with Rust, Bevy, and the belief that the UI belongs with the application.
</p>
