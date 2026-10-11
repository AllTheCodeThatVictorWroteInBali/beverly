# Button

`button()` starts a primary `BeverlyButton` builder. Add its visible content with
`.text(...)` or `.icon(...)`, attach an application callback with `.on("click", ...)`,
or provide an ordered set of `ButtonChild` values with `.children(...)`.

For ordinary text content elsewhere in a Beverly app, use `text("...")`. It
creates a Bevy `Text` component that is picked up by Beverly's default
typography pipeline.

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn save_document(_commands: &mut Commands, _button: Entity) {
	// Update application state or write an application message here.
}

app()
	.children([button().text("Save").icon("save").on("click", save_document)])
	.run();
```

`BeverlyPlugin` installs `ButtonPlugin`. If the component is used without the
aggregate plugin, add `ButtonPlugin` yourself.

## Live Demo

<iframe
	src="../assets/button-demo/index.html"
	title="Interactive Beverly button showcase"
	loading="lazy"
	allow="fullscreen"
	style="display: block; width: 100%; height: 580px; border: 0; background: #101215;"
></iframe>

[Open the demo in a separate tab](../assets/button-demo/index.html).
The embedded canvas supports pointer interaction; use the top-right control to
switch between light and dark themes.

To regenerate the bundle locally, install the `wasm32-unknown-unknown` target
and Trunk `0.21.14`, then run:

```sh
mdbook build docs
bash web/button-demo/build.sh
```

The Pages workflow builds and publishes this bundle automatically.

## Content

The convenience constructors remain available when the label is known at the
call site:

```rust
let primary = BeverlyButton::primary("Save");
let danger = BeverlyButton::danger("Delete").disabled(true);
let text_only = BeverlyButton::text_button("Learn more");
```

The fluent form is useful when content is assembled conditionally:

```rust
let save = button().text("Save");
let icon_only = button().icon("save");
let custom = button().children([
	ButtonChild::icon("save"),
	ButtonChild::text("Save document"),
	ButtonChild::custom(spawn_badge),
]);
```

Custom children are setup functions with signature
`fn(&mut ChildSpawnerCommands, Color)`:

```rust
fn spawn_badge(parent: &mut ChildSpawnerCommands, color: Color) {
	parent.spawn((text("NEW"), TextColor(color)));
}
```

The first text child is used as the accessible name for a custom-content
button. Icon-only buttons should add a text child or provide an application
accessible-name component rather than relying on the icon name.

## Activation

`.on("click", save_document)` accepts a function pointer with the signature
`fn(&mut Commands, Entity)`. This keeps application behavior outside the
component while allowing the button to invoke the command when it is pressed
by pointer or keyboard activation:

```rust
fn open_settings(commands: &mut Commands, button: Entity) {
	commands.entity(button).insert(SettingsRequested);
}

let settings = button().text("Settings").on("click", open_settings);
```

Supported event names include `click`, `clickdown`/`pointerdown`,
`clickup`/`pointerup`, `mouseenter`/`pointerenter`,
`mouseleave`/`pointerleave`, `mousemove`/`pointermove`, `doubleclick`,
`dragstart`, `dragmove`, `dragend`, `dragcancel`, `longpress`, `scroll`,
`load`, and `unload`.

`load` runs when the button is added to the world. `unload` runs when its
`BeverlyButton` component is removed. Pointer and gesture handlers use the
same callback signature: `fn(&mut Commands, Entity)`.

Disabled buttons do not invoke pointer, click, or gesture handlers. Buttons do
not emit application events; all application behavior must be attached
explicitly with `.on(...)`.
