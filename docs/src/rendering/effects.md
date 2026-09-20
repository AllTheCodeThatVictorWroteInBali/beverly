# Effects

Beverly effects should emphasize state and hierarchy instead of distracting from the content. Effects are most useful for focus, depth, and transitions between states.

## Example

```rust
fn build_effect_card(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyCard::new("Sync status")
            .with_shadow(ShadowPreset::Soft)
            .with_glow(Color::srgb(0.35, 0.75, 1.0)),
    ));
}
```

## Good effect patterns

- subtle hover lift
- focus rings on interactive controls
- soft elevation for modal and overlay surfaces
- restrained transparency for high-value UI containers

## Reduce visual noise

As in theblocks_studio, effects should be controlled by a shared policy layer. This makes it easy to disable them for reduced motion, reduced transparency, or high-contrast applications.
