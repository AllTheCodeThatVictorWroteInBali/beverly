//! Sidebar demo: a collapsible snack-desk navigation rail in light/dark mode.
//! Run with `-- dark` to start in dark mode; the top-right button toggles light/dark.

use beverly::components::sidebar::spawn_sidebar;
use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::prelude::*;
use beverly::primitives::root::UiFonts;
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
        .add_systems(PostUpdate, fit_sidebar.before(bevy::ui::UiSystems::Prepare))
        .run_demo();
}

fn fit_sidebar(
    state: Res<beverly::components::sidebar::SidebarState>,
    mut sidebars: Query<&mut Node, With<beverly::components::sidebar::Sidebar>>,
    mut buttons: Query<
        &mut Node,
        (
            With<beverly::components::nav_button::DrawerButton>,
            Without<beverly::components::sidebar::Sidebar>,
        ),
    >,
) {
    for mut node in &mut sidebars {
        node.min_width = px(if state.open { 220.0 } else { 58.0 });
        node.max_width = percent(100);
        node.height = px(520);
        node.flex_shrink = 0.0;
        node.overflow = Overflow::scroll_y();
    }
    for mut node in &mut buttons {
        node.flex_shrink = 0.0;
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let ui_fonts = UiFonts {
        text: asset_server.load("embedded://beverly/components/text/fonts/DefaultSans.ttf"),
    };

    spawn_themed_page(&mut commands, move |root| {
        root.spawn(Node {
            width: percent(100),
            min_height: px(520),
            min_width: px(0),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            align_items: AlignItems::Stretch,
            ..default()
        })
        .with_children(|shell| {
            let sidebar = spawn_sidebar(shell, &ui_fonts, 0.0);
            shell
                .commands()
                .entity(sidebar)
                .insert((bevy::ui_widgets::ScrollArea, ScrollPosition::default()));
            shell
                .spawn(Node {
                    flex_basis: px(240),
                    flex_grow: 1.0,
                    min_width: px(0),
                    max_width: percent(100),
                    min_height: px(240),
                    flex_shrink: 0.0,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(12.0),
                    padding: UiRect::all(Val::Px(28.0)),
                    ..default()
                })
                .with_children(|content| {
                    content.spawn((ThemedTitle::new(TitleLevel::H2), Text::new("Snack Desk HQ")));
                    content.spawn((
                        ThemedText::new(TextRole::Body),
                        Text::new("Choose a department. The fern is interim manager."),
                    ));
                });
        });
    });
}
