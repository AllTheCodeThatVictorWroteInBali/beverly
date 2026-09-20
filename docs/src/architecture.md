# Architecture

Beverly is structured as a layered, ECS-first UI framework for Bevy. The goal is to keep the application layer focused on workflows and business logic while the UI stack provides reusable composition, semantics, theming, and rendering primitives.

## Core layers

1. Application layer
   - domain state, user workflows, and scene composition
2. UI component layer
   - buttons, cards, inputs, tables, navigation, modals, and alert patterns
3. Layout and data layer
   - stacks, containers, grids, lists, search, pagination, and dense content surfaces
4. Rendering and effects layer
   - fills, borders, gradients, shadows, blur, and liquid-glass treatments
5. Accessibility and performance layer
   - focus management, semantics, data-aware layout, motion safety, and profiling hooks

## Composition model

The design goal is simple: keep reusable UI primitives stateful and composable rather than forcing each screen to assemble bespoke widget code. In a Bevy application, the system is naturally expressed through ECS entities with shared component traits, layout constraints, and material-level rendering behavior.

This is the same general principle seen in more advanced UI toolchains: a small shared surface model can power many higher-level widgets while preserving a coherent design language.

## Theme and policy boundaries

Themes define the visual language of the application: colors, spacing, type, radius, motion, and how those values are applied. Policies define behavior under different contexts, such as reduced motion, reduced transparency, or high-contrast accessibility modes.

This split keeps the UI system adaptable without scattering one-off styles throughout the codebase. A widget should inherit semantics and tokens from the surrounding theme, while a policy layer decides whether those effects are too heavy or too aggressive for the current environment.

## Why this matters

This structure helps large app surfaces stay coherent over time. A dashboard, a media player, and an AI assistant all operate through the same underlying principles: consistent layout, accessible interaction, and a clear visual hierarchy.

For a practical starting point, see [Getting Started](./getting-started.md). For concrete product patterns, see [Examples Overview](./examples/overview.md).
