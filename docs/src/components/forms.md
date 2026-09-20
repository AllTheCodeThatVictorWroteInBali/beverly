# Forms

Forms collect user input and configuration. Keep the controls in this section consistent in spacing, labels, focus behavior, and validation feedback so they read as one family in the navigation and in the UI. A form is not just a collection of widgets; it is a structured interaction model that guides the user from intent to action.

## Included components

- Input
- Checkbox
- Toggle
- Select
- Search
- File Input
- Radio

Use these controls together when building settings screens, data entry flows, and other surfaces that ask the user to make choices or provide values.

<img src="../assets/component-form.svg" alt="Form component illustration" width="860" />

## Why form design matters

Forms are often the place where product friction becomes most obvious. A user can tolerate rough layouts in a dashboard or a media surface, but forms are usually evaluated on precision: is it clear what to enter, how to correct mistakes, and how to complete the task confidently?

This is why Beverly treats form controls as a disciplined system instead of a loose bag of widgets. The success of a form depends on consistency across labels, validation, focus, and feedback states.

## Basic principles

A strong form should:

- label every input clearly and directly
- keep the next action obvious
- provide feedback when fields are invalid or incomplete
- maintain a logical reading and tab order
- avoid unnecessary visual noise around the controls themselves

If a form produces confusion, it is usually because one of these fundamentals is missing.

## Layout and hierarchy

Forms benefit from strong vertical rhythm and a clear relationship between label, field, and helper text. A user should be able to scan the form quickly and understand what each part is doing without interpreting a dense or ambiguous layout.

The most effective patterns usually keep the following structure:

1. field label or legend
2. control itself
3. helper text or validation message
4. action area below or beside the form

This sequence gives the user one predictable path through the form and allows state messages to appear in a natural place.

## Validation and feedback

A good form does not wait until a submission attempt to reveal problems. Validation should be prompt, clear, and actionable.

For example:

- required fields should be easy to identify
- invalid input should describe the issue in plain language
- errors should appear near the relevant field
- success states should confirm the corrected value when appropriate

This is a core accessibility and usability principle: the user should understand what is wrong and how to fix it without guesswork.

## Interaction model

Beverly form controls should behave predictably across pointer and keyboard interaction. That means each control must have:

- a visible focus state
- a consistent keyboard path
- a clear label for assistive technology
- a predictable state model for selected, checked, disabled, or invalid values

Forms become much more trustworthy when the user can operate them using the same mental model regardless of input method.

## Example pattern

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_settings_form(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyStack::vertical(),
    )).with_children(|form| {
        form.spawn(BeverlyInput::new("Project name")
            .placeholder("internal-tools"));
        form.spawn(BeverlyCheckbox::new("Enable notifications"));
        form.spawn(BeverlySelect::new("Environment")
            .options(["Development", "Staging", "Production"]));
        form.spawn(BeverlyButton::primary("Save settings"));
    });
}
```

This pattern shows the right composition: labels remain clear, controls remain grouped, and the action area stays obvious at the end.

## Design system recommendation

The purpose of forms is to reduce friction, not create an additional layer of complexity. Beverly should keep control spacing consistent, field states visually legible, and errors easy to recover from. In a mature product system, forms are one of the clearest examples of how structure and semantics shape the user’s confidence.

## Summary

A well-designed form is calm, explicit, and easy to navigate. The strongest Beverly forms keep the relationship between control, label, state, and action obvious so that the user always understands the task and the system’s current status.