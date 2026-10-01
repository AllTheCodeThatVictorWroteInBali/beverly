# 15.1 Hello World

This is the smallest useful Beverly app: a root `Ui` composition with a centered,
themed text node.

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn main() {
	App::new().ui(my_ui()).run();
}

fn my_ui() -> Ui {
	ui()
		.width(percent(100))
		.height(percent(100))
		.center()
		.theme(light_theme())
		.children([text("Hello, World!")])
}
```

This same example is available in the repository at `examples/hello_world.rs`.
