//! One live snapshot of the native UI phase, not a second UI renderer.
//!
//! Bevy 0.19 schedules `ui_pass` in Core2d/Core3d (there is no UiPassNode).
//! Ordinary camera views retain Bevy's native pass. Capturing views render the
//! original TransparentUi phase in two ranges. An empty, foreign phase item breaks ALL native UI batchers before
//! preparation: moving the split back to an existing batch head would incorrectly
//! omit preceding backgrounds. Capture is before the first drawing of a sampling
//! Surface, including its native background/border/shadow, not after its material.
//!
//! Deliberately supports one full-primary-window SDR camera. Strictly later,
//! non-clearing overlay cameras (including the app's SVG icon proxies) are allowed
//! but never captured or given active backdrop materials. All sampling
//! surfaces share the earliest snapshot; intervening UI is NOT recaptured for
//! later glass. Native UI fragment output is straight alpha (ALPHA_BLENDING),
//! while the accumulated target contains composited/premultiplied RGB. We copy
//! it without conversion; opaque camera clear is required to keep alpha = 1.

use bevy::{
    asset::RenderAssetUsages,
    camera::{CameraMainTextureUsages, ClearColorConfig, CompositingSpace, Hdr, RenderTarget},
    core_pipeline::{schedule::{Core2d, Core2dSystems, Core3d, Core3dSystems}, upscaling::upscaling},
    ecs::{entity::EntityHashSet, schedule::{ScheduleCleanupPolicy, ScheduleLabel}},
    prelude::*,
    render::{
        camera::{CameraRenderGraph, ExtractedCamera},
        render_asset::RenderAssets,
        render_phase::{PhaseItemExtraIndex, SortedRenderPhase, ViewSortedRenderPhases},
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages, RenderPassDescriptor},
        renderer::{CurrentView, RenderContext},
        sync_world::{RenderEntity, TemporaryRenderEntity},
        texture::GpuImage,
        view::{ExtractedView, ViewTarget},
        Extract, ExtractSchedule, Render, RenderApp, RenderSystems,
    },
    ui_render::{ui_pass, TransparentUi, UiCameraView, UiViewTarget},
    window::{PrimaryWindow, WindowRef},
};

use crate::rendering::{plugin::{BackdropRuntimeSettings, BackdropSourceTexture}, surface::Surface};

#[derive(Resource, Default, Clone, PartialEq, Eq)]
pub(super) struct BackdropCaptureState {
    camera: Option<Entity>,
    surfaces: EntityHashSet,
}

impl BackdropCaptureState {
    pub(super) fn permits(&self, entity: Entity) -> bool {
        self.camera.is_some() && self.surfaces.contains(&entity)
    }
}

#[derive(Resource, Default)]
struct ExtractedCapture {
    camera: Option<Entity>,
    surfaces: EntityHashSet,
    image: Handle<Image>,
}

#[derive(Component)]
struct CaptureBoundary(usize);

pub(super) fn build(app: &mut App) {
    app.init_resource::<BackdropCaptureState>();
    if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
        render_app.init_resource::<ExtractedCapture>()
            .add_systems(ExtractSchedule, extract_capture)
            .add_systems(Render, insert_capture_boundary
                .after(RenderSystems::PhaseSort)
                .before(RenderSystems::Prepare));
    }
}

pub(super) fn finish(app: &mut App) {
    let Some(render_app) = app.get_sub_app_mut(RenderApp) else { return; };
    // Keep the automatic native system set: deleting then reusing that set leaves
    // stale keys in Bevy 0.19. Select exactly one implementation per camera view.
    render_app.world_mut().schedule_scope(Core2d, |world, schedule| {
        schedule.remove_systems_in_set(ui_pass, world, ScheduleCleanupPolicy::RemoveSystemsOnly)
            .expect("could not condition native Core2d UI pass");
        schedule.add_systems(ui_pass.run_if(bevy::ecs::schedule::common_conditions::not(capture_pass_required))
            .after(Core2dSystems::PostProcess).before(upscaling));
        schedule.add_systems(capture_ui_pass.run_if(capture_pass_required)
            .after(Core2dSystems::PostProcess).before(upscaling));
    });
    render_app.world_mut().schedule_scope(Core3d, |world, schedule| {
        schedule.remove_systems_in_set(ui_pass, world, ScheduleCleanupPolicy::RemoveSystemsOnly)
            .expect("could not condition native Core3d UI pass");
        schedule.add_systems(ui_pass.run_if(bevy::ecs::schedule::common_conditions::not(capture_pass_required))
            .after(Core3dSystems::PostProcess).before(upscaling));
        schedule.add_systems(capture_ui_pass.run_if(capture_pass_required)
            .after(Core3dSystems::PostProcess).before(upscaling));
    });
}

fn capture_pass_required(
    current: Res<CurrentView>,
    cameras: Query<&UiCameraView>,
    views: Query<(&UiViewTarget, &CaptureBoundary)>,
    capture: Res<ExtractedCapture>,
) -> bool {
    let target = cameras.get(current.0).ok()
        .and_then(|view| views.get(view.0).ok()).map(|(target, _)| target.0);
    uses_capture_pass(capture.camera, target)
}

fn uses_capture_pass(capture_camera: Option<Entity>, boundary_target: Option<Entity>) -> bool {
    capture_camera.is_some() && capture_camera == boundary_target
}

/// Run after layout and before material synchronization/asset-event publication.
#[allow(clippy::type_complexity)]
pub(super) fn prepare_live_backdrop(
    mut state: ResMut<BackdropCaptureState>,
    settings: Res<BackdropRuntimeSettings>,
    policy: Option<Res<crate::theme::AccessibilityVisualPolicyResource>>,
    mut source: ResMut<BackdropSourceTexture>,
    mut images: ResMut<Assets<Image>>,
    clear: Res<ClearColor>,
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    cameras: Query<(Entity, &Camera, &RenderTarget, Option<&Hdr>, Option<&CompositingSpace>,
        &CameraMainTextureUsages, &CameraRenderGraph)>,
    all_cameras: Query<(Entity, &Camera, &RenderTarget)>,
    surfaces: Query<(Entity, &Surface, &ComputedNode, &InheritedVisibility, &ComputedUiTargetCamera)>,
) {
    let mut next = BackdropCaptureState::default();
    let mut spec = None;
    let accessible = policy.as_ref().is_none_or(|policy| {
        !policy.current.reduced_effects && !policy.current.reduced_transparency
            && policy.current.contrast != crate::theme::AccessibilityContrastMode::High
    });
    if settings.enabled && !settings.reduced_effects && accessible
        && let Ok((window_entity, window)) = windows.single()
        && let Some((entity, camera, target, hdr, space, usages, graph)) =
            cameras.iter().filter(|(_, camera, target, ..)| camera.is_active
                && is_primary_target(target, window_entity)).min_by_key(|(_, camera, ..)| camera.order)
    {
        let window_target = is_primary_target(target, window_entity);
        // The app draws SVG icon proxies in later Camera2d passes. They must
        // remain later than capture, and their surfaces must never sample it.
        // Ambiguous camera ordering / another clearing camera disables capture.
        let safe_overlays = all_cameras.iter().all(|(other_entity, other, target)|
            !other.is_active || other_entity == entity || !is_primary_target(target, window_entity)
                || (other.order > camera.order && matches!(other.clear_color, ClearColorConfig::None)));
        let opaque_clear = match camera.clear_color {
            ClearColorConfig::Default => clear.0.alpha() == 1.0,
            ClearColorConfig::Custom(color) => color.alpha() == 1.0,
            ClearColorConfig::None => false,
        };
        let size = UVec2::new(window.physical_width(), window.physical_height());
        if safe_overlays && supported_camera(window_target, camera.viewport.is_none(), hdr.is_some(),
            space.copied(), opaque_clear, size)
            && usages.0.contains(TextureUsages::COPY_SRC)
            && (graph.0 == Core2d.intern() || graph.0 == Core3d.intern())
        {
            for (surface_entity, surface, computed, visible, target) in &surfaces {
                if target.get() == Some(entity) && visible.get()
                    && computed.size().min_element() > 0.0
                    && surface.backdrop.is_some_and(|backdrop| backdrop.is_active())
                {
                    next.surfaces.insert(surface_entity);
                }
            }
            if !next.surfaces.is_empty() {
                next.camera = Some(entity);
                // Matches extract_cameras' window target normalization. Srgb
                // compositing intentionally stores encoded RGB in a non-sRGB texture.
                let format = if space == Some(&CompositingSpace::Srgb) {
                    TextureFormat::Rgba8Unorm
                } else {
                    TextureFormat::Rgba8UnormSrgb
                };
                spec = Some((size, format));
            }
        }
    }
    if let Some((size, format)) = spec {
        let extent = Extent3d { width: size.x, height: size.y, depth_or_array_layers: 1 };
        if images.get(&source.image).is_none_or(|image|
            image.texture_descriptor.size != extent || image.texture_descriptor.format != format)
        {
            let mut image = Image::new_uninit(extent, TextureDimension::D2, format,
                RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD);
            image.texture_descriptor.label = Some("shared_ui_backdrop_capture");
            image.texture_descriptor.usage = TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING;
            image.sampler = bevy::image::ImageSampler::linear();
            // New handle on resize/format changes also invalidates material bind groups.
            source.image = images.add(image);
        }
    }
    trace!(enabled = settings.enabled, reduced_effects = settings.reduced_effects,
        cameras = all_cameras.iter().filter(|(_, camera, _)| camera.is_active).count(),
        candidates = cameras.iter().count(), surfaces = next.surfaces.len(), camera = ?next.camera,
        "backdrop capture eligibility");
    if *state != next { *state = next; }
}

fn supported_camera(window: bool, full_viewport: bool, hdr: bool,
    space: Option<CompositingSpace>, opaque_clear: bool, size: UVec2) -> bool
{
    window && full_viewport && !hdr && space != Some(CompositingSpace::Oklab)
        && opaque_clear && size.min_element() > 0
}

fn is_primary_target(target: &RenderTarget, primary: Entity) -> bool {
    match target {
        RenderTarget::Window(WindowRef::Primary) => true,
        RenderTarget::Window(WindowRef::Entity(entity)) => *entity == primary,
        _ => false,
    }
}

fn extract_capture(
    mut capture: ResMut<ExtractedCapture>,
    state: Extract<Res<BackdropCaptureState>>,
    source: Extract<Res<BackdropSourceTexture>>,
    cameras: Extract<Query<RenderEntity>>,
) {
    capture.camera = state.camera.and_then(|entity| cameras.get(entity).ok());
    capture.surfaces.clone_from(&state.surfaces);
    capture.image = source.image.clone();
}

/// Empty foreign items reset native image, slice, and UiMaterial batchers. They
/// have no geometry and render_range skips their empty batch_range. Both sides
/// still use Bevy's own vertex buffers, pipelines, bind groups and draw commands.
fn insert_barrier(phase: &mut SortedRenderPhase<TransparentUi>, index: usize, entity: Entity) {
    let first = &phase.items[index];
    let barrier = TransparentUi {
        sort_key: first.sort_key,
        entity: (entity, Entity::PLACEHOLDER.into()),
        pipeline: first.pipeline,
        draw_function: first.draw_function,
        batch_range: 0..0,
        extra_index: PhaseItemExtraIndex::None,
        index: usize::MAX,
        indexed: false,
    };
    let key = barrier.entity;
    phase.items.shift_insert(index, key, barrier);
    phase.transient_items.push(key);
}

fn insert_capture_boundary(
    mut commands: Commands,
    capture: Res<ExtractedCapture>,
    views: Query<(Entity, &ExtractedView, &UiViewTarget)>,
    mut phases: ResMut<ViewSortedRenderPhases<TransparentUi>>,
) {
    for (view_entity, view, target) in &views {
        commands.entity(view_entity).remove::<CaptureBoundary>();
        if Some(target.0) != capture.camera { continue; }
        let Some(phase) = phases.get_mut(&view.retained_view_entity) else { continue; };
        let Some(index) = phase.items.values().position(|item|
            capture.surfaces.contains(&item.entity.1.id())) else { continue; };
        let barrier = commands.spawn(TemporaryRenderEntity).id();
        insert_barrier(phase, index, barrier);
        trace!(?view_entity, target = ?target.0, index, items = phase.items.len(),
            "inserted capture boundary");
        commands.entity(view_entity).insert(CaptureBoundary(index));
    }
}

fn capture_ui_pass(
    world: &World,
    current: Res<CurrentView>,
    cameras: Query<&UiCameraView>,
    ui_views: Query<(&ExtractedView, &UiViewTarget, Option<&CaptureBoundary>)>,
    targets: Query<(&ViewTarget, &ExtractedCamera)>,
    phases: Res<ViewSortedRenderPhases<TransparentUi>>,
    capture: Res<ExtractedCapture>,
    images: Res<RenderAssets<GpuImage>>,
    mut ctx: RenderContext,
) {
    trace!(current = ?current.0, "capture UI pass entered");
    let Ok(camera_view) = cameras.get(current.0) else { return; };
    let ui_view_entity = camera_view.0;
    let Ok((view, ui_target, boundary)) = ui_views.get(ui_view_entity) else { trace!(?ui_view_entity, "UI pass missing view"); return; };
    let Ok((target, camera)) = targets.get(ui_target.0) else { trace!(?ui_view_entity, "UI pass missing target"); return; };
    let Some(phase) = phases.get(&view.retained_view_entity) else { trace!(?ui_view_entity, "UI pass missing phase"); return; };
    if phase.items.is_empty() { trace!(?ui_view_entity, "UI pass empty phase"); return; }

    let destination = images.get(&capture.image);
    trace!(boundary = boundary.map(|boundary| boundary.0), camera = ?capture.camera,
        target_camera = ?ui_target.0, source_format = ?target.main_texture().format(),
        destination_format = ?destination.map(|image| image.texture.format()),
        "backdrop capture render eligibility");
    let split = boundary.filter(|_| Some(ui_target.0) == capture.camera)
        .zip(destination).filter(|(_, image)| {
            let source = target.main_texture();
            source.size() == image.texture.size()
                && source.format() == image.texture.format()
                && source.sample_count() == 1
                && source.usage().contains(TextureUsages::COPY_SRC)
        });
    // If an image is not prepared yet, native rendering still runs. A material
    // referring to that new image is likewise deferred by Bevy's asset preparation.
    let first_end = split.map_or(phase.items.len(), |(boundary, _)| boundary.0);
    {
        // Even an empty prefix must open the attachment once, consuming its clear
        // operation before copying (otherwise the previous frame could leak in).
        let attachments = [Some(target.get_unsampled_color_attachment())];
        trace!(current = ?current.0, first_end, ops = ?attachments[0].as_ref().unwrap().ops,
            "recording UI prefix");
        let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("ui_before_backdrop"), color_attachments: &attachments,
            depth_stencil_attachment: None, timestamp_writes: None,
            occlusion_query_set: None, multiview_mask: None,
        });
        if let Some(viewport) = camera.viewport.as_ref() { pass.set_camera_viewport(viewport); }
        if let Err(error) = phase.render_range(&mut pass, world, ui_view_entity, 0..first_end) {
            error!(?error, "Error rendering native UI before backdrop capture");
        }
    }
    if let Some((boundary, image)) = split {
        let source = target.main_texture();
        ctx.command_encoder().copy_texture_to_texture(source.as_image_copy(),
            image.texture.as_image_copy(), source.size());
        trace!(boundary = boundary.0, width = source.width(), height = source.height(),
            "captured native UI backdrop");
        let attachments = [Some(target.get_unsampled_color_attachment())];
        trace!(current = ?current.0, start = boundary.0 + 1,
            ops = ?attachments[0].as_ref().unwrap().ops, "recording UI suffix");
        let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("ui_after_backdrop"), color_attachments: &attachments,
            depth_stencil_attachment: None, timestamp_writes: None,
            occlusion_query_set: None, multiview_mask: None,
        });
        if let Some(viewport) = camera.viewport.as_ref() { pass.set_camera_viewport(viewport); }
        if let Err(error) = phase.render_range(&mut pass, world, ui_view_entity, boundary.0 + 1..) {
            error!(?error, "Error rendering native UI after backdrop capture");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{math::FloatOrd, render::{render_phase::{Draw, DrawError, DrawFunctions,
        TrackedRenderPass}, render_resource::CachedRenderPipelineId}};

    struct Noop;

    #[test]
    fn capture_dispatch_keeps_ordinary_and_overlay_views_native() {
        let mut world = World::new();
        let primary = world.spawn_empty().id();
        let overlay = world.spawn_empty().id();
        assert!(!uses_capture_pass(None, None));
        assert!(!uses_capture_pass(None, Some(primary))); // stale boundary on disable
        assert!(!uses_capture_pass(Some(primary), None)); // image/boundary not ready
        assert!(!uses_capture_pass(Some(primary), Some(overlay)));
        assert!(uses_capture_pass(Some(primary), Some(primary)));
        assert!(!uses_capture_pass(None, Some(primary)));
        assert!(uses_capture_pass(Some(primary), Some(primary))); // re-enable
    }

    impl Draw<TransparentUi> for Noop {
        fn draw<'w>(&mut self, _: &'w World, _: &mut TrackedRenderPass<'w>,
            _: Entity, _: &TransparentUi) -> Result<(), DrawError> { Ok(()) }
    }

    #[test]
    fn capture_barrier_preserves_prefix_and_is_foreign_to_every_batcher() {
        let mut world = World::new();
        let entities: Vec<_> = (0..5).map(|_| world.spawn_empty().id()).collect();
        let draws = DrawFunctions::<TransparentUi>::default();
        let draw = draws.write().add(Noop);
        let mut phase = SortedRenderPhase::default();
        for (index, entity) in entities[..4].iter().enumerate() {
            phase.add_transient(TransparentUi {
                sort_key: FloatOrd(index as f32), entity: (*entity, (*entity).into()),
                pipeline: CachedRenderPipelineId::INVALID, draw_function: draw,
                batch_range: 0..0, extra_index: PhaseItemExtraIndex::None,
                index, indexed: true,
            });
        }
        insert_barrier(&mut phase, 2, entities[4]);
        assert_eq!(phase.items.len(), 5);
        assert_eq!(phase.items[0].entity.0, entities[0]);
        assert_eq!(phase.items[1].entity.0, entities[1]);
        assert_eq!(phase.items[3].entity.0, entities[2]);
        assert_eq!(phase.items[4].entity.0, entities[3]);
        let barrier = &phase.items[2];
        assert_eq!(barrier.index, usize::MAX);
        assert!(barrier.batch_range.is_empty());
        assert_eq!(phase.transient_items.last(), Some(&barrier.entity));
        // Native batchers use extracted.get(item.index).filter(entity match).
        // The sentinel MUST not resolve, even if the extracted arrays disagree
        // about which native pipeline owns the neighboring items.
        assert!(entities.get(barrier.index).is_none());
    }

    #[test]
    fn capture_rejects_unsupported_targets_and_transparent_clear() {
        let size = UVec2::new(1920, 1080);
        assert!(supported_camera(true, true, false, None, true, size));
        assert!(supported_camera(true, true, false, Some(CompositingSpace::Srgb), true, size));
        assert!(!supported_camera(false, true, false, None, true, size));
        assert!(!supported_camera(true, false, false, None, true, size));
        assert!(!supported_camera(true, true, true, None, true, size));
        assert!(!supported_camera(true, true, false, Some(CompositingSpace::Oklab), true, size));
        assert!(!supported_camera(true, true, false, None, false, size));
        assert!(!supported_camera(true, true, false, None, true, UVec2::ZERO));
    }

    /// Native GPU regression, with an offscreen camera ONLY in this test. The
    /// production eligibility policy still rejects offscreen sampling. All three
    /// quads use the same native white image, so without the sentinel Bevy merges
    /// the red prefix with green self and blue later content into one batch.
    #[test]
    #[ignore = "requires a native GPU adapter; run explicitly"]
    fn capture_native_pixels_include_prior_batch_but_exclude_self_and_later() {
        use bevy::{app::PluginsState, render::{RenderPlugin,
            gpu_readback::{Readback, ReadbackComplete}}, window::ExitCondition, winit::WinitPlugin};
        use std::time::{Duration, Instant};

        #[derive(Resource, Default)]
        struct Pixels { snapshot: Vec<u8>, output: Vec<u8> }

        let mut app = App::new();
        app.add_plugins(DefaultPlugins
            .set(WindowPlugin { primary_window: None, exit_condition: ExitCondition::DontExit, ..default() })
            .set(RenderPlugin { synchronous_pipeline_compilation: true, ..default() })
            .disable::<WinitPlugin>())
            .init_resource::<Pixels>();
        build(&mut app);
        let deadline = Instant::now() + Duration::from_secs(30);
        while app.plugins_state() != PluginsState::Ready {
            assert!(Instant::now() < deadline, "GPU initialization timed out");
            std::thread::yield_now();
        }
        app.finish();
        finish(&mut app);
        app.cleanup();

        let mut image = Image::new_target_texture(64, 4, TextureFormat::Rgba8UnormSrgb, None);
        image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
        let output = app.world_mut().resource_mut::<Assets<Image>>().add(image.clone());
        let snapshot = app.world_mut().resource_mut::<Assets<Image>>().add(image);
        let camera = app.world_mut().spawn((Camera2d, Msaa::Off, CompositingSpace::Linear,
            RenderTarget::Image(output.clone().into()), IsDefaultUiCamera)).id();
        let root = app.world_mut().spawn((Node { width: px(64.0), height: px(4.0), ..default() },
            UiTargetCamera(camera))).id();
        let mut children = Vec::new();
        for color in [Color::srgb(1.0, 0.0, 0.0), Color::srgb(0.0, 1.0, 0.0), Color::srgb(0.0, 0.0, 1.0)] {
            children.push(app.world_mut().spawn((Node { position_type: PositionType::Absolute,
                width: px(64.0), height: px(4.0), ..default() }, BackgroundColor(color), ChildOf(root))).id());
        }
        app.insert_resource(BackdropCaptureState { camera: Some(camera),
            surfaces: [children[1]].into_iter().collect() });
        app.insert_resource(BackdropSourceTexture { image: snapshot.clone() });
        app.world_mut().spawn(Readback::texture(snapshot)).observe(
            |event: On<ReadbackComplete>, mut pixels: ResMut<Pixels>| {
                pixels.snapshot.clone_from(&event.data);
            });
        app.world_mut().spawn(Readback::texture(output)).observe(
            |event: On<ReadbackComplete>, mut pixels: ResMut<Pixels>| {
                pixels.output.clone_from(&event.data);
            });
        // 64 RGBA texels is exactly the GPU's 256-byte row alignment.
        let deadline = Instant::now() + Duration::from_secs(20);
        let mut ready_frames = 0;
        while Instant::now() < deadline {
            app.update();
            let pixels = app.world().resource::<Pixels>();
            if pixels.output.len() == 1024 && pixels.output.chunks_exact(4).all(|p| p == [0, 0, 255, 255]) {
                ready_frames += 1;
                if ready_frames >= 3 {
                    assert_eq!(pixels.snapshot.len(), 1024);
                    assert!(pixels.snapshot.chunks_exact(4).all(|p| p == [255, 0, 0, 255]),
                        "snapshot must contain only the red prefix, not green self or blue later content");
                    return;
                }
            }
        }
        panic!("native UI output/readback did not become ready");
    }
}