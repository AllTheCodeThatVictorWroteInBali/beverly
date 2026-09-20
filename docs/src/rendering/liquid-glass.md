# Liquid Glass

The liquid-glass pattern is the most advanced visual treatment in a modern UI system. It adds translucency, refraction, highlight response, and a distinctive layered feel while preserving readability.

This is the style most closely associated with theblocks_studio reference work: a single `Surface` model is given a glass-like material, a blur treatment, and optical highlight calculations rather than a one-off hack.

## Conceptual usage

```rust
fn build_glass_surface(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlySurface::new()
            .with_material(MaterialPreset::LiquidGlass)
            .with_border(Color::WHITE, 1.0)
            .with_blur(20.0)
            .with_alpha(0.7),
    ));
}
```

## Considerations

- avoid using glass on dense text surfaces unless contrast remains acceptable
- fall back to opaque or semi-opaque surfaces under high contrast or reduced transparency
- keep the effect consistent across window sizes and theme changes
- prefer a shared material pipeline over many custom glass widgets

## Performance note

Glass is visually rich but more expensive than plain fill and shadow. Use it on key surfaces such as modal shells, floating toolbars, or premium overlays, not on every single widget.
