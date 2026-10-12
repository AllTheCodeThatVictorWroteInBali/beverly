//! Navbar demo: a cozy brand, centered destinations, and a fern status button.
//! Run with `-- dark` to start in dark mode; the top-right button toggles light/dark.

use beverly::components::button::BeverlyButton;
use beverly::components::navbar::{
    NavbarSections, add_to_center, add_to_left, add_to_right, spawn_navbar,
};
use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::prelude::*;
use beverly::theme::ThemeResource;
use bevy::prelude::*;

event! {
    NavbarTheme::Toggle
}

#[derive(Component)]
struct ThemeToggleButton;

#[derive(Component)]
struct PageContent;

struct NavbarThemeController;

impl Plugin for NavbarThemeController {
    fn build(&self, app: &mut App) {
        app.add_message::<NavbarTheme::Toggle>()
            .add_systems(Update, toggle_theme);
    }
}

#[path = "support/mod.rs"]
mod demo_support;
use demo_support::DemoAppExt;

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .add_plugins(NavbarThemeController)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, populate_navbar)
        .add_systems(Update, inset_page_content)
        .run_demo();
}

fn setup(mut commands: Commands) {
    let page = commands
        .spawn((
            bevy::ui_widgets::ScrollArea,
            ScrollPosition::default(),
            Node {
                width: percent(100),
                height: percent(100),
                min_width: px(0),
                overflow: Overflow::scroll_y(),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .id();

    let navbar = spawn_navbar(&mut commands, true);
    commands.entity(navbar).insert((
        ChildOf(page),
        Node {
            width: percent(100),
            min_width: px(0),
            min_height: px(64),
            flex_wrap: FlexWrap::Wrap,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::all(px(16)),
            column_gap: px(12),
            row_gap: px(8),
            ..default()
        },
    ));

    commands.entity(page).with_children(|root| {
        root.spawn((
            PageContent,
            Node {
                width: percent(100),
                min_width: px(0),
                flex_grow: 1.0,
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(12.0),
                padding: UiRect::all(px(16)),
                ..default()
            },
        ))
        .with_children(|content| {
            content.spawn((
                ThemedTitle::new(TitleLevel::H1),
                Text::new("The fern is thriving."),
            ));
            content.spawn((
                ThemedText::new(TextRole::Body),
                Text::new("The snack budget is under careful crumb supervision."),
            ));
        });
    });
}

fn populate_navbar(
    mut commands: Commands,
    sections: Query<&NavbarSections, Added<NavbarSections>>,
) {
    for sections in &sections {
        for section in [sections.left, sections.center, sections.right] {
            commands.entity(section).insert(Node {
                max_width: percent(100),
                min_width: px(0),
                flex_wrap: FlexWrap::Wrap,
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                column_gap: px(8),
                row_gap: px(8),
                ..default()
            });
        }
        add_to_left(
            &mut commands,
            sections,
            BeverlyButton::text_button("Moss & Mugs")
                .icon("coffee")
                .on("click", brand_clicked),
        );
        add_to_center(
            &mut commands,
            sections,
            BeverlyButton::text_button("Overview").on("click", nav_clicked),
        );
        add_to_center(
            &mut commands,
            sections,
            BeverlyButton::text_button("Snack ledger").on("click", nav_clicked),
        );
        add_to_center(
            &mut commands,
            sections,
            BeverlyButton::text_button("Plant status").on("click", nav_clicked),
        );
        add_to_right(
            &mut commands,
            sections,
            (
                BeverlyButton::success("Toggle light/dark")
                    .icon("sun")
                    .on("click", publish_theme_toggle),
                ThemeToggleButton,
            ),
        );
    }
}

fn inset_page_content(
    navbars: Query<&ComputedNode, With<beverly::components::navbar::Navbar>>,
    mut content: Query<&mut Node, With<PageContent>>,
) {
    for navbar in &navbars {
        for mut node in &mut content {
            node.padding.top = px(navbar.size().y * navbar.inverse_scale_factor() + 16.0);
        }
    }
}

fn toggle_theme(mut events: MessageReader<NavbarTheme::Toggle>, mut theme: ResMut<ThemeResource>) {
    for _event in events.read() {
        theme.current = theme.current.toggle_mode();
    }
}

fn publish_theme_toggle(commands: &mut Commands, _button: Entity) {
    commands.write_message(NavbarTheme::Toggle);
}

fn brand_clicked(_commands: &mut Commands, _button: Entity) {
    info!("Moss & Mugs welcomes you. The fern has reserved a seat.");
}

fn nav_clicked(_commands: &mut Commands, _button: Entity) {
    info!("Navigation clicked. A tiny clipboard has been notified.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use beverly::primitives::interaction::{
        InteractionAction, InteractionActionEvent, InteractionActionSource,
    };
    use beverly::theme::light_theme;

    #[test]
    fn theme_controller_calls_the_theme_model_toggle_method() {
        let mut app = App::new();
        app.insert_resource(ThemeResource {
            current: light_theme(),
        })
        .add_plugins(NavbarThemeController);

        app.world_mut().write_message(NavbarTheme::Toggle);
        app.update();
        assert_eq!(
            app.world().resource::<ThemeResource>().current.mode,
            ThemeMode::Dark
        );

        app.world_mut().write_message(NavbarTheme::Toggle);
        app.update();
        assert_eq!(
            app.world().resource::<ThemeResource>().current.mode,
            ThemeMode::Light
        );
    }

    #[test]
    fn click_event_from_green_button_reaches_the_theme_controller() {
        let mut app = App::new();
        app.insert_resource(ThemeResource {
            current: light_theme(),
        })
        .add_message::<InteractionActionEvent>()
        .add_plugins((
            beverly::components::button::ButtonPlugin,
            NavbarThemeController,
        ));
        let button = app
            .world_mut()
            .spawn((
                BeverlyButton::success("Toggle light/dark").on("click", publish_theme_toggle),
                ThemeToggleButton,
            ))
            .id();
        app.update();
        app.world_mut().write_message(InteractionActionEvent {
            action: InteractionAction::Activate,
            target: button,
            pointer_id: None,
            source: InteractionActionSource::Pointer,
            consumed: false,
        });
        app.update();
        app.update();
        assert_eq!(
            app.world().resource::<ThemeResource>().current.mode,
            beverly::theme::ThemeMode::Dark
        );
    }

    #[test]
    fn navbar_populates_a_green_theme_toggle_after_sections_are_spawned() {
        let mut app = App::new();
        app.add_systems(Startup, setup)
            .add_systems(Update, populate_navbar);

        app.update();

        let mut buttons = app
            .world_mut()
            .query_filtered::<&BeverlyButton, With<ThemeToggleButton>>();
        let toggle = buttons.single(app.world()).unwrap();
        assert_eq!(
            toggle.color,
            beverly::components::button::ButtonColor::Success
        );
        assert_eq!(toggle.label, "Toggle light/dark");
    }
}
