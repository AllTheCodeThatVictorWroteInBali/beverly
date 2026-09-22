use bevy::prelude::*;

use crate::rendering::{GradientStop, LinearGradient, Paint, Surface};

const NAVBAR_HEIGHT: f32 = 64.0;
const NAVBAR_SIDE_PADDING: f32 = 24.0;

pub struct NavbarPlugin;

impl Plugin for NavbarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, navbar_fixed_system);
    }
}

#[derive(Component)]
pub struct Navbar {
    pub fixed: bool,
}

impl Navbar {
    pub fn new() -> Self {
        Self { fixed: false }
    }

    pub fn fixed(mut self, fixed: bool) -> Self {
        self.fixed = fixed;
        self
    }
}

#[derive(Component)]
pub struct NavbarLeft;

#[derive(Component)]
pub struct NavbarCenter;

#[derive(Component)]
pub struct NavbarRight;

#[derive(Component)]
pub struct NavbarBottomDivider;

#[derive(Component, Clone, Copy)]
pub struct NavbarSections {
    pub left: Entity,
    pub center: Entity,
    pub right: Entity,
}

#[derive(Bundle)]
pub struct NavbarBundle {
    pub navbar: Navbar,
    pub node: Node,
    pub background: BackgroundColor,
    pub surface: Surface,
}

impl NavbarBundle {
    pub fn new(fixed: bool) -> Self {
        Self {
            navbar: Navbar { fixed },
            node: Node {
                width: Val::Percent(100.0),
                height: Val::Px(NAVBAR_HEIGHT),

                display: Display::Grid,

                // Three equal columns.
                grid_template_columns: vec![
                    GridTrack::fr(1.0),
                    GridTrack::fr(1.0),
                    GridTrack::fr(1.0),
                ],

                align_items: AlignItems::Center,

                ..default()
            },

            background: BackgroundColor(Color::NONE),
            surface: Surface::rounded_rect_fill(
                0.0,
                Paint::linear(LinearGradient::vertical(vec![
                    GradientStop::new(0.0, Color::srgb(0.05, 0.05, 0.05)),
                    GradientStop::new(1.0, Color::srgb(0.07, 0.07, 0.09)),
                ])),
            ),
        }
    }
}

pub fn spawn_navbar(commands: &mut Commands, fixed: bool) -> Entity {
    let mut sections = None;

    let navbar = commands
        .spawn(NavbarBundle::new(fixed))
        .with_children(|parent| {
            parent.spawn((
                NavbarBottomDivider,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    height: Val::Px(1.0),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ));

            let left = parent
                .spawn((
                    NavbarLeft,
                    navbar_section(1, JustifyContent::FlexStart, true),
                ))
                .id();
            let center = parent
                .spawn((
                    NavbarCenter,
                    navbar_section(2, JustifyContent::FlexStart, true),
                ))
                .id();
            let right = parent
                .spawn((
                    NavbarRight,
                    navbar_section(3, JustifyContent::FlexEnd, true),
                ))
                .id();

            sections = Some((left, center, right));
        })
        .id();

    let (left, center, right) = sections.expect("navbar sections must be created");

    commands.entity(navbar).insert(NavbarSections {
        left,
        center,
        right,
    });

    navbar
}

pub fn add_to_left(
    commands: &mut Commands,
    sections: &NavbarSections,
    bundle: impl Bundle,
) -> Entity {
    add_to_section(commands, sections.left, bundle)
}

pub fn add_to_center(
    commands: &mut Commands,
    sections: &NavbarSections,
    bundle: impl Bundle,
) -> Entity {
    add_to_section(commands, sections.center, bundle)
}

pub fn add_to_right(
    commands: &mut Commands,
    sections: &NavbarSections,
    bundle: impl Bundle,
) -> Entity {
    add_to_section(commands, sections.right, bundle)
}

fn add_to_section(commands: &mut Commands, section: Entity, bundle: impl Bundle) -> Entity {
    let mut child = None;
    commands.entity(section).with_children(|parent| {
        child = Some(parent.spawn(bundle).id());
    });
    child.expect("navbar section child must be created")
}

fn navbar_section(column: i16, justify_content: JustifyContent, padded: bool) -> Node {
    Node {
        grid_column: GridPlacement::start(column),
        height: Val::Percent(100.0),
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        justify_content,
        padding: if padded {
            UiRect::horizontal(Val::Px(NAVBAR_SIDE_PADDING))
        } else {
            UiRect::default()
        },
        ..default()
    }
}

fn navbar_fixed_system(mut query: Query<(&Navbar, &mut Node), Changed<Navbar>>) {
    for (navbar, mut node) in &mut query {
        if navbar.fixed {
            node.position_type = PositionType::Absolute;
            node.top = Val::Px(0.0);
            node.left = Val::Px(0.0);
            node.right = Val::Px(0.0);
        } else {
            node.position_type = PositionType::Relative;
            node.top = Val::Auto;
            node.left = Val::Auto;
            node.right = Val::Auto;
        }
    }
}
