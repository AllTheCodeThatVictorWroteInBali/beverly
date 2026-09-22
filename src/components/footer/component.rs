use bevy::prelude::*;

use crate::rendering::{GradientStop, LinearGradient, Paint, Surface};

const FOOTER_HEIGHT: f32 = 52.0;
const FOOTER_SIDE_PADDING: f32 = 16.0;

/// Marker component for the footer root.
#[derive(Component)]
pub struct Footer {
    pub fixed: bool,
}

impl Footer {
    pub fn new() -> Self {
        Self { fixed: false }
    }

    pub fn fixed(mut self, fixed: bool) -> Self {
        self.fixed = fixed;
        self
    }
}

/// Thin top divider line, rendered as a plain background-color node rather
/// than a native `Node::border` (which is drawn by the SDF shape shader and
/// anti-aliases a 1px line into near-invisibility at HiDPI scale factors).
#[derive(Component)]
pub struct FooterTopDivider;

#[derive(Component)]
pub struct FooterLeft;

#[derive(Component)]
pub struct FooterCenter;

#[derive(Component)]
pub struct FooterRight;

#[derive(Component, Clone, Copy)]
pub struct FooterSections {
    pub left: Entity,
    pub center: Entity,
    pub right: Entity,
}

/// Footer section.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum FooterSection {
    Left,
    Center,
    Right,
}

/// Configuration for a footer.
#[derive(Clone, Debug)]
pub struct FooterConfig {
    /// Height of the footer.
    pub height: Val,

    /// Whether the footer should have a border at the top.
    pub show_border: bool,

    /// Padding inside the footer.
    pub padding: UiRect,

    /// Gap between items in each section.
    pub gap: Val,

    /// Background color of the footer.
    pub background: Color,

    /// Top border color when border is enabled.
    pub border_color: Color,
}

impl Default for FooterConfig {
    fn default() -> Self {
        Self {
            height: Val::Px(FOOTER_HEIGHT),
            show_border: true,
            padding: UiRect::default(),
            gap: Val::Px(12.0),
            background: Color::srgb(0.05, 0.05, 0.05),
            border_color: Color::srgb(0.20, 0.20, 0.20),
        }
    }
}

#[derive(Bundle)]
pub struct FooterBundle {
    pub footer: Footer,
    pub node: Node,
    pub background: BackgroundColor,
    pub surface: Surface,
}

impl FooterBundle {
    pub fn new(fixed: bool, config: FooterConfig) -> Self {
        Self {
            footer: Footer { fixed },
            node: Node {
                width: Val::Percent(100.0),
                max_width: Val::Percent(100.0),
                min_width: Val::Percent(100.0),
                height: config.height,
                min_height: config.height,
                display: Display::Grid,
                grid_template_columns: vec![
                    GridTrack::px(184.0),
                    GridTrack::fr(1.0),
                    GridTrack::px(184.0),
                ],
                align_items: AlignItems::Center,
                padding: config.padding,
                margin: UiRect::ZERO,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                ..default()
            },
            background: BackgroundColor(Color::NONE),
            surface: Surface::rounded_rect_fill(
                0.0,
                Paint::linear(LinearGradient::vertical(vec![
                    GradientStop::new(0.0, config.background),
                    GradientStop::new(1.0, config.background.with_alpha(0.96)),
                ])),
            ),
        }
    }
}

/// Plugin for the footer component.
pub struct FooterPlugin;

impl Plugin for FooterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, footer_fixed_system);
    }
}

/// Spawns a footer with 3 sections.
pub fn spawn_footer(commands: &mut Commands, fixed: bool, config: FooterConfig) -> Entity {
    let mut sections = None;

    let footer = commands
        .spawn(FooterBundle::new(fixed, config.clone()))
        .with_children(|parent| {
            if config.show_border {
                parent.spawn((
                    FooterTopDivider,
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        right: Val::Px(0.0),
                        top: Val::Px(0.0),
                        height: Val::Px(1.0),
                        ..default()
                    },
                    BackgroundColor(config.border_color),
                ));
            }

            let left = parent
                .spawn((
                    FooterSection::Left,
                    FooterLeft,
                    footer_section(1, JustifyContent::FlexStart, config.gap, true),
                ))
                .id();
            let center = parent
                .spawn((
                    FooterSection::Center,
                    FooterCenter,
                    footer_section(2, JustifyContent::Center, config.gap, false),
                ))
                .id();
            let right = parent
                .spawn((
                    FooterSection::Right,
                    FooterRight,
                    footer_section(3, JustifyContent::FlexEnd, config.gap, true),
                ))
                .id();

            sections = Some((left, center, right));
        })
        .id();

    let (left, center, right) = sections.expect("footer sections must be created");

    commands.entity(footer).insert(FooterSections {
        left,
        center,
        right,
    });

    footer
}

pub fn add_to_left(
    commands: &mut Commands,
    sections: &FooterSections,
    bundle: impl Bundle,
) -> Entity {
    add_to_section(commands, sections.left, bundle)
}

pub fn add_to_center(
    commands: &mut Commands,
    sections: &FooterSections,
    bundle: impl Bundle,
) -> Entity {
    add_to_section(commands, sections.center, bundle)
}

pub fn add_to_right(
    commands: &mut Commands,
    sections: &FooterSections,
    bundle: impl Bundle,
) -> Entity {
    add_to_section(commands, sections.right, bundle)
}

fn add_to_section(commands: &mut Commands, section: Entity, bundle: impl Bundle) -> Entity {
    let mut child = None;
    commands.entity(section).with_children(|parent| {
        child = Some(parent.spawn(bundle).id());
    });
    child.expect("footer section child must be created")
}

fn footer_section(column: i16, justify_content: JustifyContent, gap: Val, padded: bool) -> Node {
    Node {
        grid_column: GridPlacement::start(column),
        height: Val::Percent(100.0),
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        justify_content,
        column_gap: gap,
        padding: if padded {
            UiRect::horizontal(Val::Px(FOOTER_SIDE_PADDING))
        } else {
            UiRect::default()
        },
        ..default()
    }
}

fn footer_fixed_system(mut query: Query<(&Footer, &mut Node), Changed<Footer>>) {
    for (footer, mut node) in &mut query {
        if footer.fixed {
            node.position_type = PositionType::Absolute;
            node.width = Val::Percent(100.0);
            node.max_width = Val::Percent(100.0);
            node.min_width = Val::Percent(100.0);
            node.left = Val::Px(0.0);
            node.right = Val::Px(0.0);
            node.bottom = Val::Px(0.0);
        } else {
            node.position_type = PositionType::Relative;
            node.width = Val::Percent(100.0);
            node.max_width = Val::Percent(100.0);
            node.min_width = Val::Percent(100.0);
            node.left = Val::Auto;
            node.right = Val::Auto;
            node.bottom = Val::Auto;
        }
    }
}
