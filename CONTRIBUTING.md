# Contributing to Beverly

Thanks for your interest in contributing to Beverly.

Beverly is an early-stage Rust UI system for Bevy. The project is still establishing its architecture, APIs, and design principles, so contributions, experiments, feedback, and thoughtful criticism are all welcome.

The goal is to build a UI system that is fast, accessible, composable, and well suited to modern data-intensive and AI-native applications.

---

## Before You Start

For larger changes, especially changes to the architecture or public API, please open an issue or discussion before starting implementation.

This helps avoid duplicated work and gives us a chance to agree on the direction before the API becomes more difficult to change.

For small fixes, documentation improvements, tests, and straightforward components, feel free to open a pull request directly.

---

## Development Setup

Beverly is built with Rust and Bevy.

Clone the repository:

```bash
git clone https://github.com/AllTheCodeThatVictorWroteInBali/beverly.git
cd beverly
```

Make sure Rust is installed and up to date:

```bash
rustup update
```

Check that the project builds:

```bash
cargo check
```

---

## Before Opening a Pull Request

Please run the following locally:

```bash
cargo fmt --check
cargo check
cargo test
```

If applicable, also run:

```bash
cargo clippy -- -D warnings
```

Please make sure your changes do not introduce compiler warnings or formatting failures.

---

## Adding a Component

Beverly components should be designed as reusable primitives rather than application-specific implementations.

When adding a component, consider:

### API

- Is the component easy to construct?
- Does the API fit existing Beverly conventions?
- Can it be composed with other components?
- Are sensible defaults provided?

### States

Consider the states relevant to the component:

- Default
- Hover
- Focus
- Pressed
- Disabled
- Loading
- Error
- Selected
- Active

Not every component needs every state.

### Accessibility

Accessibility should be considered during implementation rather than added afterward.

Where applicable, consider:

- Keyboard interaction
- Focus behavior
- Focus visibility
- Semantic roles
- Clear state communication
- Reduced motion
- High contrast
- Screen-reader compatibility

### Performance

Avoid unnecessary per-frame work, allocations, or state changes.

Components should fit naturally into Bevy's ECS and rendering architecture.

If a component is expected to handle large collections of data, consider virtualization and incremental updates rather than rendering everything at once.

### Documentation

New public components should include documentation explaining:

- What the component does
- Basic usage
- Available variants
- Important states
- Accessibility considerations
- Relevant configuration or API

---

## Design Principles

Contributions should generally align with Beverly's four core principles.

### Performance

Beverly should take advantage of Rust and Bevy rather than recreating browser-oriented architecture.

Prefer efficient data flow, predictable updates, GPU-friendly rendering, and scalable primitives.

### Accessibility

Accessibility is a first-class design concern.

A component that looks correct but cannot be operated predictably with a keyboard is not finished.

### AI Interaction

Beverly is designed for interfaces where humans and software agents may interact with the same application.

When appropriate, consider how components could support structured actions, streaming state, confirmations, permissions, and machine-readable interaction.

### Sovereignty

Beverly prioritizes local-first and self-hosted software.

Avoid introducing mandatory external services, telemetry, CDNs, or network dependencies into core functionality.

---

## Code Style

Follow standard Rust conventions.

Use:

```bash
cargo fmt
```

for formatting.

Prefer clear, idiomatic Rust over clever abstractions.

Keep APIs small and composable.

Avoid introducing abstractions until they solve a demonstrated problem.

---

## Commit Messages

There is no strict commit-message format yet.

Please keep commits reasonably focused and describe what changed.

Good:

```text
Add keyboard navigation to dropdown
```

Less useful:

```text
Fix stuff
```

For larger changes, breaking work into logical commits is encouraged.

---

## Pull Requests

A good pull request should explain:

1. What changed
2. Why it changed
3. How it was implemented
4. How it was tested
5. Any API or architectural implications

For UI changes, screenshots or short recordings are encouraged when they make the change easier to understand.

A typical pull request might look like:

```text
## What changed

Added keyboard navigation to Dropdown.

## Why

Dropdown previously required pointer interaction.

## Testing

- cargo fmt --check
- cargo check
- cargo test

## Accessibility

Added keyboard focus and Escape-to-close behavior.
```

---

## Issues

Issues are useful for:

- Bug reports
- Feature proposals
- API discussions
- Accessibility problems
- Performance problems
- Documentation improvements

When reporting a bug, please include enough information to reproduce it.

Where possible, include:

- Rust version
- Bevy version
- Beverly version or commit
- Operating system
- Reproduction steps
- Expected behavior
- Actual behavior
- Relevant logs or screenshots

---

## Feature Requests

Feature requests are welcome.

Before proposing a new component or abstraction, consider whether the functionality can be composed from existing Beverly primitives.

For larger features, explain the problem being solved rather than only proposing a particular implementation.

For example:

> "Applications need a way to display thousands of rapidly changing records without rendering every row."

is more useful than:

> "Add a `VirtualizedTable` component."

The problem allows us to evaluate different implementations while keeping the API flexible.

---

## Documentation

Beverly's documentation is built with [mdBook](https://rust-lang.github.io/mdBook/).

Documentation lives in:

```text
docs/
└── src/
```

To run the documentation locally:

```bash
mdbook serve docs
```

Then open:

```text
http://localhost:3000
```

Documentation improvements are always welcome.

---

## Project Structure

The repository currently follows this general structure:

```text
beverly/
├── .github/
│   └── workflows/
├── assets/
├── docs/
│   ├── book.toml
│   └── src/
├── src/
├── index.html
├── Cargo.toml
├── README.md
├── CONTRIBUTING.md
└── LICENSE
```

The structure will evolve as the project grows.

---

## Community

Please keep discussions constructive and respectful.

Beverly is an open-source project, and contributors may have different ideas about implementation, architecture, and design.

Good-faith disagreement is encouraged.

Harassment, discrimination, personal attacks, and deliberately disruptive behavior are not.

---

## A Note on the Project's Stage

Beverly is currently experimental.

Public APIs, architecture, rendering approaches, and component designs may change substantially before a stable release.

If you are building something important on top of Beverly, keep this in mind and consider pinning to a known commit or version.

Early contributors have an opportunity to help shape the architecture before those decisions become permanent.

---

## Thank You

Whether you're submitting a bug report, improving documentation, building a component, finding an accessibility issue, profiling performance, or simply trying Beverly and giving feedback:

**thank you for contributing.**

The goal is to build a UI system that makes Rust and Bevy a compelling foundation for modern application interfaces.

**Beverly — UI for Bevy.**
