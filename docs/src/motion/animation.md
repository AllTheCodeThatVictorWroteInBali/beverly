# Animation

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Animation is useful when a small, purposeful movement makes a system response feel immediate and understandable. Beverly favors restrained motion over expressive, loop-heavy effects. The best animations in a design system feel like feedback, not performance.

## Appropriate uses

Animation is valuable for:

- button press and release feedback
- active selection or status changes
- indicator movement tied to state
- safe micro-interactions that reinforce action without interrupting the task
- confirmation that a state has changed or a task is underway

It is not ideal for decorative effects that do not reveal state, structure, or intent.

## Design guidance

- Keep animation physically plausible and visually light
- Prefer easing that feels measured and calm
- Avoid frequent or repeated motion that competes with content
- Let the animation reinforce the action, not replace clarity
- Keep animation durations aligned with the broader motion system

## Motion categories

### Micro-interactions

Micro-interactions should feel responsive and immediate. A press state, hover reveal, or selection pulse should be quick enough that the user perceives the system as responsive rather than delayed.

A micro-interaction should answer the question, “Did the system receive my input?” without creating a separate entertainment layer.

### Status and feedback

When something changes, such as a success state or an active filter, animation can help confirm a result. The motion should read as confirmation, not as a separate event.

This makes animation a communication tool rather than a purely aesthetic one.

### Continuous motion

Continuous motion is acceptable only when it is informative. Progress indicators, status pulses, and subtle ambient motion can support awareness, but they should never become the primary focus of the interface.

In a professional product, continuous decorative motion is usually a sign that the animation is doing too much.

## Practical limits

Avoid:

- looping animations on static content
- large objects moving across the viewport for no reason
- repeated bounce or wobble patterns
- motion that makes reading difficult or makes controls feel unstable
- long, accidental delays that create the feeling of lag

The best motion is brief, clear, and aligned with the action it supports.

## Reduced-motion fallback

When `prefers-reduced-motion` is active, suppress non-essential animation and rely on color, label, structure, and contrast to communicate state. A state change should remain understandable without the visual flourish.

This is a critical design rule because animation should support the product’s meaning, not become a requirement for comprehension.

