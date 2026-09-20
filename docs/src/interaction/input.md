# Input

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Input treatment covers both pointer and keyboard interaction semantics. Beverly should treat input as a shared interaction layer rather than a collection of random widget behaviors. This is the layer where user intent becomes real application action: text entry, selection, activation, keyboard traversal, and pointer-driven interaction all converge.

## What input means in a UI architecture

Input is not just an event system. It is the way the interface translates user actions into state changes. In a Bevy-style ECS model, the interaction layer should be organized around a few clear concepts:

- pointer state such as hover, press, drag, and release
- keyboard state such as focus and key events
- text entry and editing state
- selection and activation state
- structural context such as the active form, panel, or list

These concerns should remain separate enough to be reasoned about but connected enough to work as a coherent interface.

## Interaction principles

- pointer and keyboard input should have equivalent outcomes where possible
- focus should be stable and visible during editing
- input areas should provide immediate and legible feedback
- interaction rules should be consistent across text, selects, search boxes, and forms
- input state should not be hidden inside a widget without a clear system-wide pattern

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_input_surface(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyStack::vertical(),
    )).with_children(|stack| {
        stack.spawn(BeverlyInput::new("Query").placeholder("Search for files"));
        stack.spawn(BeverlyButton::primary("Run search"));
    });
}
```

This example shows the right way to think about input: the field has a clear purpose, the interaction model is consistent, and the user can move smoothly between editing and action.

## Pointer and keyboard parity

The best UI systems avoid a split between “mouse controls” and “keyboard controls.” If a control is important, it should work well with both inputs. For example:

- pressing Enter on a search field should trigger the search action
- a button should be keyboard-operable and pointer-operable
- text inputs should support focus, selection, and editing without requiring a mouse
- list and menu navigation should work with both pointer and arrow-key patterns

This improves the product for everyone, but it is particularly important for keyboard users and people working in constrained or assistive setups.

## Immediate feedback

When a user interacts with a form field, the interface should ideally respond at once. That feedback may include:

- focus ring or border emphasis
- value preview or inline validation
- hover or pressed visual state
- status messaging when a search or request is running

A good interaction layer tells the user “this input is active,” “this value changed,” or “this action has started.” Without that feedback, users can feel uncertain or lose trust in the system.

## Input consistency across widgets

Beverly should keep a shared interaction model across input surfaces so the user does not need to learn a new pattern for each component. Search, select, text input, radio, toggle, and file inputs should feel like part of the same interaction language.

That means consistent behavior around:

- focus order
- visible active states
- keyboard controls
- disabled styling
- clear labels and names
- error messaging

## Guidance

Input surfaces should feel consistent and reliable. The user should not have to learn different interaction rules for each field type, and the app should support both direct editing and accessible keyboard workflows. A good interaction layer improves confidence because it is predictable, legible, and grounded in clear structure.
