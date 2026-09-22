//! Opt-in native smoke/stress audit. No resources or systems are installed unless
//! UI_AUDIT_FRAMES is positive. Frames start after AppState::Ready and UI camera
//! creation, not at process launch. The first Last tick establishes a baseline;
//! the requested number of subsequent Last-to-Last intervals includes warmup.
//! Screenshot readback/save drain frames are excluded from the measurements.

use std::{path::PathBuf, time::Instant};

use bevy::{
    app::AppExit,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    ui::IsDefaultUiCamera,
};

use super::{GradientStop, LinearGradient, OuterShadow, Paint, Surface, material::UiShapeMaterial};
use crate::animation::loading::AppState;
use crate::components::toggle::component::ToggleShadowMaterial;
use crate::theme::{ThemeMode, ThemeResource, dark_theme, light_theme};

pub struct UiFrameworkAuditPlugin;

impl Plugin for UiFrameworkAuditPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(target_arch = "wasm32"))]
        match AuditConfig::parse(|key| std::env::var(key).ok()) {
            Ok(Some(config)) => {
                info!("UI audit enabled: frames={} warmup={} measured={} nodes={} style={:?} screenshot={:?}; counting starts at Ready + UI camera",
                    config.frames, config.warmup, config.frames - config.warmup,
                    config.nodes, config.style, config.screenshot);
                info!("UI audit: Last-to-Last wall-clock intervals include scheduling, vsync/pacing and audit overhead; NOT GPU timing or draw calls. Stress is an overlay; media app remains active.");
                app.insert_resource(AuditState::new(config))
                    .add_systems(PreStartup, initialize_audit_theme)
                    .add_systems(Last, audit_tick);
            }
            Ok(None) => {}
            Err(error) => warn!("UI audit disabled: {error}"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum AuditStyle {
    #[default]
    Plain,
    Border,
    Gradient,
    Effects,
}

#[derive(Debug)]
struct AuditConfig {
    frames: usize,
    warmup: usize,
    nodes: usize,
    style: AuditStyle,
    screenshot: Option<PathBuf>,
    theme: Option<ThemeMode>,
}

impl AuditConfig {
    fn parse(mut env: impl FnMut(&str) -> Option<String>) -> Result<Option<Self>, String> {
        let Some(frames) = env("UI_AUDIT_FRAMES") else {
            return Ok(None);
        };
        let frames = parse_usize("UI_AUDIT_FRAMES", &frames)?;
        if frames == 0 {
            return Ok(None);
        }
        let warmup = env("UI_AUDIT_WARMUP")
            .map(|value| parse_usize("UI_AUDIT_WARMUP", &value))
            .transpose()?
            .unwrap_or((frames / 4).min(120));
        if warmup >= frames {
            return Err("UI_AUDIT_WARMUP must be less than UI_AUDIT_FRAMES".into());
        }
        let nodes = env("UI_AUDIT_NODES")
            .map(|value| parse_usize("UI_AUDIT_NODES", &value))
            .transpose()?
            .unwrap_or(0);
        if !matches!(nodes, 0 | 100 | 1000 | 10000) {
            return Err("UI_AUDIT_NODES must be 0, 100, 1000 or 10000".into());
        }
        let style = match env("UI_AUDIT_STYLE").as_deref().map(str::trim) {
            None | Some("plain") => AuditStyle::Plain,
            Some("border") => AuditStyle::Border,
            Some("gradient") => AuditStyle::Gradient,
            Some("effects") => AuditStyle::Effects,
            _ => return Err("UI_AUDIT_STYLE must be plain, border, gradient or effects".into()),
        };
        let screenshot = env("UI_AUDIT_SCREENSHOT")
            .filter(|value| !value.trim().is_empty())
            .map(PathBuf::from);
        let theme = match env("UI_AUDIT_THEME").as_deref().map(str::trim) {
            None => None,
            Some("dark") => Some(ThemeMode::Dark),
            Some("light") => Some(ThemeMode::Light),
            _ => return Err("UI_AUDIT_THEME must be light or dark".into()),
        };
        Ok(Some(Self {
            frames,
            warmup,
            nodes,
            style,
            screenshot,
            theme,
        }))
    }
}

fn parse_usize(key: &str, value: &str) -> Result<usize, String> {
    let value = value.trim();
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!("{key} must be a non-negative integer"));
    }
    value.parse().map_err(|_| format!("{key} is too large"))
}

#[derive(Resource)]
struct AuditState {
    config: AuditConfig,
    previous: Option<Instant>,
    samples_ms: Vec<f64>,
    reported: bool,
    drain_frames: usize,
    drain_started: Option<Instant>,
    screenshot_result: Option<bool>,
}

impl AuditState {
    fn new(config: AuditConfig) -> Self {
        Self {
            config,
            previous: None,
            samples_ms: Vec::new(),
            reported: false,
            drain_frames: 0,
            drain_started: None,
            screenshot_result: None,
        }
    }
}

#[derive(Component)]
struct AuditCell;

// Audit theme selection is an initial condition, not a runtime toggle. Run
// after plugin/resource construction (main explicitly inserts the light theme)
// but before Startup and OnEnter(Ready) mount any UI. Do not emit ThemeChanged:
// that would unnecessarily despawn/rebuild the freshly mounted page contents.
fn initialize_audit_theme(audit: Res<AuditState>, mut theme: ResMut<ThemeResource>) {
    if let Some(mode) = audit.config.theme {
        theme.current = match mode {
            ThemeMode::Dark => dark_theme(),
            ThemeMode::Light => light_theme(),
        };
        info!("UI audit: initial theme set to {mode:?} before UI startup");
    }
}

fn audit_tick(world: &mut World) {
    world.resource_scope(|world, mut audit: Mut<AuditState>| {
        if audit.previous.is_none() {
            if world.get_resource::<State<AppState>>().map(State::get) != Some(&AppState::Ready) {
                return;
            }
            let mut cameras =
                world.query_filtered::<Entity, (With<Camera>, With<IsDefaultUiCamera>)>();
            let Ok(camera) = cameras.single(world) else {
                return;
            };
            if audit.config.nodes > 0 {
                spawn_fixture(world, camera, &audit.config);
            }
            log_counts(world, "start");
            audit.previous = Some(Instant::now());
            return;
        }

        if !audit.reported {
            let now = Instant::now();
            let previous = audit.previous.replace(now).unwrap();
            audit
                .samples_ms
                .push(now.duration_since(previous).as_secs_f64() * 1000.0);
            if audit.samples_ms.len() < audit.config.frames {
                return;
            }
            log_distribution("warmup", &audit.samples_ms[..audit.config.warmup]);
            log_distribution("measured", &audit.samples_ms[audit.config.warmup..]);
            log_counts(world, "end");
            log_process_memory();
            audit.reported = true;
            if let Some(path) = audit.config.screenshot.clone() {
                // Request only after timing ends, so readback/encoding cannot skew it.
                info!("UI audit: screenshot requested at late frame {}; saving to {} (overwrites existing file)", audit.config.frames, path.display());
                world.spawn(Screenshot::primary_window()).observe(
                    move |capture: On<ScreenshotCaptured>, mut state: ResMut<AuditState>| {
                        // Bevy delivers this event only after asynchronous GPU readback.
                        // Save in this observer and publish completion AFTER the write.
                        let result = (|| -> Result<(), String> {
                            if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                            }
                            let image = capture.image.clone().try_into_dynamic().map_err(|e| e.to_string())?;
                            image.to_rgb8().save(&path).map_err(|e| e.to_string())
                        })();
                        match &result {
                            Ok(()) => info!("UI audit: screenshot saved to {}", path.display()),
                            Err(error) => error!("UI audit: screenshot save failed: {error}"),
                        }
                        state.screenshot_result = Some(result.is_ok());
                    },
                );
                audit.drain_started = Some(Instant::now());
            } else {
                world.write_message(AppExit::Success);
            }
            return;
        }

        let Some(started) = audit.drain_started else {
            return;
        };
        audit.drain_frames += 1;
        // A minimum of two extra frames permits extraction and readback. Do not
        // assume two is sufficient: wait for the actual save result, with a bound.
        if audit.drain_frames >= 2 && audit.screenshot_result.is_some() {
            info!("UI audit: exiting after {} unmeasured screenshot drain frames", audit.drain_frames);
            world.write_message(if audit.screenshot_result == Some(true) {
                AppExit::Success
            } else {
                AppExit::error()
            });
        } else if audit.drain_frames >= 240 || started.elapsed().as_secs() >= 30 {
            error!("UI audit: screenshot timed out after {} extra frames; capture/save not confirmed", audit.drain_frames);
            world.write_message(AppExit::error());
        }
    });
}

fn spawn_fixture(world: &mut World, camera: Entity, config: &AuditConfig) {
    let columns = (config.nodes as f32).sqrt().ceil() as u16;
    let rows = config.nodes.div_ceil(columns as usize) as u16;
    world.spawn((
        Name::new("ui-audit-stress-grid"),
        UiTargetCamera(camera),
        GlobalZIndex(100_000),
        Pickable::IGNORE,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            display: Display::Grid,
            grid_template_columns: RepeatedGridTrack::flex(columns, 1.0),
            grid_template_rows: RepeatedGridTrack::flex(rows, 1.0),
            padding: UiRect::all(px(8)),
            row_gap: px(1),
            column_gap: px(1),
            ..default()
        },
        BackgroundColor(Color::srgb(0.035, 0.045, 0.065)),
    )).with_children(|parent| {
        for index in 0..config.nodes {
            let color = Color::srgb(0.15 + (index % 7) as f32 * 0.06, 0.4, 0.75);
            let plain = Surface::rounded_rect_fill(2.0, color);
            let surface = match config.style {
                AuditStyle::Plain => plain,
                AuditStyle::Border => plain.uniform_border(1.0, Color::WHITE),
                AuditStyle::Gradient => plain.fill(Paint::linear(LinearGradient::horizontal(vec![
                    GradientStop::at(0.0, color),
                    GradientStop::at(1.0, Color::srgb(0.7, 0.2, 0.55)),
                ]))),
                AuditStyle::Effects => plain.outer_shadow(
                    OuterShadow::new(Color::BLACK)
                        .with_offset(Vec2::new(1.0, 1.0))
                        .with_blur(2.0)
                        .with_opacity(0.65),
                ),
            };
            parent.spawn((
                AuditCell,
                Node {
                    min_width: px(0),
                    min_height: px(0),
                    ..default()
                },
                surface,
                Pickable::IGNORE,
            ));
        }
    });
    info!("UI audit: stress fixture spawned: {} surfaces, {}x{} grid, style={:?}", config.nodes, columns, rows, config.style);
}

fn log_counts(world: &mut World, phase: &str) {
    let entities = world.query::<Entity>().iter(world).count();
    let nodes = world.query_filtered::<Entity, With<Node>>().iter(world).count();
    let surfaces = world.query_filtered::<Entity, With<Surface>>().iter(world).count();
    let cells = world.query_filtered::<Entity, With<AuditCell>>().iter(world).count();
    let material_nodes = world.query_filtered::<Entity, With<MaterialNode<UiShapeMaterial>>>().iter(world).count();
    let shape_assets = world.get_resource::<Assets<UiShapeMaterial>>().map(Assets::len);
    let toggle_assets = world.get_resource::<Assets<ToggleShadowMaterial>>().map(Assets::len);
    let images = world.get_resource::<Assets<Image>>().map(Assets::len);
    info!("UI audit counts [{phase}] main-world: entities={entities} ui_nodes={nodes} surfaces={surfaces} audit_cells={cells} shape_material_nodes={material_nodes} shape_material_assets={shape_assets:?} toggle_material_assets={toggle_assets:?} image_assets={images:?}; asset counts are CPU registry entries, NOT GPU allocations; material counts cover named types only");
}

fn log_distribution(phase: &str, samples: &[f64]) {
    if samples.is_empty() {
        info!("UI audit frame_ms [{phase}]: samples=0");
        return;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_by(f64::total_cmp);
    let percentile = |p: f64| sorted[((sorted.len() as f64 * p).ceil() as usize).saturating_sub(1)];
    let mean = sorted.iter().sum::<f64>() / sorted.len() as f64;
    info!("UI audit frame_ms [{phase}]: samples={} min={:.3} mean={mean:.3} p50={:.3} p95={:.3} p99={:.3} max={:.3} (nearest-rank percentiles)",
        sorted.len(), sorted[0], percentile(0.5), percentile(0.95), percentile(0.99), sorted[sorted.len() - 1]);
}

fn log_process_memory() {
    // Do not infer process memory from asset sizes or claim GPU/unified memory.
    // macOS ps reports current process resident set in KiB; sample only AFTER
    // timing so spawning ps does not contaminate the measured distribution.
    #[cfg(target_os = "macos")]
    let rss_kib = std::process::Command::new("/bin/ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output().ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|output| output.trim().parse::<u64>().ok());
    #[cfg(target_os = "linux")]
    let rss_kib = std::fs::read_to_string("/proc/self/status").ok().and_then(|status| {
        let mut fields = status.lines().find(|line| line.starts_with("VmRSS:"))?.split_whitespace();
        fields.next()?;
        let value = fields.next()?.parse::<u64>().ok()?;
        (fields.next()? == "kB").then_some(value)
    });
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    let rss_kib: Option<u64> = None;
    match rss_kib.filter(|value| *value > 0) {
        Some(value) => info!("UI audit CPU/process memory: OS-reported current RSS={value} KiB; not peak, heap-only, or GPU memory"),
        None => info!("UI audit CPU/process memory: unavailable (no trustworthy RSS sample)"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(values: &[(&str, &str)]) -> Result<Option<AuditConfig>, String> {
        AuditConfig::parse(|key| {
            values
                .iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| (*value).into())
        })
    }

    #[test]
    fn disabled_ignores_secondary_options() {
        assert!(config(&[("UI_AUDIT_STYLE", "invalid")]).unwrap().is_none());
        assert!(config(&[("UI_AUDIT_FRAMES", "0"), ("UI_AUDIT_NODES", "bad")]).unwrap().is_none());
    }

    #[test]
    fn defaults_leave_measured_frames_even_for_short_runs() {
        for (frames, warmup) in [("1", 0), ("100", 25), ("600", 120)] {
            let config = config(&[("UI_AUDIT_FRAMES", frames)]).unwrap().unwrap();
            assert_eq!(config.warmup, warmup);
            assert_eq!(config.nodes, 0);
            assert_eq!(config.style, AuditStyle::Plain);
            assert!(config.screenshot.is_none());
        }
    }

    #[test]
    fn parses_all_fixture_variants_and_screenshot() {
        for nodes in ["100", "1000", "10000"] {
            for style in ["plain", "border", "gradient", "effects"] {
                let config = config(&[
                    ("UI_AUDIT_FRAMES", " 20 "),
                    ("UI_AUDIT_WARMUP", "0"),
                    ("UI_AUDIT_NODES", nodes),
                    ("UI_AUDIT_STYLE", style),
                    ("UI_AUDIT_SCREENSHOT", "out/audit.png"),
                ])
                .unwrap()
                .unwrap();
                assert_eq!(config.nodes, nodes.parse::<usize>().unwrap());
                assert_eq!(config.screenshot, Some(PathBuf::from("out/audit.png")));
            }
        }
    }

    #[test]
    fn rejects_bad_numbers_and_options() {
        for value in ["", "-1", "+2", "1.5", "abc", "999999999999999999999999999999999999"] {
            assert!(config(&[("UI_AUDIT_FRAMES", value)]).is_err());
        }
        for (key, value) in [("UI_AUDIT_WARMUP", "20"), ("UI_AUDIT_WARMUP", "21"), ("UI_AUDIT_NODES", "99"), ("UI_AUDIT_STYLE", "unknown"), ("UI_AUDIT_THEME", "unknown")] {
            assert!(config(&[("UI_AUDIT_FRAMES", "20"), (key, value)]).is_err());
        }
    }

    #[test]
    fn audit_theme_is_initialized_before_mount_without_retheme_message() {
        use crate::theme::ThemeChanged;

        #[derive(Resource)]
        struct MountedTheme(ThemeMode);

        for (override_value, initial, expected) in [
            (Some("dark"), ThemeMode::Light, ThemeMode::Dark),
            (Some(" light "), ThemeMode::Dark, ThemeMode::Light),
            (None, ThemeMode::Dark, ThemeMode::Dark),
        ] {
            let mut values = vec![("UI_AUDIT_FRAMES", "10")];
            if let Some(value) = override_value {
                values.push(("UI_AUDIT_THEME", value));
            }
            let mut app = App::new();
            app.insert_resource(AuditState::new(config(&values).unwrap().unwrap()))
                .add_message::<ThemeChanged>()
                .add_systems(PreStartup, initialize_audit_theme)
                .add_systems(Startup, |mut commands: Commands, theme: Res<ThemeResource>| {
                    commands.insert_resource(MountedTheme(theme.current.mode));
                })
                // Match main: explicit default insertion AFTER audit registration.
                .insert_resource(ThemeResource {
                    current: match initial {
                        ThemeMode::Light => light_theme(),
                        ThemeMode::Dark => dark_theme(),
                    },
                });
            app.update();
            assert_eq!(app.world().resource::<MountedTheme>().0, expected);
            assert!(app.world().resource::<Messages<ThemeChanged>>().is_empty());

            // Initialization must not overwrite subsequent user theme changes.
            app.world_mut().resource_mut::<ThemeResource>().current = light_theme();
            app.update();
            assert_eq!(app.world().resource::<ThemeResource>().current.mode, ThemeMode::Light);
        }
    }

    // These use bare ECS worlds, never a native window, renderer, or app runner.
    fn draining_world() -> World {
        let config = config(&[("UI_AUDIT_FRAMES", "1")]).unwrap().unwrap();
        let mut audit = AuditState::new(config);
        audit.previous = Some(Instant::now());
        audit.reported = true;
        audit.drain_started = Some(Instant::now());
        let mut world = World::new();
        world.insert_resource(audit);
        world.init_resource::<Messages<AppExit>>();
        world
    }

    #[test]
    fn screenshot_waits_for_save_completion_not_just_extra_frames() {
        let mut world = draining_world();
        for _ in 0..3 {
            audit_tick(&mut world);
        }
        assert!(world.resource::<Messages<AppExit>>().is_empty());
        world.resource_mut::<AuditState>().screenshot_result = Some(true);
        audit_tick(&mut world);
        assert!(matches!(world.resource_mut::<Messages<AppExit>>().drain().next(), Some(AppExit::Success)));
    }

    #[test]
    fn screenshot_failure_and_timeout_exit_instead_of_hanging() {
        let mut world = draining_world();
        world.resource_mut::<AuditState>().screenshot_result = Some(false);
        audit_tick(&mut world);
        assert!(world.resource::<Messages<AppExit>>().is_empty());
        audit_tick(&mut world);
        assert!(matches!(world.resource_mut::<Messages<AppExit>>().drain().next(), Some(AppExit::Error(_))));

        let mut world = draining_world();
        world.resource_mut::<AuditState>().drain_frames = 239;
        audit_tick(&mut world);
        assert!(matches!(world.resource_mut::<Messages<AppExit>>().drain().next(), Some(AppExit::Error(_))));
    }

    #[test]
    fn fixture_has_exact_surface_count_and_explicit_camera() {
        let mut world = World::new();
        let camera = world.spawn_empty().id();
        let config = config(&[("UI_AUDIT_FRAMES", "10"), ("UI_AUDIT_NODES", "100")])
            .unwrap().unwrap();
        spawn_fixture(&mut world, camera, &config);
        assert_eq!(world.query_filtered::<Entity, (With<AuditCell>, With<Surface>, With<Pickable>)>().iter(&world).count(), 100);
        assert_eq!(world.query::<&UiTargetCamera>().single(&world).unwrap().0, camera);
    }
}