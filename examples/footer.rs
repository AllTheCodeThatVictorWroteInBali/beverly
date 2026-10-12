//! Footer demo: an unstyled footer pinned to the bottom of the page.
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
    let page = spawn_themed_page(&mut commands, |root| {
        root.spawn((
            ThemedText::new(TextRole::Body),
            Text::new("Page content goes here."),
        ));
    });

    // Fixed footers pin themselves to the bottom of their parent.
    let (footer, sections) =
        spawn_footer_with_sections(&mut commands, true, FooterConfig::default());
    commands.entity(footer).queue(|mut entity: EntityWorldMut| {
        if let Some(mut node) = entity.get_mut::<Node>() {
            node.min_width = Val::Px(0.0);
            node.height = Val::Auto;
            node.min_height = Val::Px(52.0);
            node.grid_template_columns =
                vec![GridTrack::px(0.0), GridTrack::fr(1.0), GridTrack::px(0.0)];
            node.padding = UiRect::all(Val::Px(16.0));
        }
    });
    commands
        .entity(sections.center)
        .queue(|mut entity: EntityWorldMut| {
            if let Some(mut node) = entity.get_mut::<Node>() {
                node.width = Val::Percent(100.0);
                node.min_width = Val::Px(0.0);
                node.height = Val::Auto;
            }
        });
    add_to_center(
        &mut commands,
        &sections,
        (
            Node {
                min_width: Val::Px(0.0),
                ..default()
            },
            ThemedText::new(TextRole::Muted),
            Text::new("You've reached the bottom. Please enjoy this complimentary footer."),
        ),
    );
    commands.entity(page).add_child(footer);
}
