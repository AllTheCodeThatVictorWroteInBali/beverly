# Media App

A media app is a strong example of layered glass surfaces, large action buttons, and full-width content composition. The design should keep the primary content visually dominant while the controls feel lightweight and immediate.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn setup_media_app(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyAppShell::new(),
        BeverlyTheme::dark(),
    )).with_children(|parent| {
        parent.spawn(BeverlyMediaHero::new("Drift"))
            .with_children(|hero| {
                hero.spawn(BeverlyBadge::new("New release").variant(BadgeVariant::Info));
                hero.spawn(BeverlyButton::primary("Play"));
                hero.spawn(BeverlyButton::secondary("Queue"));
            });

        parent.spawn(BeverlyPlaybackBar::new()).with_children(|bar| {
            bar.spawn(BeverlyButton::icon("skip-back"));
            bar.spawn(BeverlyButton::icon("play"));
            bar.spawn(BeverlyButton::icon("skip-forward"));
            bar.spawn(BeverlySlider::new("Volume").min(0.0).max(100.0).value(68.0));
        });

        parent.spawn(BeverlySection::new("Continue listening")).with_children(|section| {
            section.spawn(BeverlyMediaRow::new("Afterglow").meta("12 min left"));
            section.spawn(BeverlyMediaRow::new("Quiet Harbor").meta("26 min left"));
            section.spawn(BeverlyMediaRow::new("Night Commerce").meta("7 min left"));
        });
    });
}
```

## Design cues

- keep media controls large enough for pointer and keyboard interaction
- use muted surfaces behind the artwork and stronger contrast around the active controls
- reserve glass and blur for chrome shells or overlays, not dense text content
- keep the now-playing metadata readable even when the artwork is rich or animated

## Visual policy

In a real application, media views usually need a reduced-transparency fallback and a stronger focus outline for keyboard users, especially when the content is highly immersive.
