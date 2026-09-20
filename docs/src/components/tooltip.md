# Tooltip

Tooltips provide contextual help for controls or labels without permanently consuming layout space. They are useful when a brief explanation will help the user without cluttering the interface permanently.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_tooltip(mut commands: Commands) {
    commands.spawn(BeverlyTooltip::new("Sync now")
        .text("Starts a full metadata refresh"));
}
```

## Use it when

- a control needs a quick explanation
- visual affordance is not obvious
- the extra text will distract if always visible
- the action is common and the help is contextual rather than essential

## Guidance

Tooltips should be short and helpful. They should never be the only place where critical instructions or required states are described; the main label or supporting content should carry essential meaning.

## Accessibility

Tooltips work best as supplemental guidance, not as the sole mechanism for understanding a control. If a user cannot discover the meaning of an action from the visible label, the underlying control should be improved rather than depending on hover-only help.
