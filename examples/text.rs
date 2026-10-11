//! Text demo: a very official snack desk memo.
//! Run with `-- dark` to start in dark mode.

use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::prelude::*;
use bevy::prelude::*;

const SAMPLES: [(TextRole, &str, &str); 7] = [
    (
        TextRole::Heading,
        "Heading",
        "A matter of biscuit importance",
    ),
    (
        TextRole::Body,
        "Body",
        "The emergency tin is now open for business.",
    ),
    (TextRole::Label, "Label", "Approved by the Kettle Committee"),
    (
        TextRole::Caption,
        "Caption",
        "Filed at 10:42, shortly before second breakfast.",
    ),
    (
        TextRole::Muted,
        "Muted",
        "The crumb inspector has declined to comment.",
    ),
    (
        TextRole::Accent,
        "Accent",
        "Good news: another batch of toast has arrived.",
    ),
    (
        TextRole::Disabled,
        "Disabled",
        "Jam requisitions are temporarily unavailable.",
    ),
];

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    spawn_themed_page(&mut commands, |root| {
        root.spawn(Node {
            width: Val::Px(680.0),
            max_width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        }).with_children(|content| {
            content.spawn((ThemedTitle::new(TitleLevel::H2), Text::new("Snack Desk Memo")));
            content.spawn((ThemedText::new(TextRole::Muted), Text::new("Internal correspondence. External crumbs.")));
            for (role, label, sample) in SAMPLES {
                content.spawn(Node {
                    width: Val::Percent(100.0),
                    align_items: AlignItems::Baseline,
                    column_gap: Val::Px(20.0),
                    ..default()
                }).with_children(|row| {
                    row.spawn((
                        ThemedText::new(TextRole::Caption),
                        Text::new(label),
                        Node { width: Val::Px(80.0), flex_shrink: 0.0, ..default() },
                    ));
                    row.spawn((
                        ThemedText::new(role),
                        Text::new(sample),
                        Node { flex_grow: 1.0, min_width: Val::Px(0.0), ..default() },
                    ));
                });
            }
            content.spawn((
                ThemedText::new(TextRole::Body),
                Text::new("Following a thorough investigation, the suspicious crumb has been identified as a perfectly ordinary crumb. The committee recommends tea, a brief pause, and absolutely no further paperwork. Please keep the emergency biscuits within a reasonable reaching distance."),
                Node { width: Val::Percent(100.0), margin: UiRect::top(Val::Px(8.0)), ..default() },
            ));
            content.spawn((ThemedText::new(TextRole::Body).size(24.0), Text::new("Less paperwork. More toast.")));
        });
    });
}
