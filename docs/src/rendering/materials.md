# Materials

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Materials connect a UI surface to its paint behavior. A material is the place where the app defines how a widget is filled, how it reacts to focus, and how it handles effects like blur, highlight, or noise. In a system like Beverly, materials are not just shaders with a few parameters; they are the interface between semantic UI state and the visual treatment applied to the surface.

## What a material does

A material describes the visual rules for a surface. It answers questions such as:

- What color should this surface use?
- How strong is the border?
- Does this surface have a raised, flat, or glass-like treatment?
- How should hover, focus, and disabled states change the appearance?
- Does this surface support blur, transparency, or highlight layers?

This makes the material a shared contract that multiple component types can use while preserving their own semantics. A card, a modal, a button, and a panel can all rely on a consistent material model even if they are visually distinct.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn spawn_material_surface(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlySurface::new()
            .with_fill(Color::srgb(0.08, 0.09, 0.15))
            .with_border(Color::srgb(0.55, 0.68, 1.0), 1.5)
            .with_material(MaterialPreset::Elevated),
    ));
}
```

This is a good example of material-driven UI. The component remains a surface, but the actual visual treatment is determined by a reusable preset rather than by a custom one-off style path.

## Why materials matter

Materials are important because they keep visual logic centralized and coherent. Without a material model, the system ends up scattering styles across widgets, and each component becomes responsible for both semantics and paint decisions.

A material model solves several problems:

- consistent surfaces across the app
- easier theming and theme swapping
- better effect tuning in one place
- simpler support for reduced transparency and contrast policies
- fewer ad hoc visual variants to maintain

## Material goals

- maintain a semantic surface layer instead of ad hoc color values
- keep the same material contract across components
- allow theme swaps without rewriting widgets
- make effect tuning easy to adjust in one place
- provide a predictable place for interaction-driven visual changes

## Material hierarchy

A good material system usually has a few conceptual layers:

1. base fill and border
2. surface behavior such as elevated, flat, or glass-like
3. optional effect parameters such as blur, highlight, and alpha
4. interaction tuning such as hover and focus overlays
5. policy adjustments for contrast, reduced motion, and transparency

This keeps the system readable. The material is not a catch-all for every visual decision, but a structured place to define how a surface behaves under normal and exceptional conditions.

## Material and theme alignment

Material definitions should not be tied to a single widget. Instead, they should respond to theme tokens and broader design policy. That means the theme can change colors, surfaces, and contrast while the material interface remains stable.

This allows a product to evolve visually without rewriting component logic. The same button material can work in a light theme, dark theme, or even a branded experimental theme so long as the material contract remains consistent.

## Interaction-aware materials

Materials should react to the state of the element. A focused control, a hovered button, or a disabled panel often need slightly different surface values even when they remain the same semantic object.

Examples:

- a hovered card may get a slightly brighter outline
- a focused control may gain a strong accent ring
- a disabled surface may reduce contrast and motion
- a selected surface may use a more distinct accent tint

This makes materials part of the interaction layer rather than a static background detail.

## The core idea

A good material model is the bridge between UI semantics and visual treatment. It keeps the app consistent, combinable, and themeable without turning each widget into a bespoke rendering implementation.

In Beverly, that means materials are not a secondary concern. They are a core primitive in the rendering system and one of the most important tools for building coherent, polished surfaces.
