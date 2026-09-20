# Shaders

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Shaders are the most powerful way to express the “look” of a Beverly surface without creating one-off widget code. In a Bevy-based UI system, shader work should sit behind a shared material interface so widgets remain consistent while their surface properties change. The point is not to make every component custom-drawn with bespoke code; the point is to give surfaces a controlled visual language that can be reused, parameterized, and tuned at runtime.

## What a shader is doing here

A shader is a small program that runs on the GPU for each painted surface. In UI terms, it usually decides how a rectangle, panel, or glass layer should be colored, softened, highlighted, or blended.

For a Beverly surface, the shader is often responsible for:

- fill color and gradients
- border softening or edge blending
- corner rounding and shape masks
- glow, highlight, and shadow-like effects
- transparency, blur, or refraction
- focus and interaction overlays

Rather than hardcoding these decisions in a widget-specific render path, the system keeps them in a reusable material model. That preserves consistency across cards, panels, overlays, and inputs without creating many one-off visual implementations.

## Why shaders fit this architecture

Beverly’s architecture is layered and compositional. Shader logic fits naturally into that model because the visual treatment of a surface is separated from the semantics of the widget itself.

A button is still a button. It still has semantics, focus behavior, and interaction state. The shader simply decides how that surface is painted in a given theme or policy state.

This separation is useful because it allows the same widget type to express many visual variants without creating separate classes or code paths. A card can be plain, elevated, tinted, or glass-like while still being the same semantic component.

## A material-driven approach

The most maintainable pattern is material-driven rather than effect-driven. Instead of saying “this one widget gets a custom shader,” the system should say “this surface uses a shared material with runtime parameters.”

Conceptually, that looks like this:

```rust
// Conceptual shader use
let material = UiShapeMaterial {
    fill_color: Color::WHITE,
    border_color: Color::srgba(0.7, 0.7, 0.9, 1.0),
    corner_radius: 20.0,
    ..default()
};
```

This approach keeps the visual API readable. The widget can vary by theme, interaction, and policy without duplicating large shader code paths for each individual element. A shared material interface is the difference between a coherent visual system and a pile of custom drawing hacks.

## Typical responsibilities of UI shaders

### 1. Surface fill and gradients

A shader can generate smooth fills or gradients across a surface instead of relying on a flat solid color. This is useful for hero panels, branding surfaces, elevated cards, and focus states.

The key is to keep the gradient logic controlled and consistent. It should feel like part of the design system, not a random visual flourish.

### 2. Rounded geometry and soft edges

One of the most common UI uses for shaders is rounded corners, feathered edges, and soft border transitions. Instead of creating brittle geometry per component, the shader can apply shape-aware treatment using a shared parameter set such as corner radius, border width, and edge softness.

This makes panels, buttons, and overlays feel coherent even when they are built from many different component combinations.

### 3. Layered interaction states

Hover, focus, pressed, selected, and disabled states often need subtle but distinct surface treatments. A shader can help by changing a highlight amount, glow intensity, or alpha without creating a separate widget variant for each interaction state.

This keeps the interaction model expressive while preserving a single surface language.

### 4. Glass and blur surfaces

Glass-like surfaces rely on layering, transparency, and soft blending. In a UI system, this is often better expressed through a shared material pattern than by custom-coding each translucent panel.

The shader can apply soft alpha blending, subtle highlights, and edge treatment without forcing every panel to become a special-case effect.

## Why not just make a new shader for every widget?

The temptation is to introduce one custom shader per component or per visual state. That quickly becomes unmaintainable. The costs include:

- more shader variants to debug
- harder theme consistency
- more asset and material churn
- more complexity in build and validation
- more opportunities for visual drift between components

This is why Beverly favors a shared material model. A small set of well-defined shader behaviors is easier to reason about and easier to theme.

## A good material model

A good shader system for UI surfaces tends to expose a small, stable parameter set, such as:

- fill_color
- border_color
- border_width
- corner_radius
- shadow_strength
- glow_strength
- alpha or tint
- accent color or highlight color
- enabled or disabled state

The shader should not require every widget to know internal rendering logic. Instead, the material exposes the data needed for the visual effect, while the widget supplies semantics and layout.

This keeps the system flexible without making rendering code too broad or too magical.

## Performance discipline

Shaders are powerful, but power should be controlled. A UI framework must respect the fact that complex effects can become expensive when used too broadly.

The practical rule is:

- keep shaders simple and reusable
- prefer runtime parameters over many specialized variants
- limit heavy blur or refraction to surfaces that truly benefit from them
- avoid expensive effects on large numbers of small elements
- keep the effect stack predictable and bounded

This is especially important in dense interfaces, dashboards, and list-heavy UIs. A beautiful shader that runs on every row can become a performance liability very quickly.

## Accessibility and shader policy

Visual richness must not come at the cost of usability. Shader-based surfaces should respect accessibility policies such as:

- readable contrast under normal and high-contrast settings
- reduced motion or reduced visual noise when requested
- legible focus treatment even on glass or translucent surfaces
- consistent state visibility for selected, disabled, or error conditions

A glossy or translucent surface is fine only if it still supports reading, focus, and clear hierarchy. The look should never hide meaning.

This is why shader design should be part of the same policy layer as motion, contrast, and accessibility. The rendering system should understand environment preferences and adjust effect intensity appropriately.

## The rendering pipeline and shader interaction

Shaders are not usually the whole render pipeline on their own. They are part of a larger rendering flow:

1. layout determines where the surface sits
2. theme and policy data choose the appropriate material parameters
3. the material supplies fill, border, and effect information
4. the shader paints the surface according to those values
5. interaction and focus overlays are layered on top

This keeps the system coherent: layout decides shape and placement, materials decide visual expression, and shaders decide how the resulting surface is drawn.

## The key idea

Shaders are not a replacement for good UI design. They are a tool for expressing that design with consistent surface behavior and controlled runtime parameters. In Beverly, the best use of shaders is not to create one-off custom effects, but to give surfaces a careful, reusable visual vocabulary that aligns with layout, theme, interaction, and accessibility policies.

In short: shaders are how the framework expresses the look of a surface without sacrificing structure, consistency, or performance.
