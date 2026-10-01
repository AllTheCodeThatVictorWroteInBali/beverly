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

fn main() {
	app()
		.children([text("Hello, World!")])
		.run();
}
```

This same example is available in the repository at `examples/hello_world.rs`.
