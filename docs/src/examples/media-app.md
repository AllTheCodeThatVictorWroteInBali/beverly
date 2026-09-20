# Media App

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

A media app is a strong example of layered glass surfaces, large action buttons, and full-width content composition. The design should keep the primary content visually dominant while the controls feel lightweight and immediate. This is a case where the system’s visual polish should support the content, not compete with it.

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
- let the hero artwork do the emotional work while the chrome stays quiet and functional

This is a key Beverly principle: the visual treatment should support the interaction, not distract from it.

## Visual policy

In a real application, media views usually need a reduced-transparency fallback and a stronger focus outline for keyboard users, especially when the content is highly immersive. A media shell that becomes unreadable because the glass is too aggressive fails its main job.

## Why media surfaces are special

Media experiences have a different hierarchy than productivity tools. The content is often emotionally charged and visually rich, while the controls must stay lightweight and easy to use. That creates a tension: the media should feel cinematic, but the interaction should still feel precise and grounded.

Beverly handles this by emphasizing contrast, structure, and purpose. Large background imagery can be immersive, but the user still needs strong control affordances, a clear play state, and trackable focus.

## Product-level guidance

When building media interfaces, decide which surfaces are ornamental and which are functional. The album art, hero, and ambient layers can be expressive. The controls, metadata, and navigation layers should remain highly legible. The interface should feel premium without becoming difficult to read or difficult to control.
