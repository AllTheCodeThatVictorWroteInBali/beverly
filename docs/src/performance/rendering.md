# Rendering

Rendering performance depends on practical batching, culling, and asset discipline. In Beverly, rendering is not a separate afterthought bolted onto the UI; it is part of the same ECS-driven architecture that handles layout, semantics, and interaction. The system keeps visual work organized so that a widget can render cleanly without becoming expensive or hard to reason about.

## What rendering means in ECS

Rendering is the stage where the app turns layout state and visual data into pixels on screen. In a Bevy-based UI stack, that usually means the renderer works from entities that already have the right components: transform, layout, paint, material, visibility, and style tokens.

A typical flow looks like this:

1. the layout system determines the position and size of each UI element
2. the style system resolves visual tokens such as color, radius, and spacing
3. the material system selects the correct surface treatment
4. the render system draws the final visual output

This keeps rendering as a data-driven step rather than a hidden, imperative side effect inside each widget.

## Material-first surfaces

Beverly treats surfaces as first-class UI objects. A surface is not just “some background with a color.” It is a structured visual layer that may include:

- fill color or gradients
- borders and corner radii
- shadows and highlights
- blur or translucency
- focus rings and interaction overlays
- shader-backed material effects

This is the same philosophy described in the rendering overview: UI surfaces carry semantic paint and border information, and effects are layered intentionally rather than injected ad hoc.

Because these aspects are material-aware, the same component model can support a plain panel, a glass card, a raised interaction surface, or a branded onboarding tile without inventing a completely different rendering path for each case.

## The render pipeline

A practical rendering pipeline in Beverly-style architecture usually moves from coarse to fine:

- determine which entities are visible
- compute their layout and bounds
- resolve theme and surface parameters
- gather the set of paint operations needed
- apply effects such as shadows, gradients, or blur
- draw the final composite surface

This ordering matters. It prevents the render system from doing expensive work for elements that are not visible or are outside the current viewport. It also makes the visual stack easier to debug because each layer is explicit.

## Why batching and culling matter

Rendering performance is not only about “how pretty is the shader.” It is also about how many draw calls and paint operations the app creates.

Two practical concerns are especially important:

### Batching

When many similar elements share the same material or surface settings, they can often be drawn together in a single pass. This reduces CPU overhead and keeps the GPU busy with fewer state changes.

### Culling

Visible elements should be drawn; hidden or clipped elements should not. Many UI systems waste work by rendering all surfaces in a scene even when only a fraction is on screen. In a dense dashboard or list-heavy interface, this can become a real bottleneck.

In ECS terms, culling is often implemented by checking the entities that match a renderable set and filtering them based on visibility and viewport bounds before the final draw step.

## Materials and effects

Beverly’s approach is intentionally selective with advanced effects. A glass surface, blur layer, or gradient is valuable when it contributes clarity or visual hierarchy; it becomes expensive and distracting when applied everywhere.

This leads to a disciplined effect stack:

1. base fill
2. gradient or border layer
3. shadow and surface depth
4. interaction overlay
5. blur or translucency only when needed
6. accessibility and contrast safeguards

That progression is important because it preserves readability and avoids overusing complex shaders on simple UI surfaces.

## Interaction and rendering are connected

Rendering is not independent of interaction. A button may have a different visual treatment when hovered, pressed, or focused. The same entity can carry interaction state and render styling, and the render system can react to that data without creating custom logic for every widget instance.

For example:

- hover state may change shadow and fill color
- focus state may add a ring or outline
- disabled state may reduce contrast and suppress motion
- pressed state may slightly inset the surface to create tactile feedback

This is why the rendering layer should be modeled as part of the same UI state machine as the rest of the system, not as a disconnected drawing step.

## Accessibility and rendering

A performant UI is not just a fast UI; it is a readable and usable one. Rendering must respect reduced-motion, contrast requirements, and semantic clarity.

For example:

- avoid relying only on color to communicate state
- keep focus indicators visible and distinct
- reduce translucency in high-contrast modes
- avoid heavy motion and shimmer effects for users who prefer reduced motion

The render system can respond to policy data and accessibility preferences without separate code paths for every surface.

## The core idea

Rendering in Beverly is best understood as a layered, data-driven process that sits inside the ECS architecture. Entities hold the state, materials define the paint model, and systems determine how the final visual output should be composed.

The result is a rendering model that is:

- composable
- visually expressive
- guided by material semantics
- efficient enough for dense interfaces
- respectful of accessibility and policy constraints

In short, rendering is not an isolated layer of drawing code; it is the visual expression of the same ECS model that drives layout, interaction, and state.
