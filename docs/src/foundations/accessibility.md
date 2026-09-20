# Accessibility

Accessibility is not a bolt-on layer for a completed interface. It is a design constraint that shapes the structure, behavior, and visual system of every component. In Beverly, accessibility should be part of the default contract: semantic structure, keyboard reachability, stable focus, and legible states are expected before visual polish is considered complete.

## Why it matters

An interface that looks polished but is hard to operate creates friction for every user. People with motor, visual, cognitive, or attention-based differences often feel the impact most strongly, but accessibility gains benefit everyone:

- keyboard users can operate dense workspaces without a pointer
- low-vision users keep context when contrast and focus are clear
- users with sensory sensitivity benefit from reduced motion and lower visual noise
- screen-reader users understand structure and state without relying on color alone

Accessibility therefore affects product quality, not just compliance. It reduces mistakes, improves confidence, and makes interfaces usable in more contexts.

## Accessibility as system design

The best accessibility model is built into the component architecture. Beverly should prefer patterns that make semantics obvious and behavior predictable by default:

- controls should have names and roles
- state changes should be visible and understandable
- focus order should match the logical reading flow
- motion should support reduced-motion preferences
- labels and hierarchy should survive theme changes and visual enhancement layers

This is especially important in dynamic Bevy UI where layouts, panels, overlays, and animated containers can change rapidly. A robust system treats accessibility as a property of composition, not as a late-stage QA checklist.

## Semantic structure

Accessibility begins with the structure of the interface, not its final color palette.

A well-structured UI should make these questions answerable without guessing:

- What is this control for?
- What happens when the user activates it?
- What state is this component in?
- Where am I in the interface?
- What changed after the last action?

In practical terms, this means:

- use clear component hierarchy for cards, panels, nav items, forms, and dialogs
- label actions explicitly instead of relying on visual context alone
- expose state such as selected, disabled, expanded, loading, or error
- keep content organized in a readable and predictable order

### Example: semantic card + action layout

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_summary_card(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyCard::new("Team overview"),
    )).with_children(|card| {
        card.spawn(BeverlyText::new("12 active tasks and 3 alerts pending"));
        card.spawn(BeverlyButton::secondary("View details"));
    });
}
```

The card has a clear heading, supporting description, and explicit action label. A screen reader can interpret the layout without needing to infer intent from color or placement alone.

A semantically clear interface makes both assistive technologies and users more confident in how the app works.

## Keyboard and pointer parity

Keyboard support is not optional. If a control is important enough to appear in the UI, it should usually be operable without a mouse.

The baseline behavior should include:

- focusable interactive controls
- a logical tab order matching the layout
- visible focus styling at every interactive step
- Enter and Space support for buttons and action triggers
- Escape support for dismissing transient interfaces
- arrow-key behavior for menus, lists, and tab-like navigation

### Example: keyboard-first action bar

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_action_bar(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyToolbar::new(),
    )).with_children(|toolbar| {
        toolbar.spawn(BeverlyButton::primary("Save changes"));
        toolbar.spawn(BeverlyButton::secondary("Discard"));
        toolbar.spawn(BeverlyButton::subtle("More actions"));
    });
}
```

The actions are explicit, tabbable, and consistent. Users should be able to reach them in order without needing a pointer or hover-based affordance.

This creates parity between pointer and keyboard interaction. A screen with a nice hover state but no keyboard path is incomplete, because the interface fails for people who do not use a pointer and for anyone working in a keyboard-first workflow.

## Focus management

Focus is the map that lets a user understand where they are in an interface. When focus is invisible, unstable, or hard to recover, the application becomes disorienting.

A strong focus model does the following:

- clearly reveals which element is active
- keeps focus consistent while content updates
- moves into dialogs or drawers when they open
- restores focus to the triggering element when a dialog closes
- preserves context when filters or lists update

### Example: dialog focus handoff

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn open_confirmation_dialog(mut commands: Commands, mut focus_state: FocusState) {
    let trigger = focus_state.current();

    commands.spawn(BeverlyModal::new("Archive project")
        .body("This will move the project to archive status.")
        .confirm_label("Archive")
        .cancel_label("Cancel")
        .on_close(move |world| {
            if let Some(element) = trigger {
                world.focus(element);
            }
        }));
}
```

This pattern keeps the user in context. When the modal closes, focus returns to the element that opened it rather than vanishing or jumping to the top of the app.

Focus should never be treated as a visual flourish alone. It is a critical accessibility affordance and an operational necessity for keyboard users.

## Contrast and legibility

Strong legibility matters for all users but especially for people with low vision or glare sensitivity. Beverly should favor reliable contrast and meaningful separation over subtle styling tricks.

The system should ensure:

- text remains readable against its background
- important states are identifiable without relying on color alone
- focus rings remain visible in light and dark themes
- disabled content remains distinguishable without becoming unreadable
- panels, borders, and controls do not disappear into translucent surfaces

### Example: status styles that do not rely on color alone

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_status_surface(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyAlert::new("Sync failed")
            .variant(AlertVariant::Error)
            .with_icon("warning")
            .with_detail("Retry the upload to continue."),
    ));
}
```

The alert combines text, icon, and semantic state so the failure is understandable even when the user cannot rely on color distinctions alone.

Color is useful, but it should not be the only signal. A selected state can include a fill change, border emphasis, and text treatment together so the difference is clear under many viewing conditions.

## Motion and reduced motion

Animation can improve clarity, but it should not be required for understanding. Reduced-motion preferences are a key accessibility requirement because motion can trigger discomfort or distraction.

Beverly should treat reduced motion as a system-level policy:

- remove or soften decorative animation when the user prefers reduced motion
- keep essential state changes readable without flair
- preserve focus movement and semantic clarity
- avoid animation-only feedback that hides meaning or delays confirmation

### Example: motion-aware transition policy

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn spawn_loading_state(mut commands: Commands) {
    let prefers_reduced_motion = true;

    commands.spawn((
        NodeBundle::default(),
        BeverlyStatus::new("Uploading"),
    )).with_children(|status| {
        status.spawn(BeverlyText::new("72% complete"));
        if prefers_reduced_motion {
            status.insert(TransitionPolicy::Instant);
        } else {
            status.insert(TransitionPolicy::Fade);
        }
    });
}
```

This approach keeps progress visible even when motion is reduced. The interface still communicates progress without relying on a decorative animation to signal state.

A user should never have to guess whether a state changed if the motion has been suppressed. The system should still communicate success, error, loading, and selection clearly through text, structure, and visual state.

## Screen-reader support

Screen readers rely on semantics and naming, not just visual cues. For an interface to be understandable by assistive technology, every interactive element should expose the right role and label.

This means:

- buttons and actions need clear labels
- form controls need descriptive names and constraints
- tabs, menus, and dialogs need meaningful structure
- status surfaces need readable text that announces updates
- repeated elements should not be ambiguously named

### Example: labeled form field and action

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_named_form(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyStack::vertical(),
    )).with_children(|parent| {
        parent.spawn(BeverlyInput::new("Project name")
            .placeholder("internal-tools")
            .required(true));
        parent.spawn(BeverlyButton::primary("Create project"));
    });
}
```

The input label provides a screen-reader name, and the action button explains exactly what it does. This keeps the form understandable even when the visual context is reduced.

A visually impressive UI is not accessible if the user cannot tell what each control does. The best accessible products do not hide critical meaning behind layout alone.

## Error, warning, and status communication

Users need to understand state changes even when they are not looking directly at the most recent interaction. Error messaging, busy states, validation states, and progress indicators should be clear and discoverable.

A good status system combines:

- explicit labels or text
- meaningful hierarchy
- visible state treatment
- predictable announcement patterns for assistive technologies

For example, an error should not depend only on a red border or an icon. It should also include explanatory text and a clear path to recovery.

## Accessible interaction patterns

Accessibility is not just about a correct HTML or Bevy role assignment. It is also about the interaction model the user experiences as they move through the interface.

A good accessibility pattern makes several things true at the same time:

- interactive controls are reachable by pointer and keyboard
- state changes are communicated in a way users can perceive
- layout and focus remain stable while content updates
- important actions stay visible without requiring complex visual memory
- users can recover from mistakes without losing their place

This matters because accessibility is often shaped by the sequence of interactions, not only the final appearance. A screen may look polished but still be frustrating if the user cannot determine what happens next, where focus went, or whether a prior action succeeded.

### Predictable action patterns

Beverly should encourage predictable action patterns across all components:

- confirmation before irreversible actions
- visible progress during long-running tasks
- clear error states with specific recovery paths
- actions that have obvious names and outcomes
- consistent patterns for opening, dismissing, and returning from dialogs and side panels

The goal is not to make every control feel identical. The goal is to ensure that when a user interacts with an element, the interface responds in a way that is obvious, stable, and recoverable.

### Recoverability and error guidance

Accessible interfaces are forgiving. When something goes wrong, the user should understand what happened and what to do next.

This means:

- show errors near the relevant field or action
- preserve entered information when a validation failure occurs
- make destructive actions explicit and reversible when possible
- keep focus on the failed control or the nearest relevant recovery action
- describe the issue in plain language, not only in color or iconography

A poor pattern is one where the user sees a red border but no message, or where a dialog closes without returning focus to the original action. Those failures create friction and can become blockers for users with cognitive or motor differences.

### Progressive disclosure and clarity

Many interfaces become less accessible as they gather more content, options, and controls. Beverly should prefer progressive disclosure and clear hierarchy over trying to make everything visually equal.

That means:

- important actions should be obvious and easy to reach
- secondary actions should not compete visually with primary actions
- dense layouts should still preserve a clear reading order
- tooltips, popovers, and menus should not hide essential context from keyboard users
- multi-step workflows should keep the user oriented at each step

In other words, more power should not mean more noise. Good accessibility design makes the important path easier, not harder.

## Policy-aware accessibility

Accessibility is also shaped by environment and user preference. The system should not treat reduced motion, high contrast, or low-vision preferences as optional extras.

At the foundation level, Beverly should support policies such as:

- reduced motion for animation-heavy states
- high-contrast mode for interfaces where subtle visual separation is insufficient
- effective focus visibility under custom themes and branded surfaces
- stronger text legibility when transparency or blur reduces clarity

These policies should be part of the design system, not something each individual component manually implements. Once the system understands user preference, accessibility decisions become more consistent across the UI.

### Example: policy-driven contrast and motion

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn apply_accessibility_policy(mut commands: Commands) {
    let reduce_motion = true;
    let high_contrast = false;

    commands.spawn((
        NodeBundle::default(),
        BeverlySurface::new()
            .with_policy(AccessibilityPolicy::from_context(
                reduce_motion,
                high_contrast,
            )),
    ));
}
```

This is the practical version of accessibility as a system concern: the application responds to context without requiring developers to hand-tune every widget for each preference.

## Practical implementation guidance

Beverly should encourage developers to build accessibility into the component lifecycle:

1. define semantics before styling
2. keep state and labels explicit
3. maintain a stable focus model
4. support keyboard-first operation
5. test contrast and motion behavior at the theme level
6. verify that critical flows still work when visual effects are reduced
7. ensure recovery and error states are understandable without relying on color alone
8. validate that navigation remains coherent in dense, dynamic interfaces

### Example: component pattern checklist

```rust
struct AccessibleButton {
    label: &'static str,
    action: fn(),
    disabled: bool,
    focusable: bool,
}

impl AccessibleButton {
    fn render(&self) {
        assert!(!self.label.is_empty());
        assert!(self.focusable);
        // keyboard handling, visible focus styles, and state labels are required
    }
}
```

This simple pattern shows the underlying rule: an interactive component should define not only how it looks, but how it behaves, how it is named, and how it communicates state.

This is the strongest way to reduce repeated fixes. When multiple components follow the same semantic and state rules, the app feels coherent and resilient across every screen.

## Accessibility checklist

Before a component or screen is considered complete, it should answer these questions:

- Can the user reach every real action with the keyboard?
- Is the focused element clearly visible?
- Does focus move predictably when dialogs, panels, or lists update?
- Are labels and state meanings clear without relying on color?
- Does reduced motion preserve usability and clarity?
- Are status changes visible and understandable to assistive technology users?
- Is the layout readable in both standard and high-contrast modes?

When these conditions hold, the interface is not merely decorative. It is usable, resilient, and aligned with the broader Beverly design philosophy.
