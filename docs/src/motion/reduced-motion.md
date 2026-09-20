# Reduced Motion

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Accessible interfaces must respect reduced-motion preferences without losing semantic clarity. Beverly treats reduced motion as a first-class policy rather than an exception. The goal is not to remove all motion, but to remove the motion that is noisy, distracting, or unnecessary while preserving the meaning of the interface.

## Core principle

When a user has requested reduced motion, the system should remove or tone down non-essential animation while preserving the meaning of the interface. The design still needs to communicate state, hierarchy, and progress clearly.

This is a user-focused principle: the application should reduce visual stimulation without making the user work harder to understand what is happening.

## Recommended behavior

- disable decorative loops and repeated visual flourishes
- shorten transition durations or replace them with instant state changes
- preserve focus order and context during dynamic layout updates
- use text, labels, and structural cues to carry meaning when animation is reduced
- keep motion policies centralized so the app behaves consistently across components

## Patterns that should calm down

- hover and press effects
- large panel transitions
- list insertion/removal motion
- loading indicators that are purely decorative
- scroll and reveal motion that is not essential to navigation
- any movement that adds emotional intensity without adding meaning

## Patterns that should remain expressive

A user who requests reduced motion still needs clarity about:

- current state
- ongoing tasks
- errors and warnings
- focus movement
- state transitions in active panels or dialogs

These cues can remain clear without requiring elaborate animation. In many cases, static or subtle changes are more readable than a more dramatic motion sequence.

## Implementation guidance

Motion-related policies should be driven by a shared configuration layer so the same preference controls across components. A consistent reduced-motion mode makes the application feel calmer and more reliable.

This also keeps the motion system more maintainable. Instead of each component deciding independently how to react to reduced-motion, a shared policy lets the app behave coherently across the whole experience.

## Rule of thumb

If motion is removed and the interface still communicates the correct state, the design is successful. Reduced motion should not reduce intelligibility; it should reduce distraction. The product should feel relaxed, not stripped of meaning.

