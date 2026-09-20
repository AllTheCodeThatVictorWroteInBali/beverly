# Radio

<img src="../assets/radio.svg" alt="Radio component illustration" width="860" />

Radio groups represent exclusive choices where only one option can be selected at a time. They are ideal for decision points where the user must choose a single valid answer from a small set.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_plan_selector(mut commands: Commands) {
    commands.spawn(BeverlyRadioGroup::new("Plan type")
        .option("Basic")
        .option("Pro")
        .option("Enterprise"));
}
```

## When to use

Use radio buttons when the set is mutually exclusive. Use checkboxes for independent boolean options.

## Guidance

Keep the options clear, comparable, and appropriate to the user context. If the choices are long or complex, consider a richer pattern such as a select or a dedicated settings panel with explanatory copy.

## Accessibility

The group should have a visible label and a clear selected state. Keyboard support is essential so the user can move between options without a pointer and understand which choice is active.
