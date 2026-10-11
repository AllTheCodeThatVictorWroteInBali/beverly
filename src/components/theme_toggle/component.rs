use bevy::prelude::*;

use crate::components::button::{BeverlyButton, ButtonChild, ButtonSize};
use crate::theme::{Theme, ThemeMode, ThemeResource, dark_theme, light_theme};

/// Full-window page whose background color follows the active theme.
#[derive(Component)]
pub struct ThemedPage;

/// Keeps `ThemedPage` backgrounds and the window clear color in sync with the theme.
pub struct ThemeTogglePlugin;

impl Plugin for ThemeTogglePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sync_page_background);
    }
}

fn sync_page_background(
    theme: Res<ThemeResource>,
    clear: Option<ResMut<ClearColor>>,
    mut pages: Query<(Ref<ThemedPage>, &mut BackgroundColor)>,
) {
    let background = theme.current.colors.background;
    let theme_changed = theme.is_changed();

    if theme_changed {
        if let Some(mut clear) = clear {
            clear.0 = background;
        }
    }

    for (page, mut color) in &mut pages {
        if theme_changed || page.is_added() {
            color.0 = background;
        }
    }
}

/// Theme to start with: dark when the process was started with a `dark` argument.
pub fn theme_from_cli_args() -> Theme {
    if std::env::args().any(|arg| arg == "dark") {
        dark_theme()
    } else {
        light_theme()
    }
}

/// Flips between the light and dark theme.
pub fn toggle_theme(commands: &mut Commands, _button: Entity) {
    commands.queue(|world: &mut World| {
        let mut theme = world.resource_mut::<ThemeResource>();
        theme.current = match theme.current.mode {
            ThemeMode::Light => dark_theme(),
            ThemeMode::Dark => light_theme(),
        };
    });
}

/// Spawns a light/dark toggle button pinned to the top-right corner of `parent`.
pub fn spawn_theme_toggle(parent: &mut ChildSpawnerCommands) -> Entity {
    // The button replaces its own `Node`, so positioning goes on a wrapper.
    parent
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            right: Val::Px(16.0),
            ..default()
        })
        .with_children(|corner| {
            corner.spawn(
                BeverlyButton::standard("")
                    .sized(ButtonSize::Sm)
                    .children([ButtonChild::icon("theme-toggle")])
                    .on("click", toggle_theme),
            );
        })
        .id()
}

/// Spawns a full-window, themed page with a light/dark toggle in the corner.
/// `content` is laid out as a centered column.
pub fn spawn_themed_page(
    commands: &mut Commands,
    content: impl FnOnce(&mut ChildSpawnerCommands),
) -> Entity {
    commands
        .spawn((
            ThemedPage,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(32.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|page| {
            spawn_theme_toggle(page);
            content(page);
        })
        .id()
}
