# Screen Readers

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Screen-reader support depends on semantics, not decoration. A polished interface is not accessible if a screen reader cannot understand the structure, names, or state of the controls.

## Core requirements

The application should provide:

- clear text labels for controls and actions
- a meaningful reading order for panels and cards
- obvious state cues for selected, disabled, expanded, or alert surfaces
- consistent names for repeated controls, especially navigation items and form fields

## Semantic structure

A Beverly interface should favor meaningful hierarchy over visual-only grouping. The app shell, cards, nav items, forms, and dialogs should all expose clear structure so screen-reader users can navigate with confidence.

When a control has a dynamic state, that state should be announced or discoverable. For example:

- selected or active tabs
- expanded or collapsed panels
- disabled controls
- validation states and error summaries
- busy or loading content

## Naming and labeling

Labels should be concise, specific, and consistent. Generic names like "Button" or "Open" are not enough when several actions appear in the same interface. A user should be able to understand the action without seeing the visual layout.

This matters for:

- action buttons in toolbars
- form inputs and filters
- modal close or confirmation controls
- status surfaces and notifications

## Example: labeled form controls with state

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_form(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyStack::vertical(),
    )).with_children(|parent| {
        parent.spawn(BeverlyInput::new("Email address")
            .placeholder("name@example.com")
            .required(true));

        parent.spawn(BeverlyButton::secondary("Send invite"));
    });
}
```

The text label gives the field a clear accessible name, and the button label provides a precise meaning for assistive technologies.

## Accessibility first, polish second

Glass surfaces, gradients, and motion effects can all be beautiful, but they should never obscure meaning. The semantic layer is what makes the interface understandable to every user, regardless of input method or assistive technology.
