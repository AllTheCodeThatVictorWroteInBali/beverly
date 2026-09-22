//! Opt-in optical fixture for the real shared playback presentation.
//!
//! Register `spawn_glass_demo` in PostUpdate, before PlayButtonSystems and
//! VisibilitySystems::VisibilityPropagate. It waits for Ready + the default UI
//! camera, then hides AppRootSurface (including footer glass). Display::None
//! also empties layout for the existing SVG proxies, which check computed size
//! rather than inherited visibility. No app entities are removed. All background
//! subtrees precede every lens subtree.
//!
//! UI_GLASS_DEMO=1; UI_GLASS_DEMO_THEME=dark|light (unset preserves theme);
//! UI_GLASS_DEMO_STATE=hover|press|release|focus|disabled (unset is idle).
//! Release holds a press for 45 fixture frames, then releases it. Capture near
//! frame 50 for recoil, or 180 for settled output. Focus targets the first large
//! circle; only one control owns keyboard focus. Use UI_AUDIT_NODES=0 and leave
//! UI_RENDERING_DEMO unset. UI_AUDIT_FRAMES / UI_AUDIT_SCREENSHOT capture the
//! entire window via the existing audit; 1100x700 or larger is recommended.

use bevy::{input_focus::{FocusCause, InputFocus, InputFocusVisible}, prelude::*, ui::IsDefaultUiCamera};

use crate::{
    animation::loading::{AppState, LoadingAssets},
    components::play_button::PlayButton,
    icons::{Icon, IconNode},
    primitives::a11y::TabGroup,
    primitives::interaction::DisabledInteraction,
    primitives::root::AppRootSurface,
    theme::{ThemeChanged, ThemeMode, ThemeResource, dark_theme, light_theme},
};

use super::{GradientStop, LinearGradient, Paint, Surface};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum DemoState {
    #[default]
    Idle,
    Hover,
    Press,
    Release,
    Focus,
    Disabled,
}

impl DemoState {
    fn parse(value: Option<&str>) -> Self {
        match value {
            Some("hover") => Self::Hover,
            Some("press") => Self::Press,
            Some("release") => Self::Release,
            Some("focus") => Self::Focus,
            Some("disabled") => Self::Disabled,
            _ => Self::Idle,
        }
    }

    fn interaction(self, frame: u64) -> Interaction {
        match self {
            Self::Hover => Interaction::Hovered,
            Self::Press => Interaction::Pressed,
            Self::Release if frame < 45 => Interaction::Pressed,
            _ => Interaction::None,
        }
    }
}

#[derive(Resource)]
struct GlassDemoRuntime {
    state: DemoState,
    frame: u64,
    focus: Entity,
}

#[derive(Component)]
struct GlassDemoControl;

#[derive(Component)]
struct GlassDemoBackgrounds;

#[derive(Component)]
struct GlassDemoLenses;

/// Public registration entry point; call every PostUpdate, not just at startup.
/// Order before `PlayButtonSystems` and visibility propagation. Synthetic input
/// runs after normal Update writers and never carries scene playback markers.
pub fn spawn_glass_demo(world: &mut World) {
    if !world.contains_resource::<GlassDemoRuntime>() {
        if std::env::var("UI_GLASS_DEMO").ok().as_deref() != Some("1")
            || world.get_resource::<State<AppState>>().map(State::get) != Some(&AppState::Ready)
        {
            return;
        }
        let mut cameras = world.query_filtered::<Entity, (With<Camera>, With<IsDefaultUiCamera>)>();
        let Ok(camera) = cameras.single(world) else { return };
        let Some(assets) = world.get_resource::<LoadingAssets>().cloned() else { return };
        let requested = match std::env::var("UI_GLASS_DEMO_THEME").ok().as_deref() {
            Some("dark") => Some(dark_theme()),
            Some("light") => Some(light_theme()),
            None => None,
            Some(value) => {
                warn!("UI_GLASS_DEMO_THEME={value:?}: expected dark or light; preserving theme");
                None
            }
        };
        if let Some(theme) = requested {
            world.resource_mut::<ThemeResource>().current = theme;
            world.write_message(ThemeChanged { mode: theme.mode });
        }
        let theme = *world.resource::<ThemeResource>();
        let state = DemoState::parse(std::env::var("UI_GLASS_DEMO_STATE").ok().as_deref());
        let focus = spawn_fixture(world, camera, &assets, &theme, state);
        world.insert_resource(GlassDemoRuntime { state, frame: 0, focus });
        info!("Glass optical fixture: theme={:?} state={state:?}, 45 shared PlayButtons; app root hidden; backgrounds before all lenses", theme.current.mode);
    }

    // Reassert isolation: a theme change can rebuild app content on a later tick.
    // InheritedVisibility excludes descendants from backdrop capture. Empty
    // layout additionally hides SVG proxies, which only check ComputedNode size.
    let mut roots = world.query_filtered::<(&mut Visibility, &mut Node), With<AppRootSurface>>();
    for (mut visibility, mut node) in roots.iter_mut(world) {
        visibility.set_if_neq(Visibility::Hidden);
        if node.display != Display::None {
            node.display = Display::None;
        }
    }
    world.resource_scope(|world, mut runtime: Mut<GlassDemoRuntime>| {
        let interaction = runtime.state.interaction(runtime.frame);
        let mut controls = world.query_filtered::<&mut Interaction, With<GlassDemoControl>>();
        for mut current in controls.iter_mut(world) {
            current.set_if_neq(interaction);
        }
        if runtime.state == DemoState::Focus {
            if world.resource::<InputFocus>().get() != Some(runtime.focus) {
                world.resource_mut::<InputFocus>().set(runtime.focus, FocusCause::Navigated);
            }
            world.resource_mut::<InputFocusVisible>().0 = true;
        }
        if runtime.state == DemoState::Release && runtime.frame == 45 {
            info!("Glass optical fixture: release at fixture frame 45");
        }
        runtime.frame += 1;
    });
}

fn absolute(left: f32, top: f32, width: f32, height: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: percent(left), top: percent(top),
        width: percent(width), height: percent(height),
        ..default()
    }
}

fn label(world: &mut World, parent: Entity, text: impl Into<String>, font: &Handle<Font>, color: Color, size: f32, node: Node) {
    world.spawn((
        ChildOf(parent), node, Text::new(text), TextColor(color),
        TextFont { font: FontSource::Handle(font.clone()), font_size: FontSize::Px(size), ..default() },
        Pickable::IGNORE,
    ));
}

fn spawn_fixture(world: &mut World, camera: Entity, assets: &LoadingAssets, theme: &ThemeResource, state: DemoState) -> Entity {
    let colors = theme.current.colors;
    let root = world.spawn((
        Name::new("glass-optical-fixture"), absolute(0.0, 0.0, 100.0, 100.0),
        UiTargetCamera(camera), GlobalZIndex(200_000), TabGroup::new(100),
        BackgroundColor(colors.background), Pickable::IGNORE,
    )).id();
    label(world, root,
        format!("GLASS / optical matrix    {:?} · {state:?}    |    small + large · 45 controls", theme.current.mode),
        &assets.font, colors.text, 18.0, absolute(2.0, 2.0, 96.0, 5.0));

    // Sibling layers, NOT interleaved background/control card subtrees. The
    // resolved stack order (not just entity allocation order) puts even the last
    // column's stripes/image into the snapshot taken before the first lens.
    let backgrounds = world.spawn((
        Name::new("glass-backgrounds-before-capture"), GlassDemoBackgrounds,
        ChildOf(root), absolute(0.0, 0.0, 100.0, 100.0), ZIndex(0), Pickable::IGNORE,
    )).id();
    let names = ["DARK", "LIGHT", "GRADIENT", "MEDIA IMAGE", "BRIGHT ACCENTS"];
    for (column, name) in names.iter().enumerate() {
        let panel = world.spawn((
            ChildOf(backgrounds), absolute(column as f32 * 20.0 + 0.6, 9.0, 18.8, 88.0),
            BackgroundColor(match column {
                1 => Color::srgb(0.92, 0.94, 0.96),
                _ => Color::srgb(0.035, 0.045, 0.06),
            }), Pickable::IGNORE,
        )).id();
        if column == 2 {
            world.entity_mut(panel).insert(Surface::rounded_rect_fill(0.0, Paint::linear(
                LinearGradient::vertical(vec![
                    GradientStop::new(0.0, Color::srgb(0.08, 0.25, 0.95)),
                    GradientStop::new(0.5, Color::srgb(0.92, 0.14, 0.48)),
                    GradientStop::new(1.0, Color::srgb(0.98, 0.73, 0.15)),
                ]),
            )));
        }
        if column == 3 {
            world.spawn((ChildOf(panel), absolute(0.0, 0.0, 100.0, 100.0),
                ImageNode::new(assets.logo.clone()), Pickable::IGNORE));
        }
        for stripe in 0..12 {
            let accent = [Color::srgb(0.0, 0.95, 0.86), Color::srgb(1.0, 0.12, 0.55), Color::srgb(1.0, 0.88, 0.1)][stripe % 3];
            let color = if column == 4 { accent } else if stripe % 2 == 0 {
                Color::srgba(1.0, 1.0, 1.0, 0.6)
            } else { Color::srgba(0.0, 0.0, 0.0, 0.65) };
            world.spawn((ChildOf(panel), absolute(4.0 + stripe as f32 * 8.0, 7.0, 1.1, 91.0),
                BackgroundColor(color), Pickable::IGNORE));
        }
        for line in [23.0, 47.0, 71.0, 87.0] {
            world.spawn((ChildOf(panel), absolute(0.0, line, 100.0, 0.4),
                BackgroundColor(Color::srgb(0.1, 0.9, 0.95)), Pickable::IGNORE));
        }
        label(world, panel, *name, &assets.font, if column == 1 { Color::BLACK } else { Color::WHITE },
            13.0, absolute(5.0, 1.0, 94.0, 5.0));
        for (row, caption) in ["Circle / 36 · 72", "Pill / 32 · 52", "Rounded / r10", "Nested / rotate + XY scale"].iter().enumerate() {
            // Caption plates also precede capture. They sit below, never inside,
            // the controls and cannot be rewritten by PlayButton presentation.
            let plate = world.spawn((ChildOf(backgrounds),
                absolute(column as f32 * 20.0 + 0.8, 31.68 + row as f32 * 20.0, 18.4, 4.14),
                BackgroundColor(if theme.current.mode == ThemeMode::Dark {
                    Color::srgb(0.04, 0.04, 0.05)
                } else { Color::srgb(0.97, 0.98, 0.99) }), Pickable::IGNORE,
            )).id();
            label(world, plate, *caption, &assets.font, colors.text, 11.0,
                Node { margin: UiRect::all(px(3)), ..default() });
        }
    }

    let lenses = world.spawn((
        Name::new("glass-lenses-after-all-backgrounds"), GlassDemoLenses,
        ChildOf(root), absolute(0.0, 0.0, 100.0, 100.0), ZIndex(1), Pickable::IGNORE,
    )).id();
    let mut focus = None;
    for column in 0..5 {
        for row in 0..4 {
            let cell = world.spawn((ChildOf(lenses),
                absolute(column as f32 * 20.0, 18.0 + row as f32 * 20.0, 20.0, 18.0),
                Pickable::IGNORE,
            )).id();
            let holder = world.spawn((ChildOf(cell), Node {
                width: percent(100), height: percent(72),
                align_items: AlignItems::Center, justify_content: JustifyContent::Center,
                column_gap: px(14), ..default()
            }, Pickable::IGNORE)).id();
            let (small, large, radius) = match row {
                0 => (Vec2::splat(36.0), Vec2::splat(72.0), 999.0),
                1 => (Vec2::new(64.0, 32.0), Vec2::new(100.0, 52.0), 999.0),
                2 => (Vec2::new(44.0, 36.0), Vec2::new(88.0, 64.0), 10.0),
                _ => (Vec2::splat(34.0), Vec2::splat(46.0), 12.0),
            };
            if row == 3 {
                world.entity_mut(holder).insert(UiTransform {
                    rotation: Rot2::degrees(-9.0), scale: Vec2::new(1.08, 0.82), ..default()
                });
            }
            for index in 0..if row == 3 { 3 } else { 2 } {
                let parent = if row == 3 {
                    world.spawn((ChildOf(holder), Node::default(), Pickable::IGNORE,
                        UiTransform { rotation: Rot2::degrees(23.0), scale: Vec2::new(0.82, 1.18), ..default() },
                    )).id()
                } else { holder };
                let size = if index == 0 { small } else { large };
                let button = world.spawn((
                    ChildOf(parent), GlassDemoControl, PlayButton { playing: index == 2 },
                    Name::new(format!("glass-{column}-{row}-{index}")),
                    Node { width: px(size.x), height: px(size.y),
                        border_radius: BorderRadius::all(px(radius)),
                        align_items: AlignItems::Center, justify_content: JustifyContent::Center,
                        flex_shrink: 0.0, ..default() },
                    Pickable::IGNORE,
                )).id();
                if state == DemoState::Disabled {
                    world.entity_mut(button).insert(DisabledInteraction);
                }
                // Real IconNode: the existing SVG proxy camera remains after
                // the capture/composite pass. No fixture-specific icon overlay.
                let icon_size = if index == 0 { 14.0 } else { 24.0 };
                world.spawn((ChildOf(button),
                    Node { width: px(icon_size), height: px(icon_size), ..default() },
                    IconNode::new(Icon::feather("play")).size(icon_size), Pickable::IGNORE,
                ));
                if column == 0 && row == 0 && index == 1 { focus = Some(button); }
            }
        }
    }
    focus.expect("matrix includes the focus control")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_is_a_real_press_then_release() {
        assert_eq!(DemoState::parse(Some("release")), DemoState::Release);
        assert_eq!(DemoState::Release.interaction(44), Interaction::Pressed);
        assert_eq!(DemoState::Release.interaction(45), Interaction::None);
        assert_eq!(DemoState::Disabled.interaction(0), Interaction::None);
        assert_eq!(DemoState::Hover.interaction(180), Interaction::Hovered);
    }

    #[test]
    fn all_backgrounds_are_in_a_lower_sibling_layer_than_all_45_controls() {
        let mut world = World::new();
        let camera = world.spawn_empty().id();
        let assets = LoadingAssets { font: default(), logo: default() };
        let focus = spawn_fixture(&mut world, camera, &assets, &ThemeResource::default(), DemoState::Idle);
        assert!(world.get::<GlassDemoControl>(focus).is_some());
        let backgrounds = world.query_filtered::<Entity, With<GlassDemoBackgrounds>>().single(&world).unwrap();
        let lenses = world.query_filtered::<Entity, With<GlassDemoLenses>>().single(&world).unwrap();
        assert_eq!(world.get::<ChildOf>(backgrounds), world.get::<ChildOf>(lenses));
        assert!(world.get::<ZIndex>(backgrounds).unwrap().0 < world.get::<ZIndex>(lenses).unwrap().0);
        let controls: Vec<_> = world.query_filtered::<Entity, With<GlassDemoControl>>().iter(&world).collect();
        assert_eq!(controls.len(), 45);
        for control in controls {
            let mut ancestor = control;
            while ancestor != lenses {
                ancestor = world.get::<ChildOf>(ancestor).expect("control belongs to lens layer").parent();
                assert_ne!(ancestor, backgrounds);
            }
        }
        assert!(world.query::<&Surface>().iter(&world).all(|surface| surface.backdrop.is_none()));
    }
}