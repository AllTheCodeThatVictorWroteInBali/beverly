# 15.1 Hello World

This is the smallest useful Beverly app: start Bevy, add `BeverlyPlugin`, and render centered text.

```rust
use bevy::prelude::*;
use beverly::components::text::TextRole;
use beverly::prelude::{light_theme, BeverlyPlugin, ThemeResource, ThemedText};

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_plugins(BeverlyPlugin)
		.insert_resource(ThemeResource {
			current: light_theme(),
		})
		.add_systems(Startup, setup)
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
			parent.spawn((Text::new("Hello, World!"), ThemedText::new(TextRole::Heading)));
		});
}
```

This same example is available in the repository at `examples/hello_world.rs`.
