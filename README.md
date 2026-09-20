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

Beverly is **early-stage and actively evolving**.

The project is currently focused on establishing the core UI architecture, primitives, rendering system, accessibility foundations, and developer experience.

APIs may change as the system matures.

For the current implementation status, see the [documentation](https://beverlyui.com/docs/).

---

## Getting Started

Beverly is currently under active development.

Once the initial API stabilizes, the intended experience will look approximately like:

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

See the [Getting Started documentation](https://beverlyui.com/docs/) for the current setup and API.

---

## Components

Beverly is being developed as a reusable system of UI primitives rather than a collection of application-specific widgets.

The component system is expected to include primitives such as:

- Button
- Card
- Input
- Checkbox
- Toggle
- Select
- Dropdown
- Modal
- Tabs
- Navigation
- Badge
- Alert
- Toast
- Tooltip
- Avatar
- Divider
- Pagination
- Table
- Search
- File Input
- Radio
- Button Group

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

## Roadmap

### Foundation

- [x] Public project repository
- [x] Project website
- [x] Documentation site
- [x] GitHub Pages deployment
- [ ] Design tokens
- [ ] Theme system
- [ ] Core layout primitives
- [ ] Animation system

### Components

- [ ] Button
- [ ] Card
- [ ] Input
- [ ] Checkbox
- [ ] Toggle
- [ ] Select
- [ ] Dropdown
- [ ] Modal
- [ ] Tabs
- [ ] Navigation
- [ ] Tooltip
- [ ] Toast
- [ ] Data table

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
