//! State decoration demo: the snack desk's focus ring workshop.
//! Run with `-- dark` to start in dark mode.

use beverly::components::button::BeverlyButton;
use beverly::components::slider::{Slider, SliderStyle, spawn_slider};
use beverly::components::tabs::{Tab, TabsConfig, spawn_tabs_in_with_bar};
use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::prelude::*;
use beverly::primitives::a11y::TabGroup;
use beverly::rendering::{
    FocusRing, FocusRingLayer, FocusRingPlacement, OuterGlow, Paint, ShadowFalloff, Surface,
};
use bevy::prelude::*;

const INTENTS: [&str; 3] = ["Default", "Selected", "Invalid"];
const PLACEMENTS: [(&str, FocusRingPlacement); 3] = [
    ("Outside", FocusRingPlacement::Outside),
    ("Center", FocusRingPlacement::Center),
    ("Inside", FocusRingPlacement::Inside),
];

#[derive(Resource, Clone, PartialEq)]
struct RingSettings {
    width: f32,
    offset: f32,
    placement: FocusRingPlacement,
    intent: usize,
    secondary: bool,
    glow: bool,
}

impl Default for RingSettings {
    fn default() -> Self {
        Self {
            width: 3.0,
            offset: 3.0,
            placement: FocusRingPlacement::Outside,
            intent: 0,
            secondary: false,
            glow: true,
        }
    }
}

#[derive(Component)]
struct RingControl(usize);

#[derive(Component, Clone, Copy, PartialEq)]
enum Setting {
    Placement(FocusRingPlacement),
    Intent(usize),
    Secondary,
    Glow,
}

#[derive(Component)]
struct SettingButton(Setting);

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
enum RingPreview {
    Main,
    Outside,
    Center,
    Inside,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .init_resource::<RingSettings>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (read_sliders, update_previews, style_buttons).chain(),
        )
        .run();
}

fn setup(mut commands: Commands, theme: Res<ThemeResource>) {
    spawn_themed_page(&mut commands, |root| {
        root.spawn(Node {
            width: Val::Px(760.0),
            max_width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(16.0),
            ..default()
        })
        .with_children(|content| {
            content.spawn((
                ThemedTitle::new(TitleLevel::H2),
                Text::new("The Focus Ring Workshop"),
            ));
            content.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("Making sure the keyboard always knows where the biscuits are."),
            ));
            spawn_focus_targets(content);
            content.spawn((
                ThemedText::new(TextRole::Label),
                Text::new("Authored focus ring"),
            ));
            spawn_controls(content, &theme);
            spawn_previews(content, &theme);
            content.spawn((
                ThemedText::new(TextRole::Caption),
                Text::new("Only the focus ring renders today. Selection and validation decorations are reserved slots with no visuals yet; use fill, border, or a Selected/Invalid colored ring instead."),
            ));
        });
    });
}

/// The built-in tabs component: Beverly applies the theme's focus ring to the focused tab.
fn spawn_focus_targets(content: &mut ChildSpawnerCommands) {
    content.spawn((
        ThemedText::new(TextRole::Label),
        Text::new("Automatic focus ring"),
    ));
    content.spawn((
        ThemedText::new(TextRole::Caption),
        Text::new(
            "Press Tab to enter the tabs, then the arrow keys to move. The ring comes from the theme and follows accessibility settings.",
        ),
    ));
    content
        .spawn((
            TabGroup::default(),
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(72.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
        ))
        .with_children(|group| {
            spawn_tabs_in_with_bar(
                group,
                TabsConfig::new(vec![
                    Tab::new("biscuits", "Biscuits"),
                    Tab::new("tea", "Tea"),
                    Tab::new("toast", "Toast"),
                ]),
                |_| {},
                |_, _, _| {},
            );
        });
}

fn spawn_controls(content: &mut ChildSpawnerCommands, theme: &ThemeResource) {
    content
        .spawn(Node {
            column_gap: Val::Px(24.0),
            row_gap: Val::Px(10.0),
            flex_wrap: FlexWrap::Wrap,
            ..default()
        })
        .with_children(|controls| {
            for (index, (label, min, max, value)) in [
                ("Ring width (px)", 1.0, 8.0, 3.0),
                ("Ring offset (px)", 0.0, 12.0, 3.0),
            ]
            .into_iter()
            .enumerate()
            {
                controls
                    .spawn(Node {
                        width: Val::Px(232.0),
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
                                width: 232.0,
                                show_value: true,
                                track_color: theme.current.colors.border,
                                fill_color: theme.current.colors.text,
                                thumb_color: theme.current.colors.surface_elevated,
                                ..default()
                            },
                            theme,
                        );
                        row.commands().entity(slider).insert(RingControl(index));
                    });
            }
        });
    content
        .spawn(Node {
            column_gap: Val::Px(8.0),
            row_gap: Val::Px(8.0),
            flex_wrap: FlexWrap::Wrap,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|buttons| {
            buttons.spawn((ThemedText::new(TextRole::Label), Text::new("Placement")));
            for (label, placement) in PLACEMENTS {
                buttons.spawn((
                    SettingButton(Setting::Placement(placement)),
                    BeverlyButton::standard(label).on("click", select_setting),
                ));
            }
            buttons.spawn((
                Node {
                    margin: UiRect::left(Val::Px(12.0)),
                    ..default()
                },
                ThemedText::new(TextRole::Label),
                Text::new("Color"),
            ));
            for (index, label) in INTENTS.into_iter().enumerate() {
                buttons.spawn((
                    SettingButton(Setting::Intent(index)),
                    BeverlyButton::standard(label).on("click", select_setting),
                ));
            }
            buttons.spawn((
                Node {
                    margin: UiRect::left(Val::Px(12.0)),
                    ..default()
                },
                SettingButton(Setting::Secondary),
                BeverlyButton::standard("Second ring").on("click", select_setting),
            ));
            buttons.spawn((
                SettingButton(Setting::Glow),
                BeverlyButton::standard("Glow").on("click", select_setting),
            ));
        });
}

fn spawn_previews(content: &mut ChildSpawnerCommands, theme: &ThemeResource) {
    let settings = RingSettings::default();
    content
        .spawn(Node {
            column_gap: Val::Px(36.0),
            row_gap: Val::Px(28.0),
            padding: UiRect::axes(Val::Px(16.0), Val::Px(20.0)),
            flex_wrap: FlexWrap::Wrap,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|previews| {
            previews
                .spawn((
                    RingPreview::Main,
                    Node {
                        width: Val::Px(260.0),
                        height: Val::Px(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    preview_surface(RingPreview::Main, &settings, theme),
                ))
                .with_children(|panel| {
                    panel.spawn((ThemedText::new(TextRole::Body), Text::new("Tune the ring")));
                });
            for (kind, label) in [
                (RingPreview::Outside, "Outside"),
                (RingPreview::Center, "Center"),
                (RingPreview::Inside, "Inside"),
            ] {
                previews
                    .spawn((
                        kind,
                        Node {
                            width: Val::Px(96.0),
                            height: Val::Px(64.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        preview_surface(kind, &settings, theme),
                    ))
                    .with_children(|panel| {
                        panel.spawn((ThemedText::new(TextRole::Caption), Text::new(label)));
                    });
            }
        });
}

fn ring_for(
    settings: &RingSettings,
    placement: FocusRingPlacement,
    theme: &ThemeResource,
) -> FocusRing {
    let colors = theme.current.colors;
    let color = [colors.focus, colors.info, colors.error][settings.intent.min(2)];
    let mut ring = FocusRing::outside(settings.width, settings.offset, Paint::solid(color))
        .with_placement(placement);
    if settings.secondary {
        let contrast = if color.to_linear().luminance() > 0.5 {
            Color::BLACK
        } else {
            Color::WHITE
        };
        ring = ring.with_secondary(
            FocusRingLayer::new(settings.width + 1.0, 0.0, Paint::solid(contrast))
                .with_opacity(0.95),
        );
    }
    if settings.glow {
        ring = ring.with_glow(
            OuterGlow::new(color)
                .with_blur(16.0)
                .with_spread(2.0)
                .with_opacity(0.3)
                .with_falloff(ShadowFalloff::Gaussian),
        );
    }
    ring
}

fn preview_placement(kind: RingPreview, settings: &RingSettings) -> FocusRingPlacement {
    match kind {
        RingPreview::Main => settings.placement,
        RingPreview::Outside => FocusRingPlacement::Outside,
        RingPreview::Center => FocusRingPlacement::Center,
        RingPreview::Inside => FocusRingPlacement::Inside,
    }
}

fn preview_surface(kind: RingPreview, settings: &RingSettings, theme: &ThemeResource) -> Surface {
    let colors = theme.current.colors;
    let surface = Surface::rounded_rect_fill(8.0, Paint::solid(colors.surface_elevated));
    let surface = if kind == RingPreview::Main {
        surface
    } else {
        surface.uniform_border(1.0, Paint::solid(colors.border))
    };
    surface.with_focus_ring(ring_for(settings, preview_placement(kind, settings), theme))
}

fn select_setting(commands: &mut Commands, button: Entity) {
    commands.queue(move |world: &mut World| {
        let Some(setting) = world.get::<SettingButton>(button).map(|button| button.0) else {
            return;
        };
        let mut settings = world.resource_mut::<RingSettings>();
        match setting {
            Setting::Placement(placement) => settings.placement = placement,
            Setting::Intent(intent) => settings.intent = intent,
            Setting::Secondary => settings.secondary = !settings.secondary,
            Setting::Glow => settings.glow = !settings.glow,
        }
    });
}

fn read_sliders(controls: Query<(&RingControl, &Slider)>, mut settings: ResMut<RingSettings>) {
    for (control, slider) in &controls {
        let target = if control.0 == 0 {
            settings.width
        } else {
            settings.offset
        };
        if target != slider.value {
            if control.0 == 0 {
                settings.width = slider.value;
            } else {
                settings.offset = slider.value;
            }
        }
    }
}

fn update_previews(
    settings: Res<RingSettings>,
    theme: Res<ThemeResource>,
    mut previews: Query<(&RingPreview, &mut Surface)>,
) {
    for (kind, mut surface) in &mut previews {
        let next = preview_surface(*kind, &settings, &theme);
        if *surface != next {
            *surface = next;
        }
    }
}

fn style_buttons(
    settings: Res<RingSettings>,
    mut buttons: Query<(&SettingButton, &mut BeverlyButton)>,
) {
    for (button, mut view) in &mut buttons {
        let selected = match button.0 {
            Setting::Placement(placement) => settings.placement == placement,
            Setting::Intent(intent) => settings.intent == intent,
            Setting::Secondary => settings.secondary,
            Setting::Glow => settings.glow,
        };
        if view.outline == selected {
            view.outline = !selected;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use beverly::rendering::Surface;

    fn theme() -> ThemeResource {
        ThemeResource::default()
    }

    #[test]
    fn ring_settings_map_to_ring_layers_and_theme_colors() {
        let theme = theme();
        let colors = theme.current.colors;
        let base = RingSettings {
            width: 5.0,
            offset: 7.0,
            placement: FocusRingPlacement::Center,
            intent: 2,
            secondary: false,
            glow: false,
        };
        let ring = ring_for(&base, base.placement, &theme);
        assert_eq!(ring.placement, FocusRingPlacement::Center);
        assert_eq!(ring.primary.width, 5.0);
        assert_eq!(ring.primary.offset, 7.0);
        assert_eq!(ring.primary.paint, Paint::solid(colors.error));
        assert!(ring.secondary.is_none() && ring.glow.is_none());

        let full = RingSettings {
            secondary: true,
            glow: true,
            intent: 1,
            ..base
        };
        let ring = ring_for(&full, FocusRingPlacement::Inside, &theme);
        assert_eq!(ring.placement, FocusRingPlacement::Inside);
        assert_eq!(ring.primary.paint, Paint::solid(colors.info));
        assert_eq!(ring.secondary.as_ref().map(|layer| layer.width), Some(6.0));
        assert!(ring.glow.is_some());
    }

    #[test]
    fn main_preview_follows_placement_while_comparison_swatches_keep_theirs() {
        let mut app = App::new();
        app.init_resource::<ThemeResource>()
            .insert_resource(RingSettings {
                placement: FocusRingPlacement::Inside,
                width: 6.0,
                ..default()
            })
            .add_systems(Update, update_previews);
        let theme = theme();
        let kinds = [
            RingPreview::Main,
            RingPreview::Outside,
            RingPreview::Center,
            RingPreview::Inside,
        ];
        for kind in kinds {
            app.world_mut().spawn((
                kind,
                preview_surface(kind, &RingSettings::default(), &theme),
            ));
        }
        app.update();
        let mut query = app.world_mut().query::<(&RingPreview, &Surface)>();
        for (kind, surface) in query.iter(app.world()) {
            let ring = surface.decorations.focus_ring.as_ref().unwrap();
            assert_eq!(surface.border.is_some(), *kind != RingPreview::Main);
            let expected = match kind {
                RingPreview::Main | RingPreview::Inside => FocusRingPlacement::Inside,
                RingPreview::Outside => FocusRingPlacement::Outside,
                RingPreview::Center => FocusRingPlacement::Center,
            };
            assert_eq!(ring.placement, expected, "{kind:?}");
            assert_eq!(ring.primary.width, 6.0, "{kind:?}");
        }
    }

    #[test]
    fn setting_buttons_set_choices_and_toggle_options() {
        let mut world = World::new();
        world.insert_resource(RingSettings::default());
        let placement = world
            .spawn(SettingButton(Setting::Placement(
                FocusRingPlacement::Center,
            )))
            .id();
        let intent = world.spawn(SettingButton(Setting::Intent(2))).id();
        let secondary = world.spawn(SettingButton(Setting::Secondary)).id();
        let glow = world.spawn(SettingButton(Setting::Glow)).id();
        for button in [placement, intent, secondary, glow] {
            select_setting(&mut world.commands(), button);
        }
        world.flush();
        let settings = world.resource::<RingSettings>().clone();
        assert_eq!(settings.placement, FocusRingPlacement::Center);
        assert_eq!(settings.intent, 2);
        assert!(settings.secondary);
        assert!(!settings.glow, "glow starts on, so one press turns it off");
        select_setting(&mut world.commands(), secondary);
        world.flush();
        assert!(!world.resource::<RingSettings>().secondary);
    }

    #[test]
    fn focus_section_is_a_tabs_component_inside_one_tab_group() {
        use beverly::components::tabs::TabButton;

        let mut world = World::new();
        world
            .commands()
            .spawn(Node::default())
            .with_children(spawn_focus_targets);
        world.flush();
        let mut groups = world.query_filtered::<Entity, With<TabGroup>>();
        assert_eq!(groups.iter(&world).count(), 1);
        let mut buttons = world.query::<&TabButton>();
        assert_eq!(buttons.iter(&world).count(), 3);
    }

    #[test]
    fn tab_key_focuses_a_tab_button_and_gives_it_a_ring() {
        use beverly::primitives::a11y::A11yPlugin;
        use bevy::input::keyboard::{Key, KeyboardInput};
        use bevy::input::{ButtonState, InputPlugin};
        use bevy::input_focus::{InputDispatchPlugin, InputFocus, InputFocusPlugin};
        use bevy::window::{PrimaryWindow, Window};

        let mut app = App::new();
        app.add_plugins((
            InputPlugin,
            InputFocusPlugin,
            InputDispatchPlugin,
            A11yPlugin,
        ))
        .init_resource::<ThemeResource>()
        .init_resource::<beverly::theme::AccessibilityVisualPolicyResource>()
        .init_resource::<Time>();
        let window = app
            .world_mut()
            .spawn((Window::default(), PrimaryWindow))
            .id();
        app.world_mut()
            .commands()
            .spawn(Node::default())
            .with_children(spawn_focus_targets);
        app.update();
        app.world_mut().write_message(KeyboardInput {
            key_code: KeyCode::Tab,
            logical_key: Key::Tab,
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window,
        });
        app.update();
        app.update();

        let focused = app.world().resource::<InputFocus>().get();
        let focused = focused.expect("Tab should focus a target");
        assert!(
            app.world()
                .get::<beverly::components::tabs::TabButton>(focused)
                .is_some(),
            "Tab should land on a tab button"
        );
        let surface = app
            .world()
            .get::<Surface>(focused)
            .expect("focused tab should have a surface");
        let has_target = app
            .world()
            .get::<beverly::animation::animation::SurfaceTransitionTarget>(focused)
            .is_some_and(|t| t.target.decorations.focus_ring.is_some());
        assert!(surface.decorations.focus_ring.is_some() || has_target);
    }
}
