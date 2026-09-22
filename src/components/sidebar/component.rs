use bevy::prelude::*;

use crate::rendering::{GradientStop, LinearGradient, Paint, Surface};
use crate::components::nav_button::{
    DrawerButton, DrawerLabel, DrawerToggle, DrawerToggleLabel, PageId, drawer_toggle, nav_button,
};
use crate::theme::ThemeResource;
use crate::primitives::root::UiFonts;

const SIDEBAR_OPEN_WIDTH: f32 = 30.0;
const SIDEBAR_COLLAPSED_WIDTH: f32 = 6.0;
const SIDEBAR_ANIMATION_SPEED: f32 = 12.0;
const SIDEBAR_OPEN_PADDING: f32 = 20.0;
const SIDEBAR_COLLAPSED_PADDING: f32 = 8.0;
const SIDEBAR_BUTTON_OPEN_HEIGHT: f32 = 58.0;
const SIDEBAR_BUTTON_COLLAPSED_SIZE: f32 = 42.0;
const SIDEBAR_BUTTON_OPEN_PADDING_X: f32 = 18.0;
const SIDEBAR_LABEL_VISIBILITY_THRESHOLD: f32 = 16.0;

#[derive(Component)]
pub struct Sidebar;

#[derive(Component)]
pub struct SidebarDivider;

#[derive(Resource)]
pub struct SidebarState {
    pub open: bool,
}

impl Default for SidebarState {
    fn default() -> Self {
        Self { open: true }
    }
}

pub struct SidebarPlugin;

impl Plugin for SidebarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SidebarState>().add_systems(
            Update,
            (
                handle_sidebar_toggle_click,
                animate_sidebar,
                sidebar_theme_system,
            ),
        );
    }
}

pub fn spawn_sidebar(
    parent: &mut ChildSpawnerCommands,
    ui_fonts: &UiFonts,
    top_offset_px: f32,
) -> Entity {
    parent
        .spawn((
            Name::new("app-sidebar"),
            Sidebar,
            Node {
                width: percent(SIDEBAR_OPEN_WIDTH),
                height: percent(100),
                margin: UiRect::top(px(top_offset_px)),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(SIDEBAR_OPEN_PADDING)),
                row_gap: px(10),
                align_items: AlignItems::Stretch,
                overflow: Overflow::clip(),
                ..default()
            },
            Surface::rounded_rect_fill(
                0.0,
                Paint::linear(LinearGradient::vertical(vec![
                    GradientStop::new(0.0, Color::srgb(0.09, 0.10, 0.13)),
                    GradientStop::new(1.0, Color::srgb(0.12, 0.13, 0.17)),
                ])),
            ),
        ))
        .with_children(|nav| {
            nav.spawn((
                SidebarDivider,
                Node {
                    position_type: PositionType::Absolute,
                    top: px(0.0),
                    right: px(0.0),
                    bottom: px(0.0),
                    width: px(1.0),
                    ..default()
                },
                Surface::rounded_rect_fill(0.0, Paint::solid(Color::NONE)),
            ));

            drawer_toggle(nav, ui_fonts);
            for page in PageId::all() {
                nav_button(nav, page, ui_fonts);
            }
        })
        .id()
}

fn handle_sidebar_toggle_click(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<DrawerToggle>)>,
    mut sidebar_state: ResMut<SidebarState>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            sidebar_state.open = !sidebar_state.open;
        }
    }
}

fn animate_sidebar(
    time: Res<Time>,
    sidebar_state: Res<SidebarState>,
    mut sidebar_query: Query<&mut Node, With<Sidebar>>,
    mut drawer_button_query: Query<
        &mut Node,
        (With<DrawerButton>, Without<Sidebar>, Without<DrawerLabel>),
    >,
    mut label_query: Query<&mut Node, (With<DrawerLabel>, Without<Sidebar>, Without<DrawerButton>)>,
) {
    let target = if sidebar_state.open {
        SIDEBAR_OPEN_WIDTH
    } else {
        SIDEBAR_COLLAPSED_WIDTH
    };

    let mut labels_visible = None;

    for mut sidebar in &mut sidebar_query {
        let current = match sidebar.width {
            Val::Percent(value) => value,
            _ => target,
        };

        let new_width = current
            + (target - current) * (1.0 - (-SIDEBAR_ANIMATION_SPEED * time.delta_secs()).exp());

        sidebar.width = Val::Percent(new_width);

        if new_width < SIDEBAR_LABEL_VISIBILITY_THRESHOLD {
            sidebar.align_items = AlignItems::Center;
            sidebar.padding = UiRect::all(px(SIDEBAR_COLLAPSED_PADDING));
            sidebar.row_gap = px(8);
        } else {
            sidebar.align_items = AlignItems::Stretch;
            sidebar.padding = UiRect::all(px(SIDEBAR_OPEN_PADDING));
            sidebar.row_gap = px(10);
        }

        labels_visible = Some(new_width > SIDEBAR_LABEL_VISIBILITY_THRESHOLD);
    }

    let Some(labels_visible) = labels_visible else {
        return;
    };

    for mut button_node in &mut drawer_button_query {
        if labels_visible {
            button_node.width = Val::Percent(100.0);
            button_node.height = Val::Px(SIDEBAR_BUTTON_OPEN_HEIGHT);
            button_node.justify_content = JustifyContent::FlexStart;
            button_node.align_self = AlignSelf::Auto;
            button_node.padding = UiRect::horizontal(px(SIDEBAR_BUTTON_OPEN_PADDING_X));
        } else {
            button_node.width = Val::Px(SIDEBAR_BUTTON_COLLAPSED_SIZE);
            button_node.height = Val::Px(SIDEBAR_BUTTON_COLLAPSED_SIZE);
            button_node.justify_content = JustifyContent::Center;
            button_node.align_self = AlignSelf::Center;
            button_node.padding = UiRect::all(px(0.0));
        }
    }

    for mut label_node in &mut label_query {
        label_node.display = if labels_visible {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn sidebar_theme_system(
    theme: Res<ThemeResource>,
    mut background_queries: ParamSet<(
        Query<&mut Surface, (With<Sidebar>, Without<DrawerToggle>)>,
        Query<&mut Surface, With<SidebarDivider>>,
        Query<
            (&Interaction, &mut Surface),
            (
                With<DrawerToggle>,
                With<Button>,
                Without<Sidebar>,
                Without<DrawerButton>,
            ),
        >,
    )>,
    mut toggle_label_query: Query<&mut TextColor, (With<DrawerLabel>, With<DrawerToggleLabel>)>,
) {
    let colors = theme.current.colors;

    for mut background in &mut background_queries.p0() {
        background.fill = Paint::solid(colors.surface);
    }

    for mut background in &mut background_queries.p1() {
        background.fill = Paint::solid(colors.border);
    }

    for (interaction, mut background) in &mut background_queries.p2() {
        background.fill = Paint::solid(match *interaction {
            Interaction::Pressed => colors.surface_selected,
            Interaction::Hovered => colors.surface_hover,
            Interaction::None => Color::NONE,
        });
        if let Some(border) = background.border.as_mut() {
            border.paint = Paint::solid(match *interaction {
                Interaction::Pressed => colors.border_strong.with_alpha(0.55),
                Interaction::Hovered | Interaction::None => Color::NONE,
            });
        }
    }

    for mut color in &mut toggle_label_query {
        color.0 = colors.text;
    }
}
