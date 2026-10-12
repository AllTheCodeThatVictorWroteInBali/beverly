# 15.1 Hello World

## Live Demo

<iframe src="../assets/demos/index.html?demo=hello_world" title="Interactive Beverly hello world demo" loading="lazy" allow="fullscreen" style="display: block; width: 100%; height: min(820px, 90vh); min-height: 580px; border: 0;"></iframe>

<a href="../assets/demos/index.html?demo=hello_world" target="_blank" rel="noopener">Open the demo in a separate tab</a>.

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
