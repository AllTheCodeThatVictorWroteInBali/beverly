# Beverly

### The UI for AI.

Beverly is a small, native UI runtime for AI-native applications — fast for humans, understandable by agents, and safe by default.

Built with Rust, Bevy, and GPU-native rendering, Beverly is for interfaces where conversations, tools, workflows, approvals, data, files, telemetry, and real-time state are part of the product — not an afterthought.

**Website:** [beverlyui.com](https://beverlyui.com/)  
**Documentation:** [beverlyui.com/docs/](https://beverlyui.com/
docs/)
**Crate** [crates.io/crates/beverly](https://crates.io/crates/beverly)

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
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: light_theme(),
        })
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
```

`BeverlyPlugin` wires up Beverly's component systems, rendering/material pipeline, theming, animation, and accessibility primitives.

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
