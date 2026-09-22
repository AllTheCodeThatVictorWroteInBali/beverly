# Beverly

### A Rust UI system for Bevy.

Beverly is a Rust-native UI system for building high-performance, accessible interfaces with [Bevy](https://bevyengine.org/).

It is designed for applications where traditional browser UI starts to become a constraint: data-intensive interfaces, real-time applications, AI-native software, local-first tools, and systems that need to run without depending on a web stack.

**Website:** [beverlyui.com](https://beverlyui.com/)
**Documentation:** [beverlyui.com/docs/](https://beverlyui.com/docs/)

---

## Why Beverly?

Modern application interfaces increasingly need to handle:

- Large, dynamic datasets
- High-frequency state updates
- Real-time streaming
- AI-generated content and actions
- Complex interaction states
- Offline and local-first operation
- GPU-accelerated rendering
- Strict control over application data

Beverly approaches UI from a different foundation:

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

Rather than treating UI as a layer on top of a browser, Beverly treats UI as part of the application itself.

---

## Design Principles

### Performance

Beverly is built on Bevy and Rust, providing a foundation for interfaces that need to remain responsive under high-frequency updates and large amounts of data.

The goal is not simply to make traditional UI faster. It is to make UI a first-class part of a high-performance application architecture.

### Accessibility

Accessibility is a foundational requirement rather than a final polish step.

Beverly aims to provide accessible interaction primitives including:

- Keyboard navigation
- Focus management
- Clear interaction states
- Reduced-motion support
- High-contrast interfaces
- Predictable component behavior
- Screen-reader integration where applicable

The long-term target is WCAG 2.2 AA for supported interface primitives.

### AI Interaction

AI changes the way software is operated.

Beverly is designed around the idea that the same interface primitives should be usable by both humans and software agents.

This includes primitives for:

- Command interfaces
- Structured actions
- Tool execution
- Streaming responses
- Agent status
- Confirmations
- Permissions
- Activity and telemetry

The goal is not an "AI UI layer" bolted onto an existing application, but UI primitives that understand AI interaction as a first-class use case.

### Sovereignty

Beverly is designed for software that can remain under the developer's control.

The project prioritizes:

- Local-first operation
- Offline capability
- Self-hosted deployments
- No mandatory external services
- No mandatory telemetry
- No external CDN dependency
- Air-gapped environments
- Explicit control over AI execution

Your application should not need permission from a third-party service to render its interface.

---

## Status

Beverly ships a working, standalone Bevy component library: a `BeverlyPlugin`, ~35 UI components, a GPU shader/material rendering system, animation, theming, and accessibility primitives, all installable as a normal crate dependency.

APIs may still change as the system matures, but the crate compiles, its examples run, and its test suite passes independently of any other project.

---

## Getting Started

### Installation

```toml
[dependencies]
bevy = "0.19"
beverly = "0.1"
```

Beverly targets the same Bevy version as the version listed above (currently Bevy `0.19`).

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

`BeverlyPlugin` wires up every component system, the rendering/material pipeline, theming, animation, and accessibility primitives in one call. Individual components can also be used without it as long as their own plugin (or the systems they depend on) is added manually; see each component's documentation for its specific requirements.

### Importing the prelude

```rust
use beverly::prelude::*;
```

The prelude re-exports `BeverlyPlugin`, the theme system, rendering/styling primitives (`Paint`, `Surface`, `Border`, gradients, glass/shadow effects), icons, and the primary type for every built-in component (`Alert`, `Button`, `Card`, `Modal`, `Table`, ...).

---

## Components

Beverly ships the following components, each with its own spawn helper and (where it has runtime behavior) its own `Plugin`:

Alert · Avatar · Badge · Button · Button Group · Card · Checkbox · Container · Divider · Dropdown ·
File Input · Footer · Form · Input · Link · List Item · Modal · Nav Button · Navbar · Pagination ·
Photo · Progress Bar · Radio · Search · Select · Sidebar · Slider · Spinner · Table · Tabs · Text/Title ·
Textarea · Toast · Toggle · Tooltip

Plus cross-cutting primitives: accessibility (`primitives::a11y`), focus management, keyboard
navigation, pointer/gesture interaction, clipboard, and semantic tree helpers; and animation:
transitions, motion, loading/skeleton placeholders, and backdrop blur.

More complex application interfaces can be composed from these primitives.

---

## Architecture

Beverly is built around Bevy's ECS architecture and Rust's type system.

The project aims to keep UI state, rendering, interaction, animation, and application state close to the same underlying execution model.

A simplified architecture:

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

This architecture is particularly useful for applications where UI state and application state are tightly coupled.

---

## Theming

Every component reads its colors, gradients, and shadows from a `ThemeResource`:

```rust
use beverly::prelude::*;

app.insert_resource(ThemeResource {
    current: light_theme(), // or dark_theme()
});
```

Themes are plain data (`ThemeColors`, `ThemeTransitions`, ...) so custom palettes can be built by
constructing a `Theme` value directly. Components react to `ThemeChanged` to re-paint when the
active theme is swapped at runtime.

---

## Shaders and assets

Beverly's rendering system (`beverly::rendering`) implements gradients, borders, inner/outer
shadows, glass/backdrop-blur effects, noise, and skeleton shimmer as a single GPU shader
(`UiShapeMaterial`) plus a couple of small companion shaders. All of it is embedded into the
compiled crate via Bevy's `embedded_asset!` mechanism, and Feather icon SVGs are embedded the
same way. This means:

- No `assets/` folder needs to be copied into a consuming project.
- No custom `AssetPlugin` path configuration is required for Beverly's own assets.
- The crate works identically whether used via a local `path` dependency or from a published
  version on crates.io.

---

## Feature flags

| Feature | Default | Enables |
| --- | --- | --- |
| `file_dialog` | on | Native file picker support for `FileInput` (via `rfd`). |
| `http_form` | off | HTTP form submission for `Form` (via `reqwest`). |
| `test-support` | off | Exposes headless UI-transform test fixtures for use in downstream crates' own tests. |

---

## Roadmap

### Foundation

- [x] Public project repository
- [x] Project website
- [x] Documentation site
- [x] GitHub Pages deployment
- [x] Design tokens
- [x] Theme system
- [x] Core layout primitives
- [x] Animation system

### Components

- [x] Button
- [x] Card
- [x] Input
- [x] Checkbox
- [x] Toggle
- [x] Select
- [x] Dropdown
- [x] Modal
- [x] Tabs
- [x] Navigation
- [x] Tooltip
- [x] Toast
- [x] Data table

### Platform

- [ ] Virtualized lists
- [ ] Advanced scrolling
- [ ] Accessibility primitives
- [ ] GPU-native visual effects
- [ ] Reduced-motion system
- [ ] High-contrast system
- [ ] AI interaction primitives
- [ ] Command system
- [ ] Streaming UI primitives

### Applications

- [ ] Component gallery
- [ ] Example application
- [ ] Data-intensive reference application
- [ ] AI-native reference application

The roadmap will evolve as the architecture becomes clearer.

---

## Contributing

Beverly is open source and contributions are welcome.

The project is still early, so discussion and experimentation are encouraged.

Before submitting a pull request, please make sure that:

```bash
cargo fmt --check
cargo check
cargo test
```

pass locally.

For larger architectural changes, opening an issue first is encouraged so the approach can be discussed before implementation.

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for additional guidelines.

---

## Development

Clone the repository:

```bash
git clone https://github.com/AllTheCodeThatVictorWroteInBali/beverly.git
cd beverly
```

Build the project:

```bash
cargo check
```

Run tests:

```bash
cargo test
```

Format the code:

```bash
cargo fmt
```

---

## Documentation

The documentation is built with [mdBook](https://rust-lang.github.io/mdBook/) and published automatically through GitHub Actions.

**Read the documentation:**
https://beverlyui.com/docs/

Local development:

```bash
mdbook serve docs
```

Then open:

```text
http://localhost:3000
```

---

## Philosophy

Beverly is built around a simple idea:

> **UI should be part of the application, not a separate application running beside it.**

Rust provides memory safety and predictable performance.

Bevy provides an ECS-based application architecture and GPU-oriented rendering foundation.

Beverly provides the interface layer that connects those capabilities to modern application UX.

The result is intended to be a foundation for software that is fast, accessible, intelligent, local, and under the developer's control.

---

## License

Beverly is currently under active development.

The project's license will be documented here before the first stable release.

---

## Links

- **Website:** https://beverlyui.com/
- **Documentation:** https://beverlyui.com/docs/
- **Repository:** https://github.com/AllTheCodeThatVictorWroteInBali/beverly
- **Issues:** https://github.com/AllTheCodeThatVictorWroteInBali/beverly/issues

---

<p align="center">
  Built with Rust and Bevy.
</p>
