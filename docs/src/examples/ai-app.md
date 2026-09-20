# AI App

AI-native interfaces need trust cues, clear command feedback, and smooth state transitions. The strongest AI experiences make the system's status, permissions, and action boundaries feel explicit instead of hiding them behind clever animations.

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

## Trust patterns

AI interfaces succeed when they reveal what is happening: what tool is being used, what safety checks are in place, and what result the user should expect. The underlying Beverly primitives can support this through badges, alerts, confirmations, and clear content hierarchy.
