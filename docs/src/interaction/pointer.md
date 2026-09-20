# Pointer

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Pointer handling aligns hover, press, and drag behaviors across widgets. It is the layer that turns pointer movement and clicks into meaningful interaction state for the UI. In a framework like Beverly, pointer behavior should remain predictable and consistent across widgets so users do not need to relearn the same fundamentals in each component.

## Core pointer behaviors

The base pointer model usually includes:

- hover state while the cursor is over an interactive surface
- press state while the pointer is held down on a control
- click or activation when the pointer is released on the same control
- drag and move tracking for reordering, resizing, or moving content
- pointer leave and cancel handling when actions are interrupted

These states should be handled consistently so that a button, card, menu item, or custom control feels like part of the same interaction system.

## Hover and press feedback

Hover and press states are not just decorative; they help users understand that the interface is responding to their input. A widget should show that it is interactive without becoming distracting.

Common examples include:

- subtle fill or border changes on hover
- slight elevation or emphasis during press
- visual feedback for active selection or drag targets
- immediate states that match active or disabled conditions

The goal is not to overdo the effect, but to provide enough feedback that the action is legible.

## Consistency across widgets

A pointer should behave similarly across the app. If a button highlights on hover, a card-action area or nav item should generally follow the same rules. That consistency is one of the main reasons the interaction layer exists as a shared system instead of scattered widget-specific logic.

This helps users build a mental model for the product. They understand that hover and press cues are a language, not a one-off feature added to a small group of components.

## Pointer cancellation and reliability

Pointer handling must also account for interruption. The user may move out of a widget, leave the app, or trigger a drag that cancels the original interaction. The system should respond gracefully by clearing hover or press state rather than leaving the UI in a stale active state.

This is a key part of interaction reliability. If a pressed state remains active after the pointer leaves, the interface can look broken or behave unpredictably.

## The role of the pointer layer

The pointer layer is the bridge between raw input and the app’s shared interaction state. It should convert low-level movement and click events into meaningful state transitions that other systems can consume.

That means pointer behavior and the rest of the interaction system should remain aligned. Layout, focus, selection, and rendering can all react to the same interaction state without fighting one another.

## Guidance

Pointer handling should align with the broader interaction design: it should be consistent, legible, and predictable. A user should feel like the UI is responding clearly to input, not like they are fighting against hidden widget logic.
