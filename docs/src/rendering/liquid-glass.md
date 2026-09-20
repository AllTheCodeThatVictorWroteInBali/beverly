# Liquid Glass

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

The liquid-glass pattern is the most advanced visual treatment in a modern UI system. It adds translucency, refraction, highlight response, and a distinctive layered feel while preserving readability. This is the style most closely associated with theblocks_studio reference work: a single Surface model is given a glass-like material, a blur treatment, and optical highlight calculations rather than a one-off hack.

## Why liquid glass is special

Liquid glass is not just a transparency effect. It is a compositional surface treatment that tries to create a premium, layered, and atmospheric effect while still making the interface readable and usable. That makes it a powerful design tool when applied intentionally.

It works well for:

- modal shells and negotiation surfaces
- floating toolbars and utility panels
- premium overlays and command surfaces
- glass-like cards that need visual differentiation

The design goal is to create a feeling of depth and softness without losing the structure or legibility of the content inside the surface.

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

This pattern shows the right instinct: the glass effect is a surface treatment layered on top of a conventional UI component, not a completely different visual system.

## Considerations

- avoid using glass on dense text surfaces unless contrast remains acceptable
- fall back to opaque or semi-opaque surfaces under high contrast or reduced transparency
- keep the effect consistent across window sizes and theme changes
- prefer a shared material pipeline over many custom glass widgets
- ensure the effect does not hide focus states or critical affordances

## Performance note

Glass is visually rich but more expensive than plain fill and shadow. Use it on key surfaces such as modal shells, floating toolbars, or premium overlays, not on every single widget. This keeps the effect expressive while preventing the GPU cost from becoming a systemic problem.

## Accessibility and policy boundaries

A glass surface should never be allowed to reduce usability. That means the system should respect policies such as:

- reduced transparency
- high-contrast mode
- reduced-motion preferences
- strong focus visibility
- readable text at all times

If the interface needs to fall back to a more solid treatment in those modes, it should do so without creating a separate custom version of the component. The material system should handle the variation.

## The right balance

Liquid glass is best when it sits in a restrained, premium layer. It adds atmosphere, but it should not become a dominant visual element across the whole product. A single carefully chosen glass panel can feel luxurious; many of them can become visually noisy and expensive.

This is exactly why the surface model matters: it gives the framework a disciplined way to apply glass treatments only where they make sense. The effect stays expressive, but the product remains coherent and usable.

## The key idea

Liquid glass is a strong example of design expression built on top of a disciplined material system. It is visually rich, but it only works well when the rest of the product remains structurally clear. That is the difference between a premium product surface and a distracting aesthetic layer.
