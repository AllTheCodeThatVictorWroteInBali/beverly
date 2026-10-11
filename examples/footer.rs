//! Footer demo: an unstyled footer pinned to the bottom of the page.
//! Run with `-- dark` to start in dark mode; the top-right button toggles light/dark.

use beverly::prelude::*;
use bevy::prelude::*;

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
    let page = spawn_themed_page(&mut commands, |root| {
        root.spawn((
            ThemedText::new(TextRole::Body),
            Text::new("Page content goes here."),
        ));
    });

    // Fixed footers pin themselves to the bottom of their parent.
    let (footer, sections) =
        spawn_footer_with_sections(&mut commands, true, FooterConfig::default());
    add_to_center(
        &mut commands,
        &sections,
        (
            ThemedText::new(TextRole::Muted),
            Text::new("You've reached the bottom. Please enjoy this complimentary footer."),
        ),
    );
    commands.entity(page).add_child(footer);
}
