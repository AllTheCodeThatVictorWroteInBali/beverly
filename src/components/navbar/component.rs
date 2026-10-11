use bevy::prelude::*;

use crate::rendering::{Border, BorderWidths, OuterShadow, Paint, Surface};
use crate::theme::{ThemeMode, ThemeResource};

const NAVBAR_HEIGHT: f32 = 64.0;
const NAVBAR_SIDE_PADDING: f32 = 24.0;
// A fixed navbar overlays page content, so it must stack above it to receive hover and clicks.
const NAVBAR_FIXED_Z_INDEX: i32 = 100;

pub struct NavbarPlugin;

impl Plugin for NavbarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (navbar_fixed_system, navbar_theme_system));
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
    pub z_index: ZIndex,
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
            z_index: navbar_z_index(fixed),

            background: BackgroundColor(Color::NONE),
            surface: Surface::rounded_rect_fill(0.0, Paint::solid(Color::WHITE))
                .border(Border::per_side(
                    BorderWidths::sides(0.0, 0.0, 1.0, 0.0),
                    Paint::solid(Color::srgb(0.84, 0.86, 0.88)),
                ))
                .outer_shadow(OuterShadow::small(Color::BLACK)),
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

fn navbar_z_index(fixed: bool) -> ZIndex {
    ZIndex(if fixed { NAVBAR_FIXED_Z_INDEX } else { 0 })
}

fn navbar_fixed_system(mut query: Query<(&Navbar, &mut Node, &mut ZIndex), Changed<Navbar>>) {
    for (navbar, mut node, mut z_index) in &mut query {
        *z_index = navbar_z_index(navbar.fixed);
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

fn navbar_theme_system(theme: Res<ThemeResource>, mut navbars: Query<&mut Surface, With<Navbar>>) {
    let color = match theme.current.mode {
        ThemeMode::Light => Color::WHITE,
        ThemeMode::Dark => Color::srgb(0.055, 0.055, 0.065),
    };
    let fill = Paint::solid(color);
    let border = Border::per_side(
        BorderWidths::sides(0.0, 0.0, 1.0, 0.0),
        Paint::solid(theme.current.colors.border),
    );

    for mut surface in &mut navbars {
        if surface.fill != fill {
            surface.fill = fill.clone();
        }
        if surface.border.as_ref() != Some(&border) {
            surface.border = Some(border.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{dark_theme, light_theme};

    #[test]
    fn fixed_navbar_stacks_above_page_content() {
        let mut app = App::new();
        app.insert_resource(ThemeResource {
            current: light_theme(),
        })
        .add_plugins(NavbarPlugin);
        let fixed = app.world_mut().spawn(NavbarBundle::new(true)).id();
        let flow = app.world_mut().spawn(NavbarBundle::new(false)).id();
        app.update();

        assert!(app.world().get::<ZIndex>(fixed).unwrap().0 > 0);
        assert_eq!(app.world().get::<ZIndex>(flow).unwrap().0, 0);
    }

    #[test]
    fn navbar_background_tracks_theme_mode() {
        let mut app = App::new();
        app.insert_resource(ThemeResource {
            current: light_theme(),
        })
        .add_plugins(NavbarPlugin);
        let navbar = app.world_mut().spawn(NavbarBundle::new(false)).id();
        app.update();
        assert_eq!(
            app.world().get::<Surface>(navbar).unwrap().fill,
            Paint::solid(Color::WHITE)
        );
        let surface = app.world().get::<Surface>(navbar).unwrap();
        assert_eq!(
            surface.border.as_ref().unwrap().width,
            BorderWidths::sides(0.0, 0.0, 1.0, 0.0)
        );
        assert_eq!(
            surface.effects.outer_shadow,
            Some(OuterShadow::small(Color::BLACK))
        );

        app.world_mut().resource_mut::<ThemeResource>().current = dark_theme();
        app.update();
        assert_eq!(
            app.world().get::<Surface>(navbar).unwrap().fill,
            Paint::solid(Color::srgb(0.055, 0.055, 0.065))
        );
    }
}
