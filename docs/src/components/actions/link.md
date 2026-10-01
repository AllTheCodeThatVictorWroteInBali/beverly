# Link

`Link` is an interactive UI component that emits a `LinkClicked` message when pressed. It does not navigate to a URL or choose a destination by itself. Your application reads the message and decides what action to perform.

> **The Link reports an activation. The application owns its meaning and destination.**

## Basic Usage

Add `Link` to a Bevy UI entity that also has `Button` and `Interaction`. Create the visual children yourself; the component plugin handles interaction feedback and emits the click message.

```rust
use bevy::prelude::*;
use beverly::components::link::{Link, LinkText};
use beverly::rendering::{Paint, Surface};

fn spawn_link(mut commands: Commands) {
    commands
        .spawn((
            Button,
            Interaction::None,
            Link::new("Open report"),
            Node {
                padding: UiRect::all(px(8.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            Surface::rounded_rect_fill(4.0, Paint::solid(Color::NONE)),
        ))
        .with_children(|parent| {
            parent.spawn((LinkText, text("Open report")));
        });
}
```

`BeverlyPlugin` installs `LinkPlugin`, which registers `LinkClicked` and runs the interaction and visual systems. If you are not using `BeverlyPlugin`, add `LinkPlugin` yourself.

## Click Messages

The link system watches entities with `Link` whose `Interaction` has changed. If the interaction is `Pressed` and the link is enabled, it writes one `LinkClicked` message containing the link entity.

```text
UI activation
    ↓
Interaction changes to Pressed
    ↓
LinkPlugin checks Link.disabled
    ↓
LinkClicked { entity }
    ↓
Application handles the action
```

Read the message in an application system:

```rust
use bevy::prelude::*;
use beverly::components::link::LinkClicked;

fn handle_link_clicks(mut clicks: MessageReader<LinkClicked>) {
    for click in clicks.read() {
        info!("Link activated: {:?}", click.entity);
        // Resolve the entity to an app route or action here.
    }
}
```

`LinkClicked` contains only `entity`; it does not contain a URL or destination. Attach your own route or action data to the entity, or look it up in application state. Message readers only receive messages written after they start reading, so keep the handler registered while links can be activated.

## Configuration

`Link::new(text)` creates the component with the supplied text metadata, no icon metadata, and `disabled` set to `false`.

- `.icon(icon)` stores an optional icon name as a `String`.
- `.disabled(disabled)` sets whether the link is disabled.

These are consuming builder methods, so they can be chained:

```rust
let link = Link::new("Documentation")
    .icon("book-open")
    .disabled(false);
```

The component does not create or update child text or icon nodes from these fields. In the current implementation, `text` and `icon` are metadata; create the visible child content explicitly. `LinkText` can mark a text child for the component's theme-aware color updates. `LinkIcon` is available as a marker for application-owned icon children, but the link plugin does not render or update it automatically.

## Disabled State

Disabled links do not emit `LinkClicked`. Their surface becomes transparent and marked `LinkText` children use the theme's disabled text color.

```rust
let link = Link::new("Unavailable report").disabled(true);
```

To change the state at runtime, replace the component with an updated value:

```rust
commands
    .entity(link_entity)
    .insert(Link::new("Open report").disabled(is_unavailable));
```

The component controls its own message and visual behavior. Your application should also ensure disabled links cannot be activated through any separate input or automation path it provides.

## Visual States

The built-in visual system uses the theme's primary, hover, active, and disabled text colors for `LinkText` children. On the link's `Surface`, it applies a subtle fill for idle, hovered, and pressed states, and clears the fill when disabled.

The link entity needs a `Surface` for that background feedback. The visual system does not create padding, layout, text, or icon content; those remain part of the entity and its children.

## Accessibility

Give the link a meaningful visible label and preserve a logical keyboard order. Since `Link` does not supply a destination or accessible name independently of its UI children, make sure your composed entity exposes an appropriate accessible name and action in your application.