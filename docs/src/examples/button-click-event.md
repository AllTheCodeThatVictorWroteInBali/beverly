# 15.2 Button Click Event

This example extends 15.1 by adding a button, firing an app message when it is pressed, and listening for that message to log the click.

```rust
use bevy::prelude::*;
use beverly::prelude::*;

#[derive(Message)]
struct HelloButtonClicked;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_plugins(BeverlyPlugin)
		.insert_resource(ThemeResource {
			current: light_theme(),
		})
		.add_message::<HelloButtonClicked>()
		.add_systems(Startup, setup)
		.add_systems(Update, log_click_event)
		.run();
}

fn setup(mut commands: Commands) {
	commands.spawn(Camera2d);

	commands
		.spawn(Node {
			width: percent(100),
			height: percent(100),
			justify_content: JustifyContent::Center,
			align_items: AlignItems::Center,
			..default()
		})
		.with_children(|parent| {
			parent.spawn(button().text("Click me").on("click", emit_click));
		});
}

fn emit_click(commands: &mut Commands, _button: Entity) {
	commands.write_message(HelloButtonClicked);
}

fn log_click_event(mut reader: MessageReader<HelloButtonClicked>) {
	for _ in reader.read() {
		info!("Hello button clicked");
	}
}
```

When you press the button, this app writes `Hello button clicked` to the console.
