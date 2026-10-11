//! Loading skeleton demo: the snack desk's biscuit delivery dock.
//! Run with `-- dark` to start in dark mode.

use beverly::animation::skeleton::{Skeleton, SkeletonDirection, SkeletonGroup};
use beverly::components::button::BeverlyButton;
use beverly::components::slider::{Slider, SliderStyle, spawn_slider};
use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::prelude::*;
use beverly::rendering::{GradientStop, LinearGradient, Paint, Surface};
use bevy::prelude::*;

const RELOAD_SECONDS: f32 = 2.5;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    Skeleton,
    Content,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Action {
    Skeleton,
    Content,
    Reload,
}

#[derive(Resource, Clone, Copy, PartialEq)]
struct DockState {
    mode: Mode,
    direction: SkeletonDirection,
    /// Sweep duration in tenths of a second, and highlight intensity in percent.
    values: [f32; 2],
}

impl Default for DockState {
    fn default() -> Self {
        Self {
            mode: Mode::Skeleton,
            direction: SkeletonDirection::LeftToRight,
            values: [15.0, 65.0],
        }
    }
}

/// Seconds elapsed since "Reload" was pressed, while the skeleton is being held.
#[derive(Resource, Default)]
struct ReloadTimer(Option<f32>);

#[derive(Component)]
struct CardBody;

#[derive(Component)]
struct CardSurface;

#[derive(Component)]
struct DockControl(usize);

#[derive(Component)]
struct ActionButton(Action);

#[derive(Component)]
struct DirectionButton(SkeletonDirection);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .init_resource::<DockState>()
        .init_resource::<ReloadTimer>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                read_sliders,
                tick_reload,
                rebuild_card,
                style_card,
                style_buttons,
            )
                .chain(),
        )
        .run();
}

fn setup(mut commands: Commands, theme: Res<ThemeResource>) {
    spawn_themed_page(&mut commands, |root| {
        root.spawn(Node {
            width: Val::Px(520.0),
            max_width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(16.0),
            ..default()
        })
        .with_children(|content| {
            content.spawn((
                ThemedTitle::new(TitleLevel::H2),
                Text::new("The Biscuit Delivery Dock"),
            ));
            content.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("Placeholders keep the layout steady while the biscuits arrive."),
            ));
            content
                .spawn(Node {
                    column_gap: Val::Px(8.0),
                    row_gap: Val::Px(8.0),
                    flex_wrap: FlexWrap::Wrap,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|buttons| {
                    for (label, action) in [
                        ("Skeleton", Action::Skeleton),
                        ("Content", Action::Content),
                        ("Reload", Action::Reload),
                    ] {
                        buttons.spawn((
                            ActionButton(action),
                            BeverlyButton::standard(label).on("click", select_action),
                        ));
                    }
                    buttons.spawn((
                        Node {
                            margin: UiRect::left(Val::Px(16.0)),
                            ..default()
                        },
                        ThemedText::new(TextRole::Label),
                        Text::new("Sweep"),
                    ));
                    for (label, direction) in [
                        ("Left to right", SkeletonDirection::LeftToRight),
                        ("Right to left", SkeletonDirection::RightToLeft),
                    ] {
                        buttons.spawn((
                            DirectionButton(direction),
                            BeverlyButton::standard(label).on("click", select_direction),
                        ));
                    }
                });
            content
                .spawn(Node {
                    column_gap: Val::Px(24.0),
                    row_gap: Val::Px(10.0),
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                })
                .with_children(|controls| {
                    for (index, (label, min, max, value)) in [
                        ("Sweep duration (0.1 s)", 5.0, 40.0, 15.0),
                        ("Highlight intensity (%)", 0.0, 100.0, 65.0),
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        controls
                            .spawn(Node {
                                width: Val::Px(240.0),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(4.0),
                                ..default()
                            })
                            .with_children(|row| {
                                row.spawn((ThemedText::new(TextRole::Label), Text::new(label)));
                                let slider = spawn_slider(
                                    row,
                                    Slider::new(min, max).value(value).step(1.0).label(label),
                                    SliderStyle {
                                        width: 240.0,
                                        show_value: true,
                                        track_color: theme.current.colors.border,
                                        fill_color: theme.current.colors.text,
                                        thumb_color: theme.current.colors.surface_elevated,
                                        ..default()
                                    },
                                    &theme,
                                );
                                row.commands().entity(slider).insert(DockControl(index));
                            });
                    }
                });
            content
                .spawn((
                    CardSurface,
                    Node {
                        width: Val::Percent(100.0),
                        padding: UiRect::all(Val::Px(24.0)),
                        ..default()
                    },
                    Surface::rounded_rect_fill(12.0, Paint::solid(theme.current.colors.surface))
                        .uniform_border(1.0, Paint::solid(theme.current.colors.border)),
                ))
                .with_children(|card| {
                    card.spawn((
                        CardBody,
                        Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                    ));
                });
        });
    });
}

fn select_action(commands: &mut Commands, button: Entity) {
    commands.queue(move |world: &mut World| {
        let Some(action) = world.get::<ActionButton>(button).map(|button| button.0) else {
            return;
        };
        let mut reload = Some(0.0);
        let mode = match action {
            Action::Skeleton => {
                reload = None;
                Mode::Skeleton
            }
            Action::Content => {
                reload = None;
                Mode::Content
            }
            Action::Reload => Mode::Skeleton,
        };
        world.resource_mut::<ReloadTimer>().0 = reload;
        world.resource_mut::<DockState>().mode = mode;
    });
}

fn select_direction(commands: &mut Commands, button: Entity) {
    commands.queue(move |world: &mut World| {
        if let Some(direction) = world.get::<DirectionButton>(button).map(|button| button.0) {
            world.resource_mut::<DockState>().direction = direction;
        }
    });
}

fn read_sliders(controls: Query<(&DockControl, &Slider)>, mut state: ResMut<DockState>) {
    let mut values = state.values;
    for (control, slider) in &controls {
        values[control.0] = slider.value;
    }
    if state.values != values {
        state.values = values;
    }
}

fn tick_reload(time: Res<Time>, mut timer: ResMut<ReloadTimer>, mut state: ResMut<DockState>) {
    let Some(elapsed) = timer.0 else {
        return;
    };
    let elapsed = elapsed + time.delta_secs();
    if elapsed >= RELOAD_SECONDS {
        timer.0 = None;
        state.mode = Mode::Content;
    } else {
        timer.0 = Some(elapsed);
    }
}

fn skeleton_style(skeleton: Skeleton, state: &DockState) -> Skeleton {
    skeleton
        .direction(state.direction)
        .duration(state.values[0] / 10.0)
        .highlight_intensity(state.values[1] / 100.0)
}

fn spawn_skeleton(body: &mut ChildSpawnerCommands, state: &DockState) {
    let style = |skeleton: Skeleton| skeleton_style(skeleton, state);
    body.spawn((
        SkeletonGroup::new("Loading biscuit delivery"),
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(16.0),
            ..default()
        },
    ))
    .with_children(|group| {
        group
            .spawn(Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(16.0),
                ..default()
            })
            .with_children(|header| {
                header.spawn(style(Skeleton::circle(56.0)));
                header
                    .spawn(Node {
                        flex_grow: 1.0,
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(10.0),
                        ..default()
                    })
                    .with_children(|lines| {
                        lines.spawn(style(Skeleton::text().width(Val::Percent(55.0))));
                        lines.spawn(style(Skeleton::text().width(Val::Percent(35.0))));
                    });
            });
        group.spawn(style(
            Skeleton::new()
                .width(Val::Percent(100.0))
                .height(Val::Px(140.0))
                .radius(8.0),
        ));
        for width in [100.0, 100.0, 60.0] {
            group.spawn(style(Skeleton::text().width(Val::Percent(width))));
        }
    });
}

fn spawn_content(body: &mut ChildSpawnerCommands) {
    body.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(16.0),
        ..default()
    })
    .with_children(|content| {
        content
            .spawn(Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(16.0),
                ..default()
            })
            .with_children(|header| {
                header
                    .spawn((
                        Node {
                            width: Val::Px(56.0),
                            height: Val::Px(56.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        Surface::rounded_rect_fill(
                            28.0,
                            Paint::solid(Color::srgb(0.10, 0.48, 0.34)),
                        ),
                    ))
                    .with_children(|avatar| {
                        avatar.spawn((
                            ThemedText::new(TextRole::Heading).color(Color::WHITE),
                            Text::new("B"),
                        ));
                    });
                header
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        ..default()
                    })
                    .with_children(|lines| {
                        lines.spawn((
                            ThemedTitle::new(TitleLevel::H4),
                            Text::new("Biscuit Delivery Dock"),
                        ));
                        lines.spawn((
                            ThemedText::new(TextRole::Muted),
                            Text::new("Arriving before second breakfast"),
                        ));
                    });
            });
        content
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(140.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                Surface::rounded_rect_fill(
                    8.0,
                    Paint::linear(LinearGradient::angle_degrees(
                        45.0,
                        vec![
                            GradientStop::new(0.0, Color::srgb(0.98, 0.76, 0.18)),
                            GradientStop::new(1.0, Color::srgb(0.94, 0.26, 0.32)),
                        ],
                    )),
                ),
            ))
            .with_children(|photo| {
                photo.spawn((
                    ThemedText::new(TextRole::Heading).color(Color::WHITE),
                    Text::new("Fresh from the oven"),
                ));
            });
        content.spawn((
            ThemedText::new(TextRole::Body),
            Text::new(
                "Today's shipment includes twelve emergency biscuits, three suspiciously \
                 round crackers, and one crumb that has been cleared by the inspector.",
            ),
            Node {
                width: Val::Percent(100.0),
                ..default()
            },
        ));
    });
}

fn rebuild_card(
    mut commands: Commands,
    state: Res<DockState>,
    bodies: Query<Entity, With<CardBody>>,
    mut shown: Local<Option<DockState>>,
) {
    if *shown == Some(*state) {
        return;
    }
    let content_only = |state: &DockState| state.mode == Mode::Content;
    // Settings only affect the skeleton, so the content view needs no rebuild for them.
    if let Some(previous) = *shown
        && content_only(&previous)
        && content_only(&state)
    {
        *shown = Some(*state);
        return;
    }
    *shown = Some(*state);
    for body in &bodies {
        commands
            .entity(body)
            .despawn_related::<Children>()
            .with_children(|body| match state.mode {
                Mode::Skeleton => spawn_skeleton(body, &state),
                Mode::Content => spawn_content(body),
            });
    }
}

fn style_card(theme: Res<ThemeResource>, mut cards: Query<&mut Surface, With<CardSurface>>) {
    let colors = theme.current.colors;
    for mut surface in &mut cards {
        let fill = Paint::solid(colors.surface);
        if surface.fill != fill {
            surface.fill = fill;
        }
        if let Some(border) = surface.border.as_mut() {
            let paint = Paint::solid(colors.border);
            if border.paint != paint {
                border.paint = paint;
            }
        }
    }
}

fn style_buttons(
    state: Res<DockState>,
    timer: Res<ReloadTimer>,
    mut actions: Query<(&ActionButton, &mut BeverlyButton), Without<DirectionButton>>,
    mut directions: Query<(&DirectionButton, &mut BeverlyButton), Without<ActionButton>>,
) {
    for (button, mut view) in &mut actions {
        let selected = match button.0 {
            Action::Skeleton => state.mode == Mode::Skeleton && timer.0.is_none(),
            Action::Content => state.mode == Mode::Content,
            Action::Reload => timer.0.is_some(),
        };
        if view.outline == selected {
            view.outline = !selected;
        }
    }
    for (button, mut view) in &mut directions {
        let outline = button.0 != state.direction;
        if view.outline != outline {
            view.outline = outline;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(state: DockState) -> (App, Entity) {
        let mut app = App::new();
        app.init_resource::<Time>()
            .insert_resource(state)
            .init_resource::<ReloadTimer>()
            .add_systems(Update, (tick_reload, rebuild_card).chain());
        let body = app.world_mut().spawn(CardBody).id();
        (app, body)
    }

    fn counts(app: &mut App) -> (usize, Vec<bool>) {
        let skeletons = app
            .world_mut()
            .query::<&Skeleton>()
            .iter(app.world())
            .count();
        let groups = app
            .world_mut()
            .query::<&SkeletonGroup>()
            .iter(app.world())
            .map(|group| group.busy)
            .collect();
        (skeletons, groups)
    }

    #[test]
    fn skeleton_mode_builds_one_busy_group_of_placeholders() {
        let (mut app, _) = app(DockState::default());
        app.update();
        let (skeletons, groups) = counts(&mut app);
        assert_eq!(
            skeletons, 7,
            "avatar, two heading lines, image, and three lines"
        );
        assert_eq!(groups, vec![true]);
    }

    #[test]
    fn content_mode_removes_every_skeleton_and_the_busy_group() {
        let (mut app, _) = app(DockState {
            mode: Mode::Content,
            ..default()
        });
        app.update();
        let (skeletons, groups) = counts(&mut app);
        assert_eq!(skeletons, 0);
        assert!(groups.is_empty());
    }

    #[test]
    fn settings_rebuild_the_skeleton_without_duplicating_it() {
        let (mut app, _) = app(DockState::default());
        app.update();
        app.world_mut().resource_mut::<DockState>().values = [30.0, 20.0];
        app.world_mut().resource_mut::<DockState>().direction = SkeletonDirection::RightToLeft;
        app.update();
        let (skeletons, groups) = counts(&mut app);
        assert_eq!(skeletons, 7);
        assert_eq!(groups, vec![true]);
    }

    #[test]
    fn reload_holds_the_skeleton_then_shows_content() {
        let (mut app, _) = app(DockState::default());
        app.world_mut().resource_mut::<ReloadTimer>().0 = Some(0.0);
        app.update();
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs_f32(RELOAD_SECONDS - 0.5));
        app.update();
        assert_eq!(app.world().resource::<DockState>().mode, Mode::Skeleton);
        assert!(app.world().resource::<ReloadTimer>().0.is_some());
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs_f32(1.0));
        app.update();
        app.update();
        assert_eq!(app.world().resource::<DockState>().mode, Mode::Content);
        assert!(app.world().resource::<ReloadTimer>().0.is_none());
        assert_eq!(counts(&mut app).0, 0);
    }

    #[test]
    fn button_actions_set_mode_and_cancel_or_start_the_reload() {
        let mut world = World::new();
        world.insert_resource(DockState::default());
        world.insert_resource(ReloadTimer::default());
        let reload = world.spawn(ActionButton(Action::Reload)).id();
        let content = world.spawn(ActionButton(Action::Content)).id();
        select_action(&mut world.commands(), content);
        world.flush();
        assert_eq!(world.resource::<DockState>().mode, Mode::Content);
        select_action(&mut world.commands(), reload);
        world.flush();
        assert_eq!(world.resource::<DockState>().mode, Mode::Skeleton);
        assert_eq!(world.resource::<ReloadTimer>().0, Some(0.0));
        select_action(&mut world.commands(), content);
        world.flush();
        assert!(world.resource::<ReloadTimer>().0.is_none());
    }
}
