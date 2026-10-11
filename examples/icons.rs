//! Icon gallery: every embedded Feather icon, including the light/dark circle.
//! Run with `-- dark` to start in dark mode.

use beverly::components::button::{BeverlyButton, ButtonChild};
use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::icons::IconCommands;
use beverly::prelude::*;
use bevy::prelude::*;

// Read the icon names from the embed list so the gallery tracks the real icon set.
const EMBEDDED_ICONS: &str = include_str!("../src/icons/embedded.rs");
const COLUMNS: usize = 9;
const PER_PAGE: usize = COLUMNS * 4;

#[derive(Resource, Default)]
struct GalleryPage(usize);

#[derive(Component)]
struct IconGrid;

#[derive(Component)]
struct PageLabel;

fn icon_names() -> Vec<&'static str> {
    EMBEDDED_ICONS
        .lines()
        .filter_map(|line| {
            let start = line.find("\"feather/")? + "\"feather/".len();
            let end = line.find(".svg\"")?;
            Some(&line[start..end])
        })
        .collect()
}

fn page_count() -> usize {
    icon_names().len().div_ceil(PER_PAGE)
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .init_resource::<GalleryPage>()
        .add_systems(Startup, setup)
        .add_systems(Update, rebuild_grid)
        .run();
}

fn setup(mut commands: Commands) {
    spawn_themed_page(&mut commands, |root| {
        root.spawn(Node {
            width: Val::Px(1000.0),
            max_width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(16.0),
            ..default()
        })
        .with_children(|content| {
            content.spawn((ThemedTitle::new(TitleLevel::H2), Text::new("Icon Gallery")));
            content.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new(format!(
                    "All {} Feather icons, including the light/dark circle.",
                    icon_names().len()
                )),
            ));
            content.spawn((
                IconGrid,
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(8.0),
                    row_gap: Val::Px(8.0),
                    ..default()
                },
            ));
            content
                .spawn(Node {
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(14.0),
                    ..default()
                })
                .with_children(|pager| {
                    pager.spawn(
                        BeverlyButton::standard("Previous")
                            .children([
                                ButtonChild::icon("arrow-left"),
                                ButtonChild::text("Previous"),
                            ])
                            .on("click", |commands, _| step_page(commands, -1)),
                    );
                    pager.spawn((PageLabel, ThemedText::new(TextRole::Body), Text::new("")));
                    pager.spawn(
                        BeverlyButton::standard("Next")
                            .children([ButtonChild::text("Next"), ButtonChild::icon("arrow-right")])
                            .on("click", |commands, _| step_page(commands, 1)),
                    );
                });
        });
    });
}

fn step_page(commands: &mut Commands, delta: isize) {
    commands.queue(move |world: &mut World| {
        let pages = page_count() as isize;
        let mut page = world.resource_mut::<GalleryPage>();
        page.0 = (page.0 as isize + delta).rem_euclid(pages) as usize;
    });
}

fn rebuild_grid(
    mut commands: Commands,
    page: Res<GalleryPage>,
    grids: Query<Entity, With<IconGrid>>,
    mut labels: Query<&mut Text, With<PageLabel>>,
) {
    if !page.is_changed() {
        return;
    }
    let names = icon_names();
    let start = page.0 * PER_PAGE;
    let visible = &names[start.min(names.len())..(start + PER_PAGE).min(names.len())];
    for grid in &grids {
        commands
            .entity(grid)
            .despawn_related::<Children>()
            .with_children(|grid| {
                for name in visible {
                    grid.spawn(Node {
                        width: Val::Px(104.0),
                        height: Val::Px(84.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        row_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|cell| {
                        cell.spawn_feather_sized(*name, 28.0);
                        cell.spawn((
                            ThemedText::new(TextRole::Caption).size(11.0),
                            Text::new(*name),
                            TextLayout::linebreak(LineBreak::WordOrCharacter),
                        ));
                    });
                }
            });
    }
    for mut label in &mut labels {
        *label = Text::new(format!(
            "Page {} of {}  -  {} icons",
            page.0 + 1,
            page_count(),
            names.len()
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gallery_lists_every_embedded_icon_once_and_includes_the_theme_circle() {
        let names = icon_names();
        let files = std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/src/icons/feather"))
            .unwrap()
            .filter(|entry| {
                entry
                    .as_ref()
                    .unwrap()
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == "svg")
            })
            .count();
        assert_eq!(names.len(), files);
        let mut unique = names.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), names.len());
        assert!(names.contains(&"theme-toggle"));
        assert!(page_count() * PER_PAGE >= names.len());
        assert!(names.len() > (page_count() - 1) * PER_PAGE);
    }

    #[test]
    fn paging_wraps_in_both_directions() {
        let mut world = World::new();
        world.init_resource::<GalleryPage>();
        step_page(&mut world.commands(), -1);
        world.flush();
        assert_eq!(world.resource::<GalleryPage>().0, page_count() - 1);
        step_page(&mut world.commands(), 1);
        world.flush();
        assert_eq!(world.resource::<GalleryPage>().0, 0);
    }
}
