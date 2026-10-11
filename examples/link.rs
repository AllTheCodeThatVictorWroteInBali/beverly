//! Link demo: interactive and disabled links that follow light/dark mode.
//! Run with `-- dark` to start in dark mode; the top-right button toggles light/dark.

use beverly::components::link::LinkClicked;
use beverly::prelude::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_systems(Update, log_link_clicks)
        .ui(ui()
            .theme(theme_from_cli_args())
            .width(percent(100))
            .height(percent(100))
            .center()
            .children([link("Visit Beverly")
                .aria("Opens the Beverly website in your web browser")
                .to("https://www.beverlyui.com")
                .external()]))
        .run();
}

fn log_link_clicks(mut clicks: MessageReader<LinkClicked>) {
    for click in clicks.read() {
        info!(entity = ?click.entity, "Link activated");
    }
}
