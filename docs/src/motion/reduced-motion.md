# Reduced Motion

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Accessible interfaces should respect reduced-motion preferences without losing semantic clarity. Beverly exposes a shared reduced-motion policy so applications can coordinate this behavior. The goal is not to remove all motion, but to remove the motion that is noisy, distracting, or unnecessary while preserving the meaning of the interface.

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

## Beverly Configuration

`BeverlyPlugin` initializes `AccessibilityVisualPolicyResource`. Its `current.reduced_motion` value defaults to `false` and can be initialized from `UI_REDUCED_MOTION` (`1`, `true`, `TRUE`, `yes`, or `on` enables it). The application can also update this resource at runtime, for example in response to its own settings or platform-preference integration.

The policy is not yet applied uniformly to every motion feature. Beverly's `Transition` property animations collapse decorative duration and delay to zero when reduced motion is enabled; semantic transitions keep their timing. `UiMotion` enter/exit presets and component-specific motion such as loading indicators do not currently all consult this policy. Review each feature's documentation and supply a static alternative where needed. Beverly does not automatically detect an operating-system reduced-motion preference.

```rust
use bevy::prelude::*;
use beverly::theme::AccessibilityVisualPolicyResource;

fn enable_reduced_motion(mut policy: ResMut<AccessibilityVisualPolicyResource>) {
	policy.current.reduced_motion = true;
}
```

A centralized setting is still valuable, but applications should not infer that every animation is disabled merely because this flag is enabled.

## Rule of thumb

If motion is removed and the interface still communicates the correct state, the design is successful. Reduced motion should not reduce intelligibility; it should reduce distraction. The product should feel relaxed, not stripped of meaning.

