# Blur

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Blur is one of the most effective ways to show depth and separation in a layered interface. Used sparingly, it helps create readable glass or ambient surfaces without reducing legibility. In a product UI, blur is most useful when it reinforces hierarchy and layering rather than when it becomes a decorative effect on its own.

## What blur does in UI

A blurred surface suggests distance, depth, or separation from the content beneath it. It works well for:

- modal backdrops
- floating utility surfaces
- translucent overlays and chrome
- layered panels that need subtle separation
- premium product surfaces that want a glass-like feel

The key is that blur should support the interface’s readability, not undermine it. If text becomes hard to read, the surface has crossed from excellent layering into bad legibility.

## Example

```rust
fn spawn_blurred_backdrop(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlySurface::new()
            .with_blur(18.0)
            .with_tint(Color::srgba(0.1, 0.12, 0.18, 0.55)),
    ));
}
```

This is a good use of blur: the surface is still legible, application-like, and layered, but it does not feel harsh or final. It creates depth while preserving enough contrast to remain usable.

## Use cases

- modal overlays
- floating panes and sidebars
- translucent chrome and utility surfaces
- layered data views with a quiet background
- premium application surfaces that need a softened, atmospheric layer

## Accessibility note

If blur reduces contrast too much, the surface should fall back to a more opaque treatment or a higher-contrast border. In accessibility-driven design, blur cannot be allowed to hide critical text or destroy state readability.

## Blur and context

The best use of blur is usually conservative. A little blur can create a sense of separation and focus, but too much blur will make a layer feel vague and reduce confidence in the interface. In dense workspaces, the user needs to understand the structure at a glance.

This means blur should be used with clear boundaries:

- behind modal shells or drawers
- around focus regions or transient surfaces
- on layered chrome that should feel ambient rather than dominant
- not on every text-heavy or dense data surface by default

## Performance and visual cost

Blur is often more expensive than a flat fill or a simple shadow because it requires more GPU work and more blending. That does not make it invalid; it just makes proper scope and restraint important.

A thoughtful bloom or blur policy keeps the effect active only where it adds value. This is especially important for dashboards, data-heavy screens, and mobile or lower-power devices.

## The design principle

Blur is most successful when it communicates depth without hiding structure. It should make the interface feel layered and refined, while still allowing the user to read, focus, and act with confidence.

Used correctly, blur is a quiet luxury. Used without constraint, it becomes a readability problem.
