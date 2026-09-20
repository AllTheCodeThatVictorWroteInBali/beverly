# Radio

Radio groups represent exclusive choices where only one option can be selected at a time.

## Example

```rust
fn build_plan_selector(mut commands: Commands) {
    commands.spawn(BeverlyRadioGroup::new("Plan type")
        .option("Basic")
        .option("Pro")
        .option("Enterprise"));
}
```

## When to use

Use radio buttons when the set is mutually exclusive. Use checkboxes for independent boolean options.
