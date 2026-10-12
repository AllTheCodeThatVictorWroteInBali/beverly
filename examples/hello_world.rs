//! Minimal "Hello, World!" example: a themed heading centered on screen.

use beverly::prelude::*;
use bevy::prelude::*;

#[path = "support/mod.rs"]
mod demo_support;
use demo_support::DemoAppExt;

pub fn main() {
    App::new().ui(my_ui()).run_demo();
}

fn my_ui() -> Ui {
    ui().width(percent(100))
        .height(percent(100))
        .center()
        .theme(light_theme())
        .children([text("Hello, World!")])
}
