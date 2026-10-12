//! Minimal example demonstrating Beverly buttons and button groups.

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
            current: light_theme(),
        })
        .add_systems(Startup, setup)
        .run_demo();
}

fn setup(mut commands: Commands) {
    commands
        .spawn((
            bevy::ui_widgets::ScrollArea,
            ScrollPosition::default(),
            Node {
                width: Val::Percent(100.0),
                max_width: Val::Px(960.0),
                min_width: Val::Px(0.0),
                max_height: Val::Percent(100.0),
                overflow: Overflow::scroll_y(),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(16.0),
                padding: UiRect::all(Val::Px(24.0)),
                ..default()
            },
        ))
        .with_children(|root| {
            // Solid color variants.
            root.spawn(Node {
                display: Display::Flex,
                flex_wrap: FlexWrap::Wrap,
                flex_shrink: 0.0,
                column_gap: Val::Px(12.0),
                row_gap: Val::Px(12.0),
                ..default()
            })
            .with_children(|row| {
                row.spawn(BeverlyButton::primary("Primary"));
                row.spawn(BeverlyButton::secondary("Secondary"));
                row.spawn(BeverlyButton::success("Success"));
                row.spawn(BeverlyButton::danger("Danger"));
                row.spawn(BeverlyButton::warning("Warning"));
                row.spawn(BeverlyButton::info("Info"));
                row.spawn(BeverlyButton::light("Light"));
                row.spawn(BeverlyButton::dark("Dark"));
                row.spawn(BeverlyButton::text_button("Text only"));
            });

            // Outline variants (transparent fill until hovered/pressed).
            root.spawn(Node {
                display: Display::Flex,
                flex_wrap: FlexWrap::Wrap,
                flex_shrink: 0.0,
                column_gap: Val::Px(12.0),
                row_gap: Val::Px(12.0),
                ..default()
            })
            .with_children(|row| {
                row.spawn(BeverlyButton::primary("Primary").outline(true));
                row.spawn(BeverlyButton::secondary("Secondary").outline(true));
                row.spawn(BeverlyButton::success("Success").outline(true));
                row.spawn(BeverlyButton::danger("Danger").outline(true));
                row.spawn(BeverlyButton::warning("Warning").outline(true));
                row.spawn(BeverlyButton::info("Info").outline(true));
                row.spawn(BeverlyButton::light("Light").outline(true));
                row.spawn(BeverlyButton::dark("Dark").outline(true));
            });

            // Disabled buttons: unavailable to pointer/keyboard and announced
            // as disabled to screen readers, not just visually muted.
            root.spawn(Node {
                display: Display::Flex,
                flex_wrap: FlexWrap::Wrap,
                flex_shrink: 0.0,
                column_gap: Val::Px(12.0),
                row_gap: Val::Px(12.0),
                ..default()
            })
            .with_children(|row| {
                row.spawn(BeverlyButton::primary("Disabled").disabled(true));
                row.spawn(
                    BeverlyButton::danger("Disabled outline")
                        .outline(true)
                        .disabled(true),
                );
            });

            // Block button stretches to the width of its parent.
            root.spawn(BeverlyButton::primary("Full-width block button").block(true));
        });
}
