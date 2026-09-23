//! Minimal example demonstrating Beverly buttons and button groups.

use bevy::prelude::*;
use beverly::prelude::*;

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
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(16.0),
            padding: UiRect::all(Val::Px(24.0)),
            ..default()
        })
        .with_children(|root| {
            // Solid color variants.
            root.spawn(Node {
                display: Display::Flex,
                flex_wrap: FlexWrap::Wrap,
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
                row.spawn(BeverlyButton::text("Text only"));
            });

            // Outline variants (transparent fill until hovered/pressed).
            root.spawn(Node {
                display: Display::Flex,
                flex_wrap: FlexWrap::Wrap,
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
                column_gap: Val::Px(12.0),
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
