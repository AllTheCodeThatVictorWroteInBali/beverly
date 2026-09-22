//! Opt-in, bounded native skeleton counters; this module does not spawn a fixture.
//!
//! Parent integration: declare `pub(crate) mod skeleton_metrics;` in rendering/mod.rs
//! and call `super::skeleton_metrics::build(app);` at the end of
//! UiRenderingPlugin::build, after UiMaterialPlugin<UiShapeMaterial> and DefaultPlugins.
//!
//! Enabled only by valid UI_SKELETON_NODES (1/10/100/500/1000) or
//! UI_SKELETON_DEMO=1. Positive UI_AUDIT_FRAMES and UI_AUDIT_WARMUP use the audit's
//! window: first Ready + default-camera frame is a baseline, then warmup, then
//! measured frames. Without a positive audit frame limit, default to 60 warmup
//! and 60 measured frames. No exit, screenshot, fixture, or render-pass changes.
//!
//! Main-world counts include hidden application Skeletons. Native extraction is
//! observed separately, after extract_ui_material_nodes<UiShapeMaterial>, before
//! Prepare clears its vector. Prepared TransparentUi batches are nonempty
//! batch_range entries after ALL Prepare systems, across all UI views. These are
//! scheduled draw batches, NOT proof of submitted/successful GPU draw calls.
//! Type filtering uses the registered DrawUiMaterial<UiShapeMaterial> ID, never
//! shader names or the global pipeline-cache size. Shared/mixed batches are not
//! attributed to individual Skeleton entities.
//!
//! Exactly one final report is emitted from main-world Last, even if the audit
//! exits that frame. Render work can lag the main world: report its independent
//! sample count and pending/unobserved frames, never fill missing frames with zero
//! or wait for the renderer. The closing main frame is normally not rendered yet.
//! This avoids relying on screenshot drain frames or rendering after AppExit.
//!
//! Exact sync duration requires instrumentation INSIDE the relevant function.
//! Optional SkeletonSyncCpuMetrics hooks are provided below; no system-to-system
//! Instant bracket is mislabeled as CPU sync cost. No GPU timer is installed:
//! Bevy 0.19.0 RenderDiagnosticsPlugin documents timestamp queries only on Vulkan
//! and DX12 (Metal records CPU only), and existing capture passes would need
//! diagnostic spans to attribute even a whole UI pass, let alone Skeletons.

use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
    time::Duration,
};

use bevy::{
    asset::AssetEventSystems,
    prelude::*,
    render::{
        Extract, ExtractSchedule, Render, RenderApp, RenderSystems,
        render_phase::{DrawFunctions, ViewSortedRenderPhases},
        render_resource::{CachedRenderPipelineId, PipelineCache},
    },
    ui::IsDefaultUiCamera,
    ui_render::{
        DrawUiMaterial, ExtractedUiMaterialNodes, TransparentUi, extract_ui_material_nodes,
    },
};

use super::material::UiShapeMaterial;
use crate::animation::loading::AppState;
use crate::animation::skeleton::Skeleton;

pub(super) fn build(app: &mut App) {
    if cfg!(target_arch = "wasm32") {
        return;
    }
    let config = match Config::parse(|key| std::env::var(key).ok()) {
        Ok(Some(config)) => config,
        Ok(None) => return,
        Err(error) => {
            warn!("Skeleton metrics disabled: {error}");
            return;
        }
    };
    let shared = SharedRenderTotals::default();
    let render_app_present = app.get_sub_app_mut(RenderApp).is_some();
    app.insert_resource(MainMetrics::new(config, render_app_present))
        .insert_resource(shared.clone())
        .init_resource::<SkeletonSyncCpuMetrics>()
        .add_systems(
            PostUpdate,
            collect_main.after(AssetEventSystems).run_if(main_pending),
        )
        .add_systems(Last, report_once.run_if(main_pending));
    if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
        render_app
            .insert_resource(shared)
            .init_resource::<ExtractedFrame>()
            .add_systems(
                ExtractSchedule,
                extract_frame.after(extract_ui_material_nodes::<UiShapeMaterial>),
            )
            .add_systems(
                Render,
                collect_prepared
                    .after(RenderSystems::Prepare)
                    .before(RenderSystems::Render),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Config {
    warmup: usize,
    frames: usize,
}

impl Config {
    fn parse(mut env: impl FnMut(&str) -> Option<String>) -> Result<Option<Self>, String> {
        let nodes = env("UI_SKELETON_NODES");
        if nodes.is_none() && env("UI_SKELETON_DEMO").as_deref() != Some("1") {
            return Ok(None);
        }
        if let Some(nodes) = nodes {
            if !matches!(number("UI_SKELETON_NODES", &nodes)?, 1 | 10 | 100 | 500 | 1000) {
                return Err("UI_SKELETON_NODES must be 1, 10, 100, 500 or 1000".into());
            }
        }
        let frames = env("UI_AUDIT_FRAMES")
            .map(|value| number("UI_AUDIT_FRAMES", &value))
            .transpose()?
            .filter(|frames| *frames > 0);
        let warmup = env("UI_AUDIT_WARMUP")
            .map(|value| number("UI_AUDIT_WARMUP", &value))
            .transpose()?
            .unwrap_or_else(|| frames.map_or(60, |frames| (frames / 4).min(120)));
        let frames = match frames {
            Some(frames) => frames,
            None => warmup.checked_add(60).ok_or("warmup + 60 overflows usize")?,
        };
        if warmup >= frames {
            return Err("UI_AUDIT_WARMUP must be less than UI_AUDIT_FRAMES".into());
        }
        Ok(Some(Self { warmup, frames }))
    }
}

fn number(key: &str, value: &str) -> Result<usize, String> {
    let value = value.trim();
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!("{key} must be a non-negative integer"));
    }
    value.parse().map_err(|_| format!("{key} is too large"))
}

/// Constant-space statistics. None means no observations, not a measured zero.
#[derive(Default)]
struct Counts {
    samples: usize,
    total: u128,
    min: usize,
    max: usize,
    last: usize,
}

impl Counts {
    fn push(&mut self, value: usize) {
        self.min = if self.samples == 0 { value } else { self.min.min(value) };
        self.max = self.max.max(value);
        self.last = value;
        self.total += value as u128;
        self.samples += 1;
    }

    fn summary(&self) -> Option<(f64, usize, usize, usize)> {
        (self.samples > 0).then(|| {
            (self.total as f64 / self.samples as f64, self.min, self.max, self.last)
        })
    }
}

#[derive(Default)]
struct CpuFrame {
    elapsed: Duration,
    calls: usize,
}

#[derive(Default)]
struct CpuTotals {
    elapsed: Duration,
    calls: usize,
    instrumented_frames: usize,
}

impl CpuTotals {
    fn push(&mut self, frame: CpuFrame) {
        self.elapsed += frame.elapsed;
        self.calls += frame.calls;
        self.instrumented_frames += usize::from(frame.calls > 0);
    }

    // Never interpret a missing hook as a zero-cost function.
    fn summary(&self) -> Option<(f64, usize, usize)> {
        (self.instrumented_frames > 0).then(|| {
            (self.elapsed.as_secs_f64() * 1_000_000.0 / self.instrumented_frames as f64,
                self.instrumented_frames, self.calls)
        })
    }
}

/// Optional inside-function wall-duration hooks; resource exists only when enabled.
/// Parent may add Option<ResMut<SkeletonSyncCpuMetrics>> to a sync function, obtain
/// Instant::now() ONLY when that option is Some, then record elapsed at function
/// exit (including early returns). No timer is installed by this module.
///
/// `record_skeleton_sync` is for sync_skeletons only, not sync_groups/layout.
/// `record_surface_sync` is for the entire sync_surface_materials function:
/// it covers ALL Surfaces, including hidden application nodes, not just Skeletons.
/// Durations exclude ECS parameter acquisition and later deferred command work;
/// they are elapsed CPU-side function-body wall time, not OS thread CPU time.
#[derive(Resource, Default)]
pub(crate) struct SkeletonSyncCpuMetrics {
    skeleton: CpuFrame,
    surface: CpuFrame,
}

impl SkeletonSyncCpuMetrics {
    #[allow(dead_code)] // Parent instrumentation is deliberately optional.
    pub(crate) fn record_skeleton_sync(&mut self, elapsed: Duration) {
        self.skeleton.elapsed += elapsed;
        self.skeleton.calls += 1;
    }

    #[allow(dead_code)]
    pub(crate) fn record_surface_sync(&mut self, elapsed: Duration) {
        self.surface.elapsed += elapsed;
        self.surface.calls += 1;
    }
}

#[derive(Resource)]
struct MainMetrics {
    config: Config,
    render_app_present: bool,
    frame: Option<usize>,
    measured_frame: Option<usize>,
    reported: bool,
    handles: HashSet<AssetId<UiShapeMaterial>>,
    previous_handles: HashSet<AssetId<UiShapeMaterial>>,
    modified_ids: HashSet<AssetId<UiShapeMaterial>>,
    skeletons: Counts,
    visible_nonempty: Counts,
    material_nodes: Counts,
    unique_handles: Counts,
    modified_events: Counts,
    modified_unique: Counts,
    skeleton_cpu: CpuTotals,
    surface_cpu: CpuTotals,
}

impl MainMetrics {
    fn new(config: Config, render_app_present: bool) -> Self {
        Self {
            config, render_app_present, frame: None, measured_frame: None, reported: false,
            handles: default(), previous_handles: default(), modified_ids: default(),
            skeletons: default(), visible_nonempty: default(), material_nodes: default(),
            unique_handles: default(), modified_events: default(), modified_unique: default(),
            skeleton_cpu: default(), surface_cpu: default(),
        }
    }

    fn advance(&mut self) {
        let frame = self.frame.map_or(0, |previous| previous + 1);
        self.frame = Some(frame);
        self.measured_frame = (frame > self.config.warmup && frame <= self.config.frames)
            .then(|| frame - self.config.warmup);
    }
}

fn main_pending(state: Res<MainMetrics>) -> bool { !state.reported }

fn is_skeleton_modification(
    event: &AssetEvent<UiShapeMaterial>,
    current: &HashSet<AssetId<UiShapeMaterial>>,
    previous: &HashSet<AssetId<UiShapeMaterial>>,
) -> Option<AssetId<UiShapeMaterial>> {
    match event {
        AssetEvent::Modified { id } if current.contains(id) || previous.contains(id) => Some(*id),
        _ => None,
    }
}

#[allow(clippy::type_complexity)]
fn collect_main(
    mut state: ResMut<MainMetrics>,
    mut cpu: ResMut<SkeletonSyncCpuMetrics>,
    mut events: MessageReader<AssetEvent<UiShapeMaterial>>,
    app_state: Option<Res<State<AppState>>>,
    cameras: Query<Entity, (With<Camera>, With<IsDefaultUiCamera>)>,
    skeletons: Query<(
        Option<&MaterialNode<UiShapeMaterial>>,
        Option<&InheritedVisibility>,
        Option<&ComputedNode>,
    ), With<Skeleton>>,
) {
    let cpu = std::mem::take(&mut *cpu);
    if state.frame.is_none()
        && (app_state.as_ref().map(|state| state.get()) != Some(&AppState::Ready)
            || cameras.single().is_err())
    {
        events.clear();
        return;
    }
    state.advance();
    // Reuse allocations and retain previous-frame IDs to count changes to a
    // handle detached/despawned/replaced during this frame. Not all shape assets.
    let state = &mut *state;
    std::mem::swap(&mut state.handles, &mut state.previous_handles);
    state.handles.clear();
    state.modified_ids.clear();
    let (mut count, mut visible, mut material_nodes) = (0, 0, 0);
    for (material, visibility, computed) in &skeletons {
        count += 1;
        visible += usize::from(visibility.is_some_and(|visibility| visibility.get())
            && computed.is_some_and(|node| !node.is_empty()));
        if let Some(material) = material {
            material_nodes += 1;
            state.handles.insert(material.id());
        }
    }
    let mut modifications = 0;
    for event in events.read() {
        if let Some(id) = is_skeleton_modification(event, &state.handles, &state.previous_handles) {
            modifications += 1;
            state.modified_ids.insert(id);
        }
    }
    // Drain warmup/baseline messages without counting them. Added/Removed/Unused
    // events and unrelated UiShapeMaterial Modified events never count as writes.
    if state.measured_frame.is_none() { return; }
    state.skeletons.push(count);
    state.visible_nonempty.push(visible);
    state.material_nodes.push(material_nodes);
    state.unique_handles.push(state.handles.len());
    state.modified_events.push(modifications);
    state.modified_unique.push(state.modified_ids.len());
    state.skeleton_cpu.push(cpu.skeleton);
    state.surface_cpu.push(cpu.surface);
}

#[derive(Resource, Default)]
struct ExtractedFrame {
    sample: Option<usize>,
    native_skeletons: Option<usize>,
    handles: HashSet<AssetId<UiShapeMaterial>>,
}

fn extract_frame(
    mut frame: ResMut<ExtractedFrame>,
    state: Extract<Res<MainMetrics>>,
    skeletons: Extract<Query<(), With<Skeleton>>>,
    nodes: Option<Res<ExtractedUiMaterialNodes<UiShapeMaterial>>>,
) {
    frame.sample = if state.reported { None } else { state.measured_frame };
    if frame.sample.is_none() { return; }
    frame.handles.clear();
    frame.native_skeletons = nodes.map(|nodes| {
        let mut count = 0;
        for node in &nodes.uinodes {
            if skeletons.contains(node.main_entity.id()) {
                count += 1;
                frame.handles.insert(node.material);
            }
        }
        count
    });
}

#[derive(Resource, Clone, Default)]
struct SharedRenderTotals(Arc<Mutex<RenderTotals>>);

#[derive(Default)]
struct RenderTotals {
    closed: bool,
    last_sample: usize,
    observed_frames: usize,
    phases_missing: usize,
    extracted_skeletons: Counts,
    extracted_handles: Counts,
    ui_batches: Counts,
    shape_batches: Counts,
    shape_ready_batches: Counts,
    shape_pipeline_count: Counts,
    shape_pipeline_ids: HashSet<CachedRenderPipelineId>,
}

fn collect_prepared(
    frame: Res<ExtractedFrame>,
    shared: Res<SharedRenderTotals>,
    phases: Option<Res<ViewSortedRenderPhases<TransparentUi>>>,
    draws: Option<Res<DrawFunctions<TransparentUi>>>,
    cache: Option<Res<PipelineCache>>,
    mut ids: Local<HashSet<CachedRenderPipelineId>>,
) {
    let Some(sample) = frame.sample else { return; };
    // All scanning stays outside the cross-world lock. No GPU readback or wait.
    let shape_draw = draws.and_then(|draws| draws.read().get_id::<DrawUiMaterial<UiShapeMaterial>>());
    ids.clear();
    let (mut ui_batches, mut shape_batches, mut ready_batches) = (0, 0, 0);
    if let Some(phases) = phases.as_ref() {
        for phase in phases.values() {
            for item in phase.items.values() {
                let batch = !item.batch_range.is_empty();
                ui_batches += usize::from(batch);
                if Some(item.draw_function) == shape_draw {
                    // Include queued-but-clipped/not-ready items in the pipeline
                    // set, but NOT empty tails or capture sentinels in batch counts.
                    ids.insert(item.pipeline);
                    shape_batches += usize::from(batch);
                    ready_batches += usize::from(batch && cache.as_ref()
                        .is_some_and(|cache| cache.get_render_pipeline(item.pipeline).is_some()));
                }
            }
        }
    }
    let mut totals = shared.0.lock().unwrap_or_else(|error| error.into_inner());
    if totals.closed || sample <= totals.last_sample { return; }
    totals.last_sample = sample;
    totals.observed_frames += 1;
    if let Some(count) = frame.native_skeletons {
        totals.extracted_skeletons.push(count);
        totals.extracted_handles.push(frame.handles.len());
    }
    if phases.is_none() {
        totals.phases_missing += 1;
        return;
    }
    totals.ui_batches.push(ui_batches);
    if shape_draw.is_some() {
        totals.shape_batches.push(shape_batches);
        totals.shape_pipeline_count.push(ids.len());
        totals.shape_pipeline_ids.extend(ids.iter().copied());
        if cache.is_some() { totals.shape_ready_batches.push(ready_batches); }
    }
}

fn report_once(
    mut commands: Commands,
    mut state: ResMut<MainMetrics>,
    shared: Res<SharedRenderTotals>,
) {
    if state.frame != Some(state.config.frames) { return; }
    state.reported = true;
    // Optional parent timers also become inert after the bounded measurement.
    commands.remove_resource::<SkeletonSyncCpuMetrics>();
    let mut render = shared.0.lock().unwrap_or_else(|error| error.into_inner());
    render.closed = true;
    let pending = state.skeletons.samples.saturating_sub(render.observed_frames);
    // Stable formatting; IDs identify this process's observed cached pipelines,
    // not stable shader identities or every entry in SpecializedRenderPipelines.
    let mut pipeline_ids: Vec<_> = render.shape_pipeline_ids.iter()
        .map(|id| format!("{id:?}")).collect();
    pipeline_ids.sort();
    info!(
        "Skeleton metrics FINAL: warmup={} main_samples={} render_app={} render_samples={} render_pending_or_unobserved={} phases_missing={}; \
         counts=(avg,min,max,last), None=unavailable: main_all_skeletons={:?} main_visible_nonempty_candidates={:?} \
         main_material_nodes={:?} main_unique_handles={:?} skeleton_handle_modified_events={:?} modified_unique_handles={:?} \
         modified_events_total={}; native_extracted_skeletons={:?} native_extracted_unique_handles={:?} extraction_samples={}; \
         prepared_all_TransparentUi_batches={:?} ui_phase_samples={} prepared_UiShapeMaterial_batches={:?} shape_samples={} \
         shape_pipeline_ready_batches={:?} pipeline_cache_samples={} shape_cached_pipeline_count={:?} shape_cached_pipeline_ids={:?}; \
         cpu=(mean_us_per_instrumented_frame,frames,calls): skeleton_sync={:?} all_surface_sync={:?}; \
         GPU timing NOT IMPLEMENTED; absent CPU hook is NOT zero cost. Main counts include hidden app nodes; \
         batches cover all UI views, not skeleton-only draws; pipeline-ready does not prove submission. \
         Modified counts are published asset events for current/previous-frame skeleton handles, not GPU uploads or unique property writes. \
         Render samples may lag at shutdown; no missing samples imputed. Metrics scanning/locking adds profiling overhead.",
        state.config.warmup, state.skeletons.samples, state.render_app_present,
        render.observed_frames, pending, render.phases_missing,
        state.skeletons.summary(), state.visible_nonempty.summary(),
        state.material_nodes.summary(), state.unique_handles.summary(),
        state.modified_events.summary(), state.modified_unique.summary(), state.modified_events.total,
        render.extracted_skeletons.summary(), render.extracted_handles.summary(), render.extracted_skeletons.samples,
        render.ui_batches.summary(), render.ui_batches.samples, render.shape_batches.summary(), render.shape_batches.samples,
        render.shape_ready_batches.summary(), render.shape_ready_batches.samples,
        render.shape_pipeline_count.summary(), pipeline_ids,
        state.skeleton_cpu.summary(), state.surface_cpu.summary(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(values: &[(&str, &str)]) -> Result<Option<Config>, String> {
        Config::parse(|key| values.iter().find(|(name, _)| *name == key)
            .map(|(_, value)| (*value).to_owned()))
    }

    #[test]
    fn disabled_even_with_audit_or_malformed_unrelated_settings() {
        assert_eq!(config(&[("UI_AUDIT_FRAMES", "bad")]).unwrap(), None);
        assert_eq!(config(&[("UI_SKELETON_DEMO", "0")]).unwrap(), None);
    }

    #[test]
    fn supports_every_dense_grid_size_and_manual_default() {
        for nodes in ["1", "10", "100", "500", "1000"] {
            assert_eq!(config(&[("UI_SKELETON_NODES", nodes)]).unwrap(),
                Some(Config { warmup: 60, frames: 120 }));
        }
        assert_eq!(config(&[("UI_SKELETON_DEMO", "1"), ("UI_AUDIT_FRAMES", "0")]).unwrap(),
            Some(Config { warmup: 60, frames: 120 }));
    }

    #[test]
    fn invalid_nodes_do_not_fall_back_to_demo() {
        assert!(config(&[("UI_SKELETON_NODES", "2"), ("UI_SKELETON_DEMO", "1")]).is_err());
        assert!(config(&[("UI_SKELETON_NODES", "-1")]).is_err());
        assert!(config(&[("UI_SKELETON_NODES", "")]).is_err());
    }

    #[test]
    fn audit_window_matches_existing_defaults_and_rejects_empty_measurement() {
        assert_eq!(config(&[("UI_SKELETON_DEMO", "1"), ("UI_AUDIT_FRAMES", "180")]).unwrap(),
            Some(Config { warmup: 45, frames: 180 }));
        assert!(config(&[("UI_SKELETON_DEMO", "1"), ("UI_AUDIT_FRAMES", "60"),
            ("UI_AUDIT_WARMUP", "60")]).is_err());
    }

    #[test]
    fn baseline_and_warmup_are_not_samples() {
        let mut state = MainMetrics::new(Config { warmup: 2, frames: 4 }, false);
        for expected in [None, None, None, Some(1), Some(2)] {
            state.advance();
            assert_eq!(state.measured_frame, expected);
        }
        assert_eq!(state.frame, Some(4));
    }

    #[test]
    fn count_summary_distinguishes_missing_from_zero() {
        let mut counts = Counts::default();
        assert_eq!(counts.summary(), None);
        counts.push(0);
        counts.push(4);
        assert_eq!(counts.summary(), Some((2.0, 0, 4, 4)));
    }

    #[test]
    fn only_tracked_modified_events_count_including_detached_previous_handle() {
        let id = Handle::<UiShapeMaterial>::default().id();
        let empty = HashSet::new();
        let tracked = HashSet::from([id]);
        assert_eq!(is_skeleton_modification(&AssetEvent::Modified { id }, &tracked, &empty), Some(id));
        assert_eq!(is_skeleton_modification(&AssetEvent::Modified { id }, &empty, &tracked), Some(id));
        assert_eq!(is_skeleton_modification(&AssetEvent::Modified { id }, &empty, &empty), None);
        assert_eq!(is_skeleton_modification(&AssetEvent::Added { id }, &tracked, &tracked), None);
        assert_eq!(is_skeleton_modification(&AssetEvent::Removed { id }, &tracked, &tracked), None);
    }

    #[test]
    fn missing_cpu_hooks_are_not_reported_as_zero_and_multiple_calls_sum() {
        let mut totals = CpuTotals::default();
        totals.push(CpuFrame::default());
        assert_eq!(totals.summary(), None);
        let mut hooks = SkeletonSyncCpuMetrics::default();
        hooks.record_skeleton_sync(Duration::from_micros(5));
        hooks.record_skeleton_sync(Duration::from_micros(15));
        totals.push(hooks.skeleton);
        let (mean, frames, calls) = totals.summary().unwrap();
        assert!((mean - 20.0).abs() < 1e-9);
        assert_eq!((frames, calls), (1, 2));
    }
}