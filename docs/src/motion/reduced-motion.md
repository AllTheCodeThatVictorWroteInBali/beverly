# Reduced Motion

Accessible interfaces must respect reduced-motion preferences without losing semantic clarity. Beverly treats reduced motion as a first-class policy rather than an exception.

## Core principle

When a user has requested reduced motion, the system should remove or tone down non-essential animation while preserving the meaning of the interface. The design still needs to communicate state, hierarchy, and progress clearly.

## Recommended behavior

- disable decorative loops and repeated visual flourishes
- shorten transition durations or replace them with instant state changes
- preserve focus order and context during dynamic layout updates
- use text, labels, and structural cues to carry meaning when animation is reduced

## Patterns that should calm down

- hover and press effects
- large panel transitions
- list insertion/removal motion
- loading indicators that are purely decorative
- scroll and reveal motion that is not essential to navigation

## Patterns that should remain expressive

A user who requests reduced motion still needs clarity about:

- current state
- ongoing tasks
- errors and warnings
- focus movement

These cues can remain clear without requiring elaborate animation.

## Implementation guidance

Motion-related policies should be driven by a shared configuration layer so the same preference controls across components. A consistent reduced-motion mode makes the application feel calmer and more reliable.

## Rule of thumb

If motion is removed and the interface still communicates the correct state, the design is successful. Reduced motion should not reduce intelligibility; it should reduce distraction.

