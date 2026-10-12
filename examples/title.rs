//! Title demo: the snack desk's official hierarchy of importance.
//! Run with `-- dark` to start in dark mode.

use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
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
        root.spawn(Node {
            width: Val::Percent(100.0),
            max_width: Val::Px(640.0),
            min_width: Val::Px(0.0),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        })
        .with_children(|content| {
            content.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("Official hierarchy of snack-related importance"),
            ));
            for (level, label, title) in [
                (TitleLevel::Display, "Display", "Department of Toast"),
                (TitleLevel::H1, "H1", "The Biscuit Bureau"),
                (TitleLevel::H2, "H2", "Kettle Committee"),
                (TitleLevel::H3, "H3", "Spread Operations"),
                (TitleLevel::H4, "H4", "Crumb Investigation Unit"),
                (TitleLevel::H5, "H5", "Napkin Administration"),
                (TitleLevel::H6, "H6", "Unofficial Second Breakfast"),
            ] {
                content
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        min_width: Val::Px(0.0),
                        flex_shrink: 0.0,
                        flex_wrap: FlexWrap::Wrap,
                        align_items: AlignItems::Baseline,
                        column_gap: Val::Px(20.0),
                        row_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            ThemedText::new(TextRole::Caption),
                            Text::new(label),
                            Node {
                                width: Val::Px(64.0),
                                flex_shrink: 0.0,
                                ..default()
                            },
                        ));
                        row.spawn((
                            ThemedTitle::new(level),
                            Text::new(title),
                            TextLayout::linebreak(LineBreak::WordOrCharacter),
                            Node {
                                flex_basis: Val::Px(240.0),
                                flex_grow: 1.0,
                                min_width: Val::Px(0.0),
                                ..default()
                            },
                        ));
                    });
            }
            content.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("All departments report directly to the biscuit tin."),
            ));
        });
    });
}
