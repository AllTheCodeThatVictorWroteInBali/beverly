# Profiling

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Profiling reveals where UI behavior or asset costs are accumulating. In a system like Beverly, profiling is not a last-minute optimization step; it is part of how the architecture stays healthy as the interface grows. The goal is to identify which systems, assets, or interaction patterns are creating the most work and then fix the actual bottleneck rather than guessing.

## Why profiling matters

A UI may feel slow for many different reasons: layout churn, expensive shaders, a large list, redundant state updates, or a single overused asset. Without profiling, these issues are difficult to distinguish. A system may look fine in isolated code but still become a bottleneck when many widgets are active at once.

Profiling matters because it turns vague complaints like “the app feels laggy” into concrete questions such as:

- which system is consuming the most frame time?
- is layout or rendering dominating the cost?
- are unnecessary recomputations happening after each input event?
- are a few expensive materials or assets creating the slowdown?

This leads to better decisions and more durable performance work.

## Profiling in an ECS architecture

ECS makes profiling more actionable because the work is naturally segmented by system. Each system can be measured independently, which makes it easier to see whether the bottleneck is in:

- layout
- interaction updates
- theme resolution
- data filtering
- garbage or allocations
- rendering and materials
- accessibility evaluation

Instead of asking “is the app slow?” the real question becomes: “which system is doing too much work?” That is a much more productive problem to solve.

## Typical profiling signals

Common indicators of performance problems include:

- a single system consuming an unusually large share of frame time
- frequent layout work on large screens
- repeated state updates triggered by small input events
- high asset or material cost during transitions or scrolls
- visible frame drops while the user is interacting with a dense list or dashboard

These signals often reveal systemic issues, not isolated code bugs. A single expensive pattern may be repeated across many entities, which multiplies the cost across the whole UI.

## Measuring layout and update cost

In a layout-heavy UI, profiling should pay attention to the cost of:

- measuring text and content
- resolving nested layout constraints
- recalculating bounds after updates
- reflowing large sections after state changes

If a small change in one part of the app causes a large section to re-layout, that is a design signal. It means the system is over-coupled or too eager to recompute. Profiling helps reveal whether the update graph is too broad.

## Measuring render cost

Render cost is often more obvious than layout cost because it shows up as frame stalls, dropped frames, or poor responsiveness during motion. Profiling a render-heavy interface should check:

- material complexity
- shader cost
- number of draw calls
- number of visible elements being painted
- whether off-screen or hidden surfaces are still being processed

This is where batching, culling, and material discipline become measurable facts rather than abstract rules.

## Measuring asset cost

Assets are a common source of hidden overhead in UI frameworks. Large images, expensive textures, duplicated materials, or excessive shader parameters can create a large cost even when the logic itself is fine.

Profiling should therefore look at:

- which assets are loaded most often
- whether textures are too large for their actual use
- whether effect-heavy surfaces are applied broadly instead of selectively
- whether cached materials or bindings are being recreated unnecessarily

Good profiling keeps the visual design honest by tying aesthetic choices to runtime cost.

## Profiling user-visible behavior

Not all performance problems show up in raw timing metrics. Sometimes the issue is interaction latency, delayed scroll response, or a button that feels “sticky” after a state change. That means profiling should also consider user experience signals such as:

- scroll smoothness
- hover and tap responsiveness
- keyboard navigation latency
- animation fluidity
- transition clarity during dense data updates

A UI can be technically correct and still feel slow if state churn or repeated layout updates make interaction less responsive.

## A disciplined profiling workflow

A practical profiling workflow in a Bevy-style architecture is:

1. reproduce the issue under a realistic workload
2. identify the affected system or area of the UI
3. isolate the operation or interaction causing the spike
4. measure the hot path: layout, interaction, render, or asset cost
5. fix the root cause
6. re-measure and compare before and after

The key is to profile with intent. A single number is not enough; the pattern matters. Small slowdowns may be okay in one area, but a hot system that repeatedly fires on every frame or every input is a serious design problem.

## Profiling as part of design

The best performance work is not reactive; it is embedded into the design of the system. Beverly’s architecture encourages this by keeping concerns separated:

- UI composition stays distinct from app workflows
- layout is isolated from rendering
- interaction state stays separate from material state
- accessibility and policy constraints are built into the stack

Because those boundaries are explicit, profiling becomes more meaningful. It shows whether a bottleneck is in the wrong layer or caused by coupling across layers.

## The key takeaway

Profiling is how a system learns where its real costs are. In a layered, ECS-first framework, profiling is most valuable when it answers a system-level question: which part of the app is doing the expensive work, and why?

That makes profiling an architectural tool, not just a debugging habit. It helps Beverly keep the UI fast, understandable, and resilient as the application grows.
