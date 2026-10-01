//! Minimal "Hello, World!" example: a themed heading centered on screen.

use beverly::prelude::*;

fn main() {
    app()
        .window_size(960, 540)
        .title("Hello, Beverly")
        .theme(light_theme())
        .children([text("Hello, World!")])
        .run();
}
