//! Badge demo: every size, one row light and one row dark.
//! Run with `-- dark` to start in dark mode; the top-right button toggles light/dark.

use beverly::prelude::*;
use bevy::prelude::*;

#[path = "support/mod.rs"]
mod demo_support;
use demo_support::DemoAppExt;

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .add_systems(Startup, setup)
        .run_demo();
}

fn setup(mut commands: Commands) {
    spawn_themed_page(&mut commands, |root| {
        let sizes = [
            ("XS", BadgeSize::Xs),
            ("S", BadgeSize::S),
            ("M", BadgeSize::M),
            ("L", BadgeSize::L),
            ("XL", BadgeSize::Xl),
        ];

        for dark in [false, true] {
            root.spawn(Node {
                max_width: Val::Percent(100.0),
                min_width: Val::Px(0.0),
                flex_shrink: 0.0,
                flex_wrap: FlexWrap::Wrap,
                align_items: AlignItems::Center,
                column_gap: Val::Px(16.0),
                row_gap: Val::Px(16.0),
                ..default()
            })
            .with_children(|row| {
                for (label, size) in sizes {
                    let badge = Badge::new(label).sized(size);
                    let badge = if dark { badge.dark() } else { badge.light() };
                    spawn_badge(row, badge, Handle::default());
                }
            });
        }
    });
}
