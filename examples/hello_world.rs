//! Minimal "Hello, World!" example: a themed heading centered on screen.

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
