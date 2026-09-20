# AI App

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

AI-native interfaces need trust cues, clear command feedback, and smooth state transitions. The strongest AI experiences make the system's status, permissions, and action boundaries feel explicit instead of hiding them behind clever animations. In other words, AI interfaces should feel informative, not mysterious.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn setup_ai_app(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyChatShell::new(),
        BeverlyTheme::dark(),
    )).with_children(|parent| {
        parent.spawn(BeverlySidebar::new()).with_children(|sidebar| {
            sidebar.spawn(BeverlyNavItem::new("Agents").active(true));
            sidebar.spawn(BeverlyNavItem::new("Memory"));
            sidebar.spawn(BeverlyNavItem::new("Permissions"));
        });

        parent.spawn(BeverlyChatPane::new()).with_children(|chat| {
            chat.spawn(BeverlyCard::new("Agent status")
                .with_body("Ready to run analysis"));
            chat.spawn(BeverlyAlert::new("Permissions reviewed").variant(AlertVariant::Info));
            chat.spawn(BeverlyMessage::from_user("Summarize the monthly report"));
            chat.spawn(BeverlyMessage::from_agent("I highlighted the anomalies and the revenue delta."));

            chat.spawn(BeverlyComposer::new()).with_children(|composer| {
                composer.spawn(BeverlyInput::new("Prompt"));
                composer.spawn(BeverlyButton::primary("Run task"));
                composer.spawn(BeverlyButton::secondary("Attach"));
            });
        });
    });
}
```

## Design guidance

- surface agent state visibly, especially in long-running or tool-using workflows
- make confirmations explicit when actions are irreversible or carry high impact
- keep streamed output readable and scannable, with structure more than decorative motion
- use reduced motion and high-contrast modes to keep the system predictable and accessible
- make permission scope visible near the relevant action rather than burying it in settings
- separate the user’s intent from the system’s interpretation whenever the distinction matters

This is a crucial design principle: in AI systems, the user must feel like they are steering the system rather than being surprised by it.

## Trust patterns

AI interfaces succeed when they reveal what is happening: what tool is being used, what safety checks are in place, and what result the user should expect. The underlying Beverly primitives can support this through badges, alerts, confirmations, and clear content hierarchy.

An AI interface is trustworthy when it can answer four questions quickly:

1. What is the agent doing?
2. What can it access?
3. What is the current state of the task?
4. What action should the user take next?

If those questions remain unclear, the interface is likely over-abstracted or under-explained.

## Why this matters for system design

The AI interface is not just a chat surface; it is a control layer for an autonomous but constrained system. That means states like “thinking,” “tool call in progress,” “needs user confirmation,” or “permission denied” must be obvious. The interface should not hide uncertainty behind a smooth animation or a minimalistic status indicator that the user cannot interpret.

Good Beverly-style AI surfaces let the user maintain agency. They keep the interaction readable, explicit, and resilient even when the model produces uncertain or partial output.
