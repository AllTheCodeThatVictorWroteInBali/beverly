//! Opt-in native Skeleton fixture. Register `spawn_skeleton_demo` every
//! PostUpdate before `SkeletonSystems` (and therefore before UI preparation).
//! No plugin, shader, capture pipeline, or default application behavior is added.
//! See docs/skeleton-audit.md for environment controls and measurement caveats.

use std::time::Duration;

use bevy::{prelude::*, ui::IsDefaultUiCamera};

use crate::{
    animation::loading::{AppState, LoadingAssets},
    animation::skeleton::{Skeleton, SkeletonDirection, SkeletonGroup},
    primitives::root::AppRootSurface,
    theme::{
        AccessibilityVisualPolicyResource, ThemeChanged, ThemeColors, ThemeMode,
        ThemeResource, dark_theme, light_theme,
    },
};

#[derive(Resource)]
struct SkeletonDemoRuntime {
    frame: u64,
    theme_frame: Option<u64>,
    motion_frame: Option<u64>,
    screenshot_time: Option<f32>,
    scroll: Option<Entity>,
    animate_scroll: bool,
    mode: ThemeMode,
}

#[derive(Component)]
struct SkeletonDemoLeaf;

#[derive(Component, Clone, Copy)]
enum BackdropRole {
    Window,
    Card,
    Inset,
    Accent,
}

impl BackdropRole {
    fn color(self, colors: ThemeColors) -> Color {
        match self {
            Self::Window => colors.background,
            Self::Card => colors.surface,
            Self::Inset => colors.surface_elevated,
            Self::Accent => colors.primary,
        }
    }
}

#[derive(Component)]
struct DemoCaption(bool);

/// Called repeatedly by the parent's rendering plugin; initialization waits for
/// Ready, a default UI camera, and fonts. A valid node count takes precedence
/// over the showcase. Runtime controls are inert unless a fixture is enabled.
pub(super) fn spawn_skeleton_demo(world: &mut World) {
    if !world.contains_resource::<SkeletonDemoRuntime>() {
        let demo = std::env::var("UI_SKELETON_DEMO").ok().as_deref() == Some("1");
        let requested_nodes = std::env::var("UI_SKELETON_NODES").ok();
        if !demo && requested_nodes.is_none() {
            return;
        }
        if world.get_resource::<State<AppState>>().map(State::get) != Some(&AppState::Ready) {
            return;
        }
        let mut cameras = world.query_filtered::<Entity, (With<Camera>, With<IsDefaultUiCamera>)>();
        let Ok(camera) = cameras.single(world) else { return };
        let Some(assets) = world.get_resource::<LoadingAssets>().cloned() else { return };
        let nodes = requested_nodes.as_deref().and_then(|value| value.trim().parse::<usize>().ok())
            .filter(|count| matches!(count, 1 | 10 | 100 | 500 | 1000));
        if requested_nodes.is_some() && nodes.is_none() {
            // Bad configuration must not silently profile the showcase instead.
            warn!("UI_SKELETON_NODES must be 1, 10, 100, 500 or 1000; fixture disabled");
            return;
        }
        // Also support a manual fixture without UI_AUDIT_FRAMES. Normally audit
        // already applied this preference in PreStartup; avoid a redundant event.
        let requested_mode = match std::env::var("UI_AUDIT_THEME").ok().as_deref() {
            Some("dark") => Some(ThemeMode::Dark),
            Some("light") => Some(ThemeMode::Light),
            _ => None,
        };
        if let Some(mode) = requested_mode {
            if world.resource::<ThemeResource>().current.mode != mode {
                set_theme(world, mode);
            }
        }
        let theme = *world.resource::<ThemeResource>();
        let root = world.spawn((
            Name::new("skeleton-native-fixture"), rect(0.0, 0.0, 100.0, 100.0),
            UiTargetCamera(camera), GlobalZIndex(200_000), Pickable::IGNORE,
            BackdropRole::Window, BackgroundColor(theme.current.colors.background),
        )).id();
        let scroll = if let Some(count) = nodes {
            spawn_grid(world, root, count, &assets, theme.current.colors);
            None
        } else {
            Some(spawn_showcase(world, root, &assets, theme.current.colors))
        };
        let screenshot_time = std::env::var("UI_SKELETON_TIME").ok().and_then(|value| {
            let time = value.trim().parse::<f32>().ok()
                .filter(|time| time.is_finite() && (0.0..3600.0).contains(time));
            if time.is_none() {
                warn!("UI_SKELETON_TIME must be finite seconds in [0, 3600); ignoring override");
            }
            time
        });
        world.insert_resource(SkeletonDemoRuntime {
            frame: 0,
            theme_frame: frame_setting("UI_SKELETON_THEME_AT"),
            motion_frame: frame_setting("UI_SKELETON_MOTION_AT"),
            screenshot_time,
            scroll,
            animate_scroll: std::env::var("UI_SKELETON_SCROLL").ok().as_deref() == Some("1"),
            mode: theme.current.mode,
        });
        let leaves = world.query_filtered::<Entity, With<SkeletonDemoLeaf>>().iter(world).count();
        info!("Skeleton fixture: {} mode, {leaves} fixture leaves, theme={:?}; app root hidden, not despawned; UI_SKELETON_TIME={screenshot_time:?}",
            if nodes.is_some() { "dense" } else { "showcase" }, theme.current.mode);
        if screenshot_time.is_some() {
            warn!("Skeleton fixture: frozen generic Time for shader capture; NOT a normal animation/performance run");
        }
    }

    // Repeat after theme-triggered app rebuilds. Display::None also zeros native
    // layout used by the application's separately rendered SVG proxies.
    let mut roots = world.query_filtered::<(&mut Visibility, &mut Node), With<AppRootSurface>>();
    for (mut visibility, mut node) in roots.iter_mut(world) {
        visibility.set_if_neq(Visibility::Hidden);
        if node.display != Display::None { node.display = Display::None; }
    }
    world.resource_scope(|world, mut runtime: Mut<SkeletonDemoRuntime>| {
        if runtime.theme_frame == Some(runtime.frame) {
            let next = match world.resource::<ThemeResource>().current.mode {
                ThemeMode::Light => ThemeMode::Dark,
                ThemeMode::Dark => ThemeMode::Light,
            };
            set_theme(world, next);
            info!("Skeleton fixture: theme toggled to {next:?} at fixture frame {}", runtime.frame);
        }
        if runtime.motion_frame == Some(runtime.frame) {
            let mut policy = world.resource_mut::<AccessibilityVisualPolicyResource>();
            policy.current.reduced_motion = !policy.current.reduced_motion;
            info!("Skeleton fixture: reduced_motion={} at fixture frame {}", policy.current.reduced_motion, runtime.frame);
        }
        let theme = *world.resource::<ThemeResource>();
        if runtime.mode != theme.current.mode {
            retheme_fixture(world, theme.current.colors);
            runtime.mode = theme.current.mode;
        }
        if runtime.animate_scroll {
            if let Some(entity) = runtime.scroll {
                if let Some(mut scroll) = world.get_mut::<ScrollPosition>(entity) {
                    // Frame-driven triangle wave; identical geometry at identical
                    // fixture frames, with no extra timer or input dependency.
                    let step = (runtime.frame % 120) as f32;
                    scroll.0.y = 12.0 + 0.8 * step.min(120.0 - step);
                }
            }
        }
        if let Some(seconds) = runtime.screenshot_time {
            // Bevy's globals shader clock extracts Time<()> after PostUpdate.
            // Do NOT alter Time<Real>/Time<Virtual> or their startup clocks.
            // Holding this on every drain frame removes readback-frame ambiguity.
            let mut time = Time::<()>::default();
            time.advance_by(Duration::from_secs_f32(seconds));
            world.insert_resource(time);
        }
        runtime.frame += 1;
    });
}

fn frame_setting(key: &str) -> Option<u64> {
    std::env::var(key).ok().and_then(|value| {
        let parsed = value.trim().parse().ok();
        if parsed.is_none() { warn!("{key} must be a non-negative fixture frame; ignoring override"); }
        parsed
    })
}

fn set_theme(world: &mut World, mode: ThemeMode) {
    world.resource_mut::<ThemeResource>().current = match mode {
        ThemeMode::Dark => dark_theme(),
        ThemeMode::Light => light_theme(),
    };
    world.write_message(ThemeChanged { mode });
}

fn retheme_fixture(world: &mut World, colors: ThemeColors) {
    let mut backgrounds = world.query::<(&BackdropRole, &mut BackgroundColor)>();
    for (role, mut color) in backgrounds.iter_mut(world) {
        color.0 = role.color(colors);
    }
    let mut captions = world.query::<(&DemoCaption, &mut TextColor)>();
    for (caption, mut color) in captions.iter_mut(world) {
        color.0 = if caption.0 { colors.text_muted } else { colors.text };
    }
    // Existing Skeleton leaves are intentionally untouched: SkeletonSystems
    // must resolve the new semantic colors and accessibility policy itself.
}

fn rect(x: f32, y: f32, width: f32, height: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: percent(x), top: percent(y), width: percent(width), height: percent(height),
        min_width: px(0), min_height: px(0), ..default()
    }
}

fn caption(world: &mut World, parent: Entity, assets: &LoadingAssets, colors: ThemeColors,
    text: impl Into<String>, size: f32, muted: bool, node: Node) {
    world.spawn((
        ChildOf(parent), node, Text::new(text), DemoCaption(muted), Pickable::IGNORE,
        TextColor(if muted { colors.text_muted } else { colors.text }),
        TextFont { font: FontSource::Handle(assets.font.clone()), font_size: FontSize::Px(size), ..default() },
    ));
}

fn panel(world: &mut World, parent: Entity, colors: ThemeColors, mut node: Node, role: BackdropRole) -> Entity {
    node.border_radius = BorderRadius::all(px(16));
    world.spawn((ChildOf(parent), node, role, BackgroundColor(role.color(colors)), Pickable::IGNORE)).id()
}

fn leaf(world: &mut World, parent: Entity, node: Node, skeleton: Skeleton) -> Entity {
    world.spawn((ChildOf(parent), node, skeleton, SkeletonDemoLeaf)).id()
}

fn group(world: &mut World, parent: Entity, label: &str, node: Node) -> Entity {
    world.spawn((ChildOf(parent), node, SkeletonGroup::new(label))).id()
}

fn spawn_grid(world: &mut World, root: Entity, count: usize, assets: &LoadingAssets, colors: ThemeColors) {
    let (columns, rows): (usize, usize) = match count {
        1 => (1, 1), 10 => (5, 2), 100 => (10, 10),
        500 => (25, 20), 1000 => (40, 25), _ => unreachable!(),
    };
    caption(world, root, assets, colors, format!("SKELETON / {count:04} visible leaves"), 24.0, false, rect(3.0, 2.0, 94.0, 6.0));
    caption(world, root, assets, colors, format!("{columns} × {rows}  ·  native Surface / shared GPU clock  ·  no showcase leaves"), 13.0, true, rect(3.0, 9.0, 94.0, 4.0));
    let grid = group(world, root, "Loading profiling grid", rect(3.0, 16.0, 94.0, 80.0));
    // Equal percentage cells with proportional gutters, not fixed pixel gaps:
    // every supported count fits the viewport without scrolling or flex shrink.
    let cell_width = 100.0 / columns as f32;
    let cell_height = 100.0 / rows as f32;
    for index in 0..count {
        let node = rect(
            (index % columns) as f32 * cell_width + cell_width * 0.06,
            (index / columns) as f32 * cell_height + cell_height * 0.10,
            cell_width * 0.88, cell_height * 0.80,
        );
        leaf(world, grid, node, Skeleton::new().radius(4.0));
    }
    info!("Skeleton dense grid: exactly {count} visible fixture Skeleton leaves; {columns} columns × {rows} rows");
}

fn spawn_showcase(world: &mut World, root: Entity, assets: &LoadingAssets, colors: ThemeColors) -> Entity {
    caption(world, root, assets, colors, "BEVERLY / LOADING STUDIES", 12.0, true, rect(3.0, 2.5, 90.0, 3.0));
    caption(world, root, assets, colors, "A little rhythm, while you wait.", 30.0, false, rect(3.0, 6.0, 94.0, 7.0));

    let sidebar = panel(world, root, colors, rect(3.0, 17.0, 18.0, 67.0), BackdropRole::Card);
    caption(world, sidebar, assets, colors, "YOUR LIBRARY", 12.0, true, rect(9.0, 5.0, 84.0, 6.0));
    let nav = group(world, sidebar, "Loading library navigation", rect(9.0, 17.0, 82.0, 56.0));
    for row in 0..5 {
        let y = row as f32 * 19.0;
        leaf(world, nav, rect(0.0, y, 14.0, 9.0), Skeleton::new().radius(4.0));
        leaf(world, nav, rect(24.0, y + 1.0, 67.0 - (row % 3) as f32 * 9.0, 6.0), Skeleton::text());
    }
    caption(world, sidebar, assets, colors, "Made for your next\nfavorite discovery.", 13.0, true, rect(9.0, 80.0, 82.0, 15.0));

    let music = panel(world, root, colors, rect(23.0, 17.0, 45.0, 31.0), BackdropRole::Card);
    caption(world, music, assets, colors, "01 / ON REPEAT", 12.0, true, rect(5.0, 6.0, 90.0, 8.0));
    let tracks = group(world, music, "Loading four music tracks", rect(5.0, 23.0, 90.0, 72.0));
    for row in 0..4 {
        let y = row as f32 * 25.0;
        leaf(world, tracks, rect(0.0, y, 9.0, 20.0), Skeleton::new().radius(6.0));
        leaf(world, tracks, rect(13.0, y + 1.0, 50.0 - row as f32 * 4.0, 6.0), Skeleton::text());
        leaf(world, tracks, rect(13.0, y + 12.0, 30.0, 4.0), Skeleton::text());
        leaf(world, tracks, rect(88.0, y + 7.0, 10.0, 5.0), Skeleton::text());
    }

    let albums = panel(world, root, colors, rect(23.0, 51.0, 45.0, 33.0), BackdropRole::Card);
    caption(world, albums, assets, colors, "02 / FRESH FINDS", 12.0, true, rect(5.0, 6.0, 90.0, 8.0));
    let covers = group(world, albums, "Loading four albums", rect(5.0, 24.0, 90.0, 69.0));
    for column in 0..4 {
        let x = column as f32 * 25.5;
        leaf(world, covers, rect(x, 0.0, 22.0, 67.0), Skeleton::new().radius(10.0));
        leaf(world, covers, rect(x, 76.0, 21.0, 7.0), Skeleton::text());
        leaf(world, covers, rect(x, 91.0, 15.0, 5.0), Skeleton::text());
    }

    let profile = panel(world, root, colors, rect(70.0, 17.0, 27.0, 18.0), BackdropRole::Card);
    caption(world, profile, assets, colors, "03 / AVATAR + TEXT LINES", 11.0, true, rect(7.0, 10.0, 90.0, 13.0));
    let bio = group(world, profile, "Loading listener profile", rect(7.0, 36.0, 86.0, 58.0));
    leaf(world, bio, rect(0.0, 0.0, 20.0, 70.0), Skeleton::circle(42.0));
    // Exercise the public text-line builder as well as individual text leaves.
    let text_parent = world.spawn((ChildOf(bio), rect(27.0, 0.0, 70.0, 100.0), Pickable::IGNORE)).id();
    let lines = Skeleton::text_lines(3).label("Loading listener biography")
        .width(percent(100)).gap(7.0).final_width(percent(58))
        .line(Skeleton::text().width(percent(100)).height(px(8)));
    let mut commands = world.commands();
    let lines_entity = lines.spawn(&mut commands);
    commands.entity(lines_entity).insert(ChildOf(text_parent));
    world.flush();
    // Mark builder-generated leaves for the fixture count, without assuming its
    // implementation always has a particular child order.
    let children: Vec<Entity> = world.get::<Children>(lines_entity)
        .map(|children| children.iter().collect()).unwrap_or_default();
    for child in children {
        if world.get::<Skeleton>(child).is_some() { world.entity_mut(child).insert(SkeletonDemoLeaf); }
    }

    let variants = panel(world, root, colors, rect(70.0, 38.0, 27.0, 21.0), BackdropRole::Card);
    let samples = group(world, variants, "Loading animation variants", rect(7.0, 8.0, 86.0, 86.0));
    for (row, (title, skeleton)) in [
        ("LTR / 1.5s", Skeleton::new()),
        ("RTL / 1.5s", Skeleton::new().direction(SkeletonDirection::RightToLeft)),
        ("2.8s / phase .35", Skeleton::new().duration(2.8).phase(0.35)),
        ("Disabled / static", Skeleton::new().enabled(false)),
    ].into_iter().enumerate() {
        caption(world, samples, assets, colors, title, 11.0, true, rect(0.0, row as f32 * 25.0, 59.0, 17.0));
        leaf(world, samples, rect(62.0, row as f32 * 25.0 + 1.0, 38.0, 12.0), skeleton);
    }

    let stress = panel(world, root, colors, rect(70.0, 62.0, 27.0, 22.0), BackdropRole::Card);
    caption(world, stress, assets, colors, "CLIP / SCROLL / TRANSFORM", 11.0, true, rect(7.0, 7.0, 90.0, 12.0));
    let mut viewport = rect(7.0, 28.0, 40.0, 61.0);
    viewport.overflow = Overflow::scroll_y();
    let scroll = panel(world, stress, colors, viewport, BackdropRole::Inset);
    world.entity_mut(scroll).insert((ScrollPosition(Vec2::new(0.0, 24.0)), SkeletonGroup::new("Loading clipped scrolling rows")));
    let content = world.spawn((ChildOf(scroll), Node {
        width: percent(100), height: px(220), min_height: px(220), flex_shrink: 0.0,
        ..default()
    }, Pickable::IGNORE)).id();
    for row in 0..8 {
        leaf(world, content, rect(7.0, row as f32 * 12.5, 86.0, 8.0), Skeleton::new().radius(4.0));
    }
    let mut clip = rect(53.0, 28.0, 40.0, 61.0);
    clip.overflow = Overflow::clip();
    let clipped = panel(world, stress, colors, clip, BackdropRole::Inset);
    world.entity_mut(clipped).insert(SkeletonGroup::new("Loading overlapping transformed shapes"));
    let transformed = world.spawn((ChildOf(clipped), rect(8.0, 8.0, 90.0, 90.0), Pickable::IGNORE,
        UiTransform { rotation: Rot2::degrees(-13.0), scale: Vec2::new(1.18, 0.82), ..default() },
    )).id();
    leaf(world, transformed, rect(-15.0, 8.0, 100.0, 46.0), Skeleton::new().radius(9.0));
    let overlap = leaf(world, transformed, rect(20.0, 38.0, 100.0, 48.0),
        Skeleton::new().direction(SkeletonDirection::RightToLeft).phase(0.5).radius(9.0));
    world.entity_mut(overlap).insert((ZIndex(2), UiTransform {
        rotation: Rot2::degrees(24.0), scale: Vec2::new(0.9, 1.15), ..default()
    }));

    let player = panel(world, root, colors, rect(3.0, 87.0, 94.0, 10.0), BackdropRole::Card);
    let playback = group(world, player, "Loading current player", rect(2.0, 16.0, 96.0, 68.0));
    leaf(world, playback, rect(0.0, 0.0, 4.0, 95.0), Skeleton::new().radius(6.0));
    leaf(world, playback, rect(6.0, 12.0, 15.0, 20.0), Skeleton::text());
    leaf(world, playback, rect(6.0, 56.0, 10.0, 15.0), Skeleton::text());
    leaf(world, playback, rect(29.0, 23.0, 3.0, 55.0), Skeleton::new().radius(99.0));
    leaf(world, playback, rect(36.0, 37.0, 40.0, 17.0), Skeleton::new().radius(99.0));
    leaf(world, playback, rect(85.0, 37.0, 12.0, 17.0), Skeleton::new().radius(99.0));
    // Small semantic accent, not an additional skeleton or custom material.
    panel(world, player, colors, rect(0.0, 22.0, 0.3, 56.0), BackdropRole::Accent);
    scroll
}