use bevy::prelude::*;
use bevy::asset::{AssetEventSystems, embedded_asset};
use bevy::ecs::entity::{EntityHashMap, EntityHashSet};
use bevy::render::render_resource::encase::UniformBuffer;
use bevy::ui::UiSystems;
use bevy::window::PrimaryWindow;
use std::collections::{HashMap, HashSet};

use super::capture;

use super::{
    backdrop::BackdropDebugView,
    demo::{
        UiRenderingDemoConfig,
        animate_demo_border_width,
        animate_demo_gradient_rotation,
        animate_demo_radius,
        spawn_demo_if_enabled,
    },
    material::{
        BackdropSampleRegion,
        SurfaceMaterialHandle,
        UiShapeMaterial,
        UiShapeUniform,
        build_shape_uniform_with_debug_and_time,
    },
    surface::Surface,
    UiRenderDebugView,
};

#[derive(Component)]
struct SurfaceMaterialManaged;

/// Opt in to immutable material interning. Consumers must not mutate the selected
/// asset directly; change `Surface` instead. Animated noise and authored backdrops
/// conservatively stay owned. Shimmer uses GPU globals time, not material writes.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct SharedSurfaceMaterial;

/// Compare the complete deterministic GPU payload, not a lossy float hash or
/// Rust struct bytes (which contain padding). HashMap still checks byte equality
/// after a hash collision. Texture identity is part of the binding contract.
#[derive(PartialEq, Eq, Hash)]
struct SharedMaterialKey {
    uniforms: Vec<u8>,
    backdrop_texture: AssetId<Image>,
}

impl SharedMaterialKey {
    fn new(uniforms: &UiShapeUniform, texture: &Handle<Image>) -> Self {
        let mut buffer = UniformBuffer::new(Vec::new());
        buffer.write(uniforms).expect("UiShapeUniform must satisfy its GPU uniform contract");
        Self { uniforms: buffer.into_inner(), backdrop_texture: texture.id() }
    }
}

#[derive(Resource, Default)]
struct SharedMaterialCache {
    entries: HashMap<SharedMaterialKey, Handle<UiShapeMaterial>>,
    // Membership is independent of the public render/mirror components. Removing
    // the opt-in marker cannot accidentally turn a shared handle into a mutable one.
    members: EntityHashMap<AssetId<UiShapeMaterial>>,
    cached_ids: HashSet<AssetId<UiShapeMaterial>>,
    // Reused each pass: no key serialization, per-node allocations, or asset
    // writes on unchanged frames. Sweep after ALL consumers to avoid order churn.
    used: HashSet<AssetId<UiShapeMaterial>>,
}

/// Shared full-physical-target snapshot, captured before the earliest sampling
/// Surface by `capture`. Startup uses a transparent pixel, never synthetic scenery.
#[derive(Resource, Clone)]
pub struct BackdropSourceTexture {
    pub image: Handle<Image>,
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct BackdropRuntimeSettings {
    pub enabled: bool,
    pub reduced_effects: bool,
    pub backdrop_debug_view: BackdropDebugView,
    pub render_debug_view: UiRenderDebugView,
}

impl Default for BackdropRuntimeSettings {
    fn default() -> Self {
        let enabled = std::env::var("UI_BACKDROP_DISABLED")
            .map(|value| !matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "on"))
            .unwrap_or(true);

        let backdrop_debug_view = match std::env::var("UI_BACKDROP_DEBUG").ok().as_deref() {
            Some("source") => BackdropDebugView::Source,
            Some("blur") | Some("blurred") => BackdropDebugView::Blurred,
            Some("mask") => BackdropDebugView::Mask,
            _ => BackdropDebugView::Final,
        };

        let render_debug_view = UiRenderDebugView::from_env(
            std::env::var("UI_RENDER_DEBUG").ok().as_deref(),
        );

        let reduced_effects = std::env::var("UI_REDUCED_EFFECTS")
            .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "on"))
            .unwrap_or(false);

        Self {
            enabled,
            reduced_effects,
            backdrop_debug_view,
            render_debug_view,
        }
    }
}

pub struct UiRenderingPlugin;

impl Plugin for UiRenderingPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/ui_shape.wgsl");
        capture::build(app);
        super::skeleton_metrics::build(app);
        configure_surface_sync(app);
        app.init_resource::<UiRenderingDemoConfig>()
            .init_resource::<BackdropRuntimeSettings>()
            .add_plugins(UiMaterialPlugin::<UiShapeMaterial>::default())
            .add_systems(Startup, (initialize_backdrop_source_texture, spawn_demo_if_enabled))
            .add_systems(PostUpdate, super::skeleton_demo::spawn_skeleton_demo
                .before(crate::animation::skeleton::SkeletonSystems))
            .add_systems(PostUpdate, super::glass_demo::spawn_glass_demo
                .before(crate::components::play_button::PlayButtonSystems)
                .before(bevy::camera::visibility::VisibilitySystems::VisibilityPropagate))
            .add_systems(PostUpdate, capture::prepare_live_backdrop
                .after(UiSystems::PostLayout)
                .before(sync_surface_materials)
                .before(AssetEventSystems))
            .add_systems(
                Update,
                (
                    animate_demo_radius,
                    animate_demo_border_width,
                    animate_demo_gradient_rotation,
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        capture::finish(app);
    }
}

fn configure_surface_sync(app: &mut App) {
    app.init_resource::<SharedMaterialCache>();
    #[cfg(test)]
    app.init_resource::<MaterialSyncStats>();
    // Main-world PostUpdate (including deferred handle insertions) completes
    // before render extraction. Native borders/backgrounds remain Bevy-owned.
    // Consumer audit: cards/tabs explicitly supply Surface; interaction bootstrap
    // falls back to LayoutRect without it. A11y focus decorations and motion's
    // fill-alpha animation are Surface-only, so native-only nodes do not acquire
    // those shader features implicitly. SurfaceMaterialHandle has no external
    // consumer today, but remains available as a compatibility mirror.
    // Render-asset extraction consumes published AssetEvents, not just Assets.
    // Publish this pass's additions/changes in the same frame, before extraction.
    app.add_systems(
        PostUpdate,
        sync_surface_materials
            .after(UiSystems::PostLayout)
            .before(AssetEventSystems),
    );
}

/// One authority: Surface describes appearance; MaterialNode selects the rendered
/// asset. SurfaceMaterialHandle is only a compatibility mirror (or recovery hint
/// if MaterialNode was removed). Adopted materials must not be shared between
/// independently styled owned Surfaces. Opted-in consumers select immutable
/// cache entries; exiting sharing always copies rather than mutating a sibling.
#[allow(clippy::type_complexity)]
fn sync_surface_materials(
    mut commands: Commands,
    mut cpu_metrics: Option<ResMut<super::skeleton_metrics::SkeletonSyncCpuMetrics>>,
    mut materials: ResMut<Assets<UiShapeMaterial>>,
    mut shared_cache: ResMut<SharedMaterialCache>,
    backdrop_source: Res<BackdropSourceTexture>,
    settings: Res<BackdropRuntimeSettings>,
    capture: Option<Res<capture::BackdropCaptureState>>,
    policy: Option<Res<crate::theme::AccessibilityVisualPolicyResource>>,
    time: Res<Time>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut previous_window_size: Local<Option<Vec2>>,
    mut previous_policy_present: Local<bool>,
    query: Query<(
        Entity,
        Ref<Surface>,
        Option<Ref<MaterialNode<UiShapeMaterial>>>,
        Option<&SurfaceMaterialHandle>,
        Option<Ref<ComputedNode>>,
        Option<Ref<ComputedUiRenderTargetInfo>>,
        Option<Ref<UiGlobalTransform>>,
        Option<Ref<Node>>,
        Has<SurfaceMaterialManaged>,
        Has<SharedSurfaceMaterial>,
    )>,
    orphans: Query<Entity, (With<SurfaceMaterialManaged>, Without<Surface>)>,
    mut removed: (
        RemovedComponents<ComputedNode>,
        RemovedComponents<ComputedUiRenderTargetInfo>,
        RemovedComponents<UiGlobalTransform>,
        RemovedComponents<Node>,
    ),
    #[cfg(test)] mut stats: ResMut<MaterialSyncStats>,
) {
    shared_cache.used.clear();
    let cpu_start = cpu_metrics.as_ref().map(|_| std::time::Instant::now());
    for entity in &orphans {
        commands.queue(move |world: &mut World| {
            if let Ok(mut entity) = world.get_entity_mut(entity)
                && !entity.contains::<Surface>()
            {
                entity.remove::<(
                    MaterialNode<UiShapeMaterial>,
                    SurfaceMaterialHandle,
                    SurfaceMaterialManaged,
                )>();
            }
        });
    }

    let window_size = windows.single().ok().map(|window| {
        Vec2::new(window.physical_width() as f32, window.physical_height() as f32)
    });
    let window_changed = *previous_window_size != window_size;
    *previous_window_size = window_size;
    let resources_changed = settings.is_changed() || backdrop_source.is_changed()
        || *previous_policy_present != policy.is_some()
        || policy.as_ref().is_some_and(|policy| policy.is_changed())
        || capture.as_ref().is_some_and(|state| state.is_changed());
    *previous_policy_present = policy.is_some();
    let removed_inputs: EntityHashSet = removed.0.read()
        .chain(removed.1.read())
        .chain(removed.2.read())
        .chain(removed.3.read())
        .collect();

    for (entity, surface, render, mirror, computed, target, transform, node, managed, sharing) in &query {
        let handle = render.as_ref().map(|render| &render.0)
            .or_else(|| mirror.map(|mirror| &mirror.0));
        let material = handle.and_then(|handle| materials.get(handle));
        let animated = !settings.reduced_effects
            && surface.noise.is_some_and(|noise| {
                let noise = noise.sanitized();
                noise.animated && noise.strength > 1e-5 && noise.speed > 0.0
            });
        // Even disabled authored backdrops stay owned: policy/capture changes
        // must never introduce positional sampling into an interned material.
        let shareable = sharing && !surface.noise.is_some_and(|noise| noise.animated)
            && surface.backdrop.is_none();
        let was_shared = shared_cache.members.contains_key(&entity);
        let cached_handle = handle.is_some_and(|handle| shared_cache.cached_ids.contains(&handle.id()));
        let dirty = !managed
            || (shareable && !was_shared)
            || (!shareable && (was_shared || cached_handle))
            || material.is_none()
            || resources_changed
            || (window_changed && target.as_ref().is_none_or(|target| target.physical_size() == UVec2::ZERO))
            || surface.is_changed()
            || render.as_ref().is_some_and(|value| value.is_changed())
            || computed.as_ref().is_some_and(|value| value.is_changed())
            || target.as_ref().is_some_and(|value| value.is_changed())
            || transform.as_ref().is_some_and(|value| value.is_changed())
            || node.as_ref().is_some_and(|value| value.is_changed())
            || removed_inputs.contains(&entity);
        let noise_time = if animated { time.elapsed_secs() } else { 0.0 };

        let next = if dirty {
            #[cfg(test)]
            { stats.rebuilds += 1; }
            let (logical_size, scale_factor) = computed.as_ref().map(|computed| {
                let inverse = computed.inverse_scale_factor();
                let inverse = if inverse.is_finite() && inverse > 0.0 { inverse } else { 1.0 };
                (computed.size() * inverse, 1.0 / inverse)
            }).unwrap_or_else(|| (node.as_deref().map(declared_size).unwrap_or(Vec2::ZERO), 1.0));
            let mut uniforms = build_shape_uniform_with_debug_and_time(
                &surface,
                logical_size,
                scale_factor,
                // Ineligible positional effects cannot read this metadata. Use
                // one canonical region so translated skeletons can share, while
                // preserving actual geometry and linear transform columns below.
                if shareable { BackdropSampleRegion::default() } else { compute_backdrop_sample_region(
                    computed.as_deref(), target.as_deref(), transform.as_deref(), window_size,
                ) },
                settings.enabled && !settings.reduced_effects
                    && capture.as_ref().is_none_or(|state| state.permits(entity)),
                settings.backdrop_debug_view,
                settings.render_debug_view,
                noise_time,
                settings.reduced_effects,
            );
            if let Some(transform) = transform.as_deref() {
                let origin = transform.transform_point2(Vec2::ZERO);
                let x = transform.transform_point2(Vec2::X) - origin;
                let y = transform.transform_point2(Vec2::Y) - origin;
                if x.is_finite() && y.is_finite() {
                    uniforms.surface_axes = Vec4::new(x.x, x.y, y.x, y.y);
                }
            }
            uniforms.glass_debug.x = match std::env::var("UI_GLASS_DEBUG").ok().as_deref() {
                Some("refraction") => 1.0, Some("bezel") => 2.0, Some("normal") => 3.0,
                Some("fresnel") => 4.0, Some("specular") => 5.0, Some("uv") => 6.0, _ => 0.0,
            };
            if policy.as_ref().is_some_and(|policy| policy.current.reduced_motion) {
                uniforms.glass_state.y = 0.0;
            }
            if settings.reduced_effects || policy.as_ref().is_some_and(|policy| {
                policy.current.reduced_motion || policy.current.reduced_effects
            }) {
                // Apply to ANY Surface authoring shimmer, not just Skeleton.
                if matches!(&surface.fill, super::paint::Paint::Shimmer(_)) {
                    uniforms.fill_paint.kind_and_flags.w = 0.0;
                }
                if surface.border.as_ref().is_some_and(|border| {
                    matches!(&border.paint, super::paint::Paint::Shimmer(_))
                }) {
                    uniforms.border_paint.kind_and_flags.w = 0.0;
                }
            }
            // Missing/unsupported capture must remain readable, not disappear.
            if surface.backdrop.and_then(|b| b.liquid_glass).is_some_and(|g| g.enabled)
                && uniforms.glass_optics.x < 0.5
            {
                if let Some(backdrop) = surface.backdrop {
                    uniforms.fill_paint.solid_color = backdrop.tint.to_linear().to_vec4();
                    uniforms.fill_paint.solid_color.w = 1.0;
                    uniforms.fill_paint.kind_and_flags.x = 0.0;
                }
            }
            Some(uniforms)
        } else {
            // Static surfaces do not rebuild uniforms or request mutable assets.
            None
        };

        let handle = if shareable {
            let selected = if let Some(next) = next {
                #[cfg(test)]
                { stats.shared_keys += 1; }
                let key = SharedMaterialKey::new(&next, &backdrop_source.image);
                if let Some(cached) = shared_cache.entries.get(&key)
                    && materials.contains(cached.id())
                {
                    cached.clone()
                } else {
                    let selected = materials.add(UiShapeMaterial {
                        uniforms: next,
                        backdrop_texture: backdrop_source.image.clone(),
                    });
                    shared_cache.cached_ids.insert(selected.id());
                    shared_cache.entries.insert(key, selected.clone());
                    selected
                }
            } else {
                handle.expect("unchanged shared consumers have a material").clone()
            };
            shared_cache.members.insert(entity, selected.id());
            shared_cache.used.insert(selected.id());
            selected
        } else if let Some(material) = material.filter(|_| !cached_handle) {
            let handle = handle.unwrap().clone();
            let uniforms_changed = next.as_ref().is_some_and(|next| *next != material.uniforms);
            let texture_changed = material.backdrop_texture != backdrop_source.image;
            let time_changed = animated && material.uniforms.noise.params2.x != noise_time;
            if uniforms_changed || texture_changed || time_changed {
                #[cfg(test)]
                { stats.writes += 1; }
                let mut material = materials.get_mut(&handle).unwrap();
                if let Some(next) = next {
                    material.uniforms = next;
                } else if time_changed {
                    // Animation-only work touches just the clock, not paint/geometry.
                    material.uniforms.noise.params2.x = noise_time;
                }
                if texture_changed {
                    material.backdrop_texture = backdrop_source.image.clone();
                }
            }
            handle
        } else {
            let uniforms = next.unwrap_or_else(|| material.expect("existing shared asset").uniforms);
            materials.add(UiShapeMaterial {
                uniforms,
                backdrop_texture: backdrop_source.image.clone(),
            })
        };

        if !shareable {
            shared_cache.members.remove(&entity);
        }
        if !managed || render.as_ref().is_none_or(|render| render.0 != handle)
            || mirror.is_none_or(|mirror| mirror.0 != handle)
        {
            let expected_render = render.as_ref().map(|render| render.0.clone());
            commands.queue(move |world: &mut World| {
                if let Ok(mut entity) = world.get_entity_mut(entity)
                    && entity.contains::<Surface>()
                {
                    // Do not overwrite a render handle replaced after this pass.
                    if entity.get::<MaterialNode<UiShapeMaterial>>().map(|render| &render.0)
                        == expected_render.as_ref()
                    {
                        if expected_render.as_ref() != Some(&handle) {
                            entity.insert(MaterialNode(handle.clone()));
                        }
                        entity.insert((SurfaceMaterialHandle(handle), SurfaceMaterialManaged));
                    }
                }
            });
        }
    }
    let cache = &mut *shared_cache;
    cache.members.retain(|entity, _| query.contains(*entity));
    cache.entries.retain(|_, handle| cache.used.contains(&handle.id()));
    cache.cached_ids.retain(|id| cache.used.contains(id));
    // Dropping the last cache handle lets Bevy's normal asset tracking reclaim
    // the asset after component/deferred references disappear. Never forcibly
    // remove an asset: external strong handles retain their normal semantics.
    if let (Some(start), Some(metrics)) = (cpu_start, cpu_metrics.as_mut()) {
        metrics.record_surface_sync(start.elapsed());
    }
}

#[cfg(test)]
#[derive(Resource, Default)]
struct MaterialSyncStats {
    rebuilds: usize,
    writes: usize,
    shared_keys: usize,
}

fn initialize_backdrop_source_texture(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    source: Option<Res<BackdropSourceTexture>>,
) {
    if source.is_some() {
        return;
    }
    let image = images.add(Image::transparent());

    commands.insert_resource(BackdropSourceTexture { image });
}

/// UiGlobalTransform maps CENTERED physical node coordinates to top-left-origin
/// physical viewport coordinates (positive Y down). No DPI conversion, window
/// centering, Y flip, or blur padding belongs in this mapping. Blur taps, not the
/// surface UV rectangle, determine sampling support.
///
/// Compatibility metadata only: this AABB cannot encode rotation or mirroring.
/// Production backdrop sampling now uses fragment screen position directly;
/// optical displacement and vertex expansion use `surface_axes` instead.
fn compute_backdrop_sample_region(
    computed: Option<&ComputedNode>,
    target: Option<&ComputedUiRenderTargetInfo>,
    transform: Option<&UiGlobalTransform>,
    window_physical_size: Option<Vec2>,
) -> BackdropSampleRegion {
    let (Some(computed), Some(transform)) = (computed, transform) else {
        return BackdropSampleRegion::default();
    };
    let viewport = target.map(|target| target.physical_size().as_vec2())
        .filter(|size| size.min_element() > 0.0)
        .or(window_physical_size);
    let Some(viewport) = viewport.filter(|size| size.is_finite() && size.min_element() > 0.0) else {
        return BackdropSampleRegion::default();
    };
    let half = computed.size() * 0.5;
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    for corner in [Vec2::new(-half.x, -half.y), Vec2::new(half.x, -half.y),
        Vec2::new(half.x, half.y), Vec2::new(-half.x, half.y)]
    {
        let point = transform.transform_point2(corner);
        if !point.is_finite() {
            return BackdropSampleRegion::default();
        }
        min = min.min(point);
        max = max.max(point);
    }
    BackdropSampleRegion { min_uv: min / viewport, max_uv: max / viewport }
}

fn declared_size(node: &Node) -> Vec2 {
    let width = match node.width {
        Val::Px(value) => value.max(0.0),
        _ => 0.0,
    };

    let height = match node.height {
        Val::Px(value) => value.max(0.0),
        _ => 0.0,
    };

    Vec2::new(width, height)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{backdrop::Backdrop, material::UiShapeUniform, noise::Noise};
    use bevy::{math::Affine2, reflect::structs::DynamicStruct};
    use std::time::{Duration, Instant};

    fn app() -> App {
        let mut app = App::new();
        app.init_resource::<Assets<UiShapeMaterial>>()
            .init_resource::<Assets<Image>>()
            .init_resource::<Time>()
            .insert_resource(BackdropSourceTexture { image: Handle::default() })
            .insert_resource(BackdropRuntimeSettings {
                enabled: true,
                reduced_effects: false,
                backdrop_debug_view: BackdropDebugView::Final,
                render_debug_view: UiRenderDebugView::Final,
            });
        configure_surface_sync(&mut app);
        app
    }

    fn surface() -> Surface {
        Surface::rounded_rect_fill(8.0, Color::WHITE)
            .with_backdrop(Backdrop::new().with_blur(12.0))
    }

    // Bevy's derived target fields are private but exposed through reflection.
    fn target(size: UVec2) -> ComputedUiRenderTargetInfo {
        let mut target = ComputedUiRenderTargetInfo::default();
        let mut fields = DynamicStruct::default();
        fields.insert("physical_size", size);
        fields.insert("scale_factor", 2.0_f32);
        target.apply(&fields);
        target
    }

    fn spawn(app: &mut App, surface: Surface) -> Entity {
        app.world_mut().spawn((
            Node { width: px(100.0), height: px(50.0), ..default() },
            ComputedNode {
                size: Vec2::new(200.0, 100.0),
                inverse_scale_factor: 0.5,
                ..default()
            },
            UiGlobalTransform::from_xy(300.0, 200.0),
            target(UVec2::new(1000, 800)),
            surface,
        )).id()
    }

    fn handle(app: &App, entity: Entity) -> Handle<UiShapeMaterial> {
        app.world().get::<MaterialNode<UiShapeMaterial>>(entity).unwrap().0.clone()
    }

    fn uniforms(app: &App, entity: Entity) -> UiShapeUniform {
        app.world().resource::<Assets<UiShapeMaterial>>().get(&handle(app, entity)).unwrap().uniforms
    }

    fn reset_stats(app: &mut App) {
        *app.world_mut().resource_mut::<MaterialSyncStats>() = MaterialSyncStats::default();
    }

    fn tick(app: &mut App) {
        app.world_mut().resource_mut::<Time>().advance_by(Duration::from_millis(16));
        app.update();
    }

    fn shimmer_surface() -> Surface {
        let paint = super::super::Paint::Shimmer(super::super::Shimmer::new(Color::BLACK, Color::WHITE));
        Surface::rounded_rect_border(8.0, paint.clone(), 2.0, paint)
    }

    fn spawn_shared(app: &mut App, surface: Surface) -> Entity {
        let entity = spawn(app, surface);
        app.world_mut().entity_mut(entity).insert(SharedSurfaceMaterial);
        entity
    }

    #[test]
    fn shared_shimmer_counts_and_time_never_rebuild_keys_or_write_materials() {
        for count in [1, 10, 100, 500, 1000] {
            let mut app = app();
            let entities: Vec<_> = (0..count).map(|i| {
                let entity = spawn_shared(&mut app, shimmer_surface());
                // Translation is per-draw UI data, not part of this local paint.
                app.world_mut().entity_mut(entity).insert(UiGlobalTransform::from_xy(i as f32 * 10.0, 30.0));
                entity
            }).collect();
            app.update();
            let first = handle(&app, entities[0]);
            assert!(entities.iter().all(|entity| handle(&app, *entity) == first));
            assert_eq!(app.world().resource::<Assets<UiShapeMaterial>>().len(), 1, "count={count}");
            assert_eq!(app.world().resource::<SharedMaterialCache>().entries.len(), 1);
            assert_eq!(app.world().resource::<SharedMaterialCache>().members.len(), count);
            assert_eq!(app.world().resource::<MaterialSyncStats>().shared_keys, count);
            app.update(); // settle the deferred MaterialNode change tick
            let before = uniforms(&app, entities[0]);
            reset_stats(&mut app);
            for _ in 0..10 { tick(&mut app); }
            let stats = app.world().resource::<MaterialSyncStats>();
            assert_eq!((stats.rebuilds, stats.writes, stats.shared_keys), (0, 0, 0));
            assert_eq!(uniforms(&app, entities[0]), before);
            assert_eq!(app.world().resource::<Assets<UiShapeMaterial>>().len(), 1);
        }
    }

    #[test]
    fn shared_style_detaches_and_rejoins_without_mutating_siblings() {
        let mut app = app();
        let a = spawn_shared(&mut app, shimmer_surface());
        let b = spawn_shared(&mut app, shimmer_surface());
        app.update();
        app.update();
        let original = handle(&app, b);
        let original_uniforms = uniforms(&app, b);
        reset_stats(&mut app);
        app.world_mut().get_mut::<Surface>(a).unwrap().fill = Color::WHITE.into();
        app.update();
        assert_ne!(handle(&app, a), original);
        assert_eq!(handle(&app, b), original);
        assert_eq!(uniforms(&app, b), original_uniforms);
        assert_eq!(app.world().resource::<MaterialSyncStats>().writes, 0);
        assert_eq!(app.world().resource::<MaterialSyncStats>().shared_keys, 1);
        assert_eq!(app.world().resource::<SharedMaterialCache>().entries.len(), 2);
        app.world_mut().entity_mut(a).insert(shimmer_surface());
        app.update();
        assert_eq!(handle(&app, a), original);
        assert_eq!(app.world().resource::<SharedMaterialCache>().entries.len(), 1);
    }

    #[test]
    fn shared_geometry_clip_mask_transform_and_texture_are_exact() {
        let mut app = app();
        let base = spawn_shared(&mut app, shimmer_surface());
        let size = spawn_shared(&mut app, shimmer_surface());
        let clip = spawn_shared(&mut app, shimmer_surface().with_clip(super::super::mask::Clip::rounded_rect(4.0)));
        let mask = spawn_shared(&mut app, shimmer_surface().with_mask(super::super::mask::Mask::rounded_rect(3.0)));
        let rotation = spawn_shared(&mut app, shimmer_surface());
        let scale = spawn_shared(&mut app, shimmer_surface());
        let scissor = spawn_shared(&mut app, shimmer_surface());
        app.world_mut().get_mut::<ComputedNode>(size).unwrap().size.x = 222.0;
        app.world_mut().entity_mut(rotation).insert(UiGlobalTransform::from(Affine2::from_cols(
            Vec2::Y, -Vec2::X, Vec2::ZERO,
        )));
        app.world_mut().entity_mut(scale).insert(UiGlobalTransform::from(Affine2::from_scale(Vec2::splat(2.0))));
        // Native inherited scissor is renderer-owned and may differ per draw.
        app.world_mut().entity_mut(scissor).insert(CalculatedClip { clip: Rect::new(0.0, 0.0, 50.0, 50.0) });
        app.update();
        let ids: HashSet<_> = [base, size, clip, mask, rotation, scale].map(|e| handle(&app, e).id()).into_iter().collect();
        assert_eq!(ids.len(), 6);
        assert_eq!(handle(&app, scissor), handle(&app, base));
        assert_eq!(uniforms(&app, size).size_and_kind.x, 222.0);
        assert_eq!(uniforms(&app, clip).clip_radii, Vec4::splat(8.0));
        assert_eq!(uniforms(&app, mask).mask_radii, Vec4::splat(6.0));
        assert_eq!(uniforms(&app, rotation).surface_axes, Vec4::new(0.0, 1.0, -1.0, 0.0));
        assert_eq!(uniforms(&app, scale).surface_axes, Vec4::new(2.0, 0.0, 0.0, 2.0));
        let old = handle(&app, base);
        let old_uniforms = uniforms(&app, base);
        let image = app.world_mut().resource_mut::<Assets<Image>>().add(Image::default());
        app.world_mut().resource_mut::<BackdropSourceTexture>().image = image.clone();
        app.update();
        assert_ne!(handle(&app, base), old);
        let assets = app.world().resource::<Assets<UiShapeMaterial>>();
        assert_eq!(assets.get(&old).unwrap().uniforms, old_uniforms);
        assert_eq!(assets.get(&handle(&app, base)).unwrap().backdrop_texture, image);
        app.world_mut().entity_mut(rotation).remove::<UiGlobalTransform>();
        app.update();
        assert_eq!(handle(&app, rotation), handle(&app, base));
    }

    #[test]
    fn shared_keys_compare_full_bytes_even_when_hashes_collide() {
        #[derive(Default)]
        struct CollisionHasher;
        impl std::hash::Hasher for CollisionHasher {
            fn finish(&self) -> u64 { 0 }
            fn write(&mut self, _: &[u8]) {}
        }
        let mut app = app();
        let entity = spawn_shared(&mut app, shimmer_surface());
        app.update();
        let original = uniforms(&app, entity);
        let mut changed = original;
        changed.debug_view = f32::from_bits(original.debug_view.to_bits() + 1);
        let mut keys = HashMap::<_, _, std::hash::BuildHasherDefault<CollisionHasher>>::default();
        keys.insert(SharedMaterialKey::new(&original, &Handle::default()), 1);
        keys.insert(SharedMaterialKey::new(&changed, &Handle::default()), 2);
        assert_eq!(keys.len(), 2);
        assert_eq!(keys[&SharedMaterialKey::new(&original, &Handle::default())], 1);
        assert_eq!(keys[&SharedMaterialKey::new(&changed, &Handle::default())], 2);
        let image = app.world_mut().resource_mut::<Assets<Image>>().add(Image::default());
        keys.insert(SharedMaterialKey::new(&original, &image), 3);
        assert_eq!(keys.len(), 3);
    }

    #[test]
    fn shared_marker_removal_copies_and_external_cached_adoption_cannot_mutate_siblings() {
        let mut app = app();
        let a = spawn_shared(&mut app, shimmer_surface());
        let b = spawn_shared(&mut app, shimmer_surface());
        app.update();
        let original = handle(&app, b);
        let before = uniforms(&app, b);
        app.world_mut().entity_mut(a).remove::<SharedSurfaceMaterial>();
        app.update();
        assert_ne!(handle(&app, a), original);
        assert_eq!(handle(&app, b), original);
        app.world_mut().get_mut::<Surface>(a).unwrap().fill = Color::WHITE.into();
        app.update();
        assert_eq!(uniforms(&app, b), before);
        let external = spawn(&mut app, Surface::rounded_rect_fill(2.0, Color::WHITE));
        app.world_mut().entity_mut(external).insert(MaterialNode(original.clone()));
        app.update();
        assert_ne!(handle(&app, external), original);
        assert_eq!(uniforms(&app, b), before);
        assert_eq!(app.world().resource::<SharedMaterialCache>().members.len(), 1);
        app.update(); // settle the external consumer's owned handle
        // Mirror-only recovery of a cached handle must rebuild from THIS Surface,
        // even though the compatibility mirror itself is not a dirty input.
        app.world_mut().entity_mut(external).remove::<MaterialNode<UiShapeMaterial>>()
            .insert(SurfaceMaterialHandle(original.clone()));
        app.update();
        assert_ne!(handle(&app, external), original);
        assert_eq!(uniforms(&app, external).fill_paint.solid_color, Vec4::ONE);
        assert_eq!(uniforms(&app, external).corner_radii, Vec4::splat(4.0));
        assert_eq!(uniforms(&app, b), before);
    }

    #[test]
    fn shared_excludes_backdrops_and_animated_noise_and_copies_on_transition() {
        let mut app = app();
        for excluded in [surface(), shimmer_surface().with_noise(
            Noise::grain(24.0, 0.5).with_animated(true).with_speed(1.0)),
            shimmer_surface().with_noise(Noise::grain(24.0, 0.5).with_animated(true))]
        {
            let a = spawn_shared(&mut app, excluded.clone());
            let b = spawn_shared(&mut app, excluded);
            app.update();
            assert_ne!(handle(&app, a), handle(&app, b));
        }
        assert!(app.world().resource::<SharedMaterialCache>().entries.is_empty());
        let a = spawn_shared(&mut app, shimmer_surface());
        let b = spawn_shared(&mut app, shimmer_surface());
        app.update();
        let original = handle(&app, b);
        let before = uniforms(&app, b);
        app.world_mut().get_mut::<Surface>(a).unwrap().noise = Some(
            Noise::grain(24.0, 0.5).with_animated(true).with_speed(1.0));
        tick(&mut app);
        assert_ne!(handle(&app, a), original);
        assert_eq!(uniforms(&app, b), before);
        tick(&mut app);
        assert!(uniforms(&app, a).noise.params2.x > 0.0);
        assert_eq!(uniforms(&app, b), before);
        app.world_mut().get_mut::<Surface>(a).unwrap().noise = None;
        app.update();
        assert_eq!(handle(&app, a), original);
        app.world_mut().get_mut::<Surface>(a).unwrap().backdrop = Some(Backdrop::new().with_blur(8.0));
        app.update();
        assert_ne!(handle(&app, a), original);
        assert_eq!(uniforms(&app, b), before);
    }

    #[test]
    fn shared_missing_assets_and_render_replacement_recover_without_mutation() {
        let mut app = app();
        let a = spawn_shared(&mut app, shimmer_surface());
        let b = spawn_shared(&mut app, shimmer_surface());
        app.update();
        let old = handle(&app, a);
        app.world_mut().entity_mut(a).remove::<MaterialNode<UiShapeMaterial>>();
        app.update();
        assert_eq!(handle(&app, a), old);
        app.world_mut().entity_mut(a).remove::<SurfaceMaterialHandle>();
        app.update();
        assert_eq!(app.world().get::<SurfaceMaterialHandle>(a).unwrap().0, old);
        let mut external = app.world().resource::<Assets<UiShapeMaterial>>().get(&old).unwrap().clone();
        external.uniforms.debug_view = 9.0;
        let adopted = app.world_mut().resource_mut::<Assets<UiShapeMaterial>>().add(external.clone());
        app.world_mut().entity_mut(a).insert(MaterialNode(adopted.clone()));
        app.update();
        assert_eq!(handle(&app, a), old);
        assert_eq!(app.world().resource::<Assets<UiShapeMaterial>>().get(&adopted).unwrap().uniforms, external.uniforms);
        app.world_mut().resource_mut::<Assets<UiShapeMaterial>>().remove(old.id());
        app.update();
        assert_ne!(handle(&app, a), old);
        assert_eq!(handle(&app, a), handle(&app, b));
        assert_eq!(app.world().resource::<SharedMaterialCache>().entries.len(), 1);
    }

    #[test]
    fn shared_cache_releases_last_consumer_but_respects_external_strong_handles() {
        for removal in ["marker", "surface", "despawn"] {
            let mut app = app();
            app.add_plugins((TaskPoolPlugin::default(), AssetPlugin::default()))
                .init_asset::<UiShapeMaterial>();
            let a = spawn_shared(&mut app, shimmer_surface());
            let b = spawn_shared(&mut app, shimmer_surface());
            app.update();
            let external = handle(&app, a);
            let old_id = external.id();
            app.world_mut().despawn(a);
            app.update();
            assert_eq!(app.world().resource::<SharedMaterialCache>().members.len(), 1);
            assert_eq!(handle(&app, b).id(), old_id);
            match removal {
                "marker" => { app.world_mut().entity_mut(b).remove::<SharedSurfaceMaterial>(); }
                "surface" => { app.world_mut().entity_mut(b).remove::<Surface>(); }
                _ => { app.world_mut().despawn(b); }
            }
            app.update();
            let cache = app.world().resource::<SharedMaterialCache>();
            assert!(cache.entries.is_empty(), "{removal}");
            assert!(cache.members.is_empty(), "{removal}");
            assert!(cache.cached_ids.is_empty(), "{removal}");
            app.update();
            assert!(app.world().resource::<Assets<UiShapeMaterial>>().contains(old_id));
            if removal == "marker" {
                assert_ne!(handle(&app, b).id(), old_id);
            } else if removal == "surface" {
                assert!(app.world().get::<MaterialNode<UiShapeMaterial>>(b).is_none());
                assert!(app.world().get::<SurfaceMaterialHandle>(b).is_none());
            }
            drop(external);
            app.update();
            app.update();
            let assets = app.world().resource::<Assets<UiShapeMaterial>>();
            assert!(!assets.contains(old_id), "{removal}");
            assert_eq!(assets.len(), usize::from(removal == "marker"), "{removal}");
        }
    }

    #[test]
    fn shimmer_policy_applies_to_owned_and_shared_fill_and_border_without_authoring_changes() {
        use crate::theme::{AccessibilityVisualPolicy, AccessibilityVisualPolicyResource};
        for shared in [false, true] {
            for reason in ["motion", "effects", "runtime"] {
                let mut app = app();
                let entity = if shared { spawn_shared(&mut app, shimmer_surface()) }
                    else { spawn(&mut app, shimmer_surface()) };
                app.update();
                app.update();
                let active = uniforms(&app, entity);
                assert_eq!(active.fill_paint.kind_and_flags.w, 1.0);
                assert_eq!(active.border_paint.kind_and_flags.w, 1.0);
                if reason == "runtime" {
                    app.world_mut().resource_mut::<BackdropRuntimeSettings>().reduced_effects = true;
                } else {
                    app.world_mut().insert_resource(AccessibilityVisualPolicyResource {
                        current: AccessibilityVisualPolicy {
                            reduced_motion: reason == "motion", reduced_effects: reason == "effects", ..default()
                        },
                    });
                }
                app.update();
                let mut expected = active;
                expected.fill_paint.kind_and_flags.w = 0.0;
                expected.border_paint.kind_and_flags.w = 0.0;
                assert_eq!(uniforms(&app, entity), expected, "shared={shared}, {reason}");
                assert_eq!(app.world().get::<Surface>(entity).unwrap(), &shimmer_surface());
                app.update();
                reset_stats(&mut app);
                tick(&mut app);
                let stats = app.world().resource::<MaterialSyncStats>();
                assert_eq!((stats.rebuilds, stats.writes, stats.shared_keys), (0, 0, 0));
                app.world_mut().remove_resource::<AccessibilityVisualPolicyResource>();
                app.world_mut().resource_mut::<BackdropRuntimeSettings>().reduced_effects = false;
                app.update();
                assert_eq!(uniforms(&app, entity), active);
            }
        }
    }

    fn glass_surface() -> Surface {
        Surface::rounded_rect_fill(8.0, Color::NONE).with_backdrop(Backdrop {
            tint: Color::srgba(0.2, 0.4, 0.6, 0.25),
            liquid_glass: Some(super::super::LiquidGlass { press_amount: 0.75, ..default() }),
            ..default()
        })
    }

    #[test]
    fn liquid_glass_fallback_for_disabled_reduced_and_ineligible_capture() {
        let mut app = app();
        let surface = glass_surface();
        let tint = surface.backdrop.unwrap().tint.to_linear().to_vec4();
        let entity = spawn(&mut app, surface.clone());
        app.update();
        app.update(); // settle deferred material attachment before resource-only changes
        let active = uniforms(&app, entity);
        assert_eq!(active.glass_optics.x, 1.0);
        assert_eq!(active.glass_optics.y, 6.0); // fixture is 2x DPI
        assert_eq!(active.glass_optics.z, 9.0);
        assert_eq!(active.glass_light.y, 1.6);
        for reason in ["disabled", "reduced", "no eligible capture"] {
            match reason {
                "disabled" => app.world_mut().resource_mut::<BackdropRuntimeSettings>().enabled = false,
                "reduced" => app.world_mut().resource_mut::<BackdropRuntimeSettings>().reduced_effects = true,
                _ => { app.world_mut().insert_resource(capture::BackdropCaptureState::default()); }
            }
            app.update();
            let fallback = uniforms(&app, entity);
            assert_eq!(fallback.glass_optics, Vec4::ZERO, "{reason}");
            assert_eq!(fallback.backdrop_params0.x, 0.0, "{reason}");
            assert_eq!(fallback.fill_paint.kind_and_flags.x, 0.0, "{reason}");
            assert_eq!(fallback.fill_paint.solid_color, tint.truncate().extend(1.0), "{reason}");
            assert_eq!(app.world().get::<Surface>(entity).unwrap(), &surface);
            // Restoring settings alone must recover authored transparent paint.
            // Missing capture state is the existing headless fixture's opt-in.
            app.world_mut().remove_resource::<capture::BackdropCaptureState>();
            let mut settings = app.world_mut().resource_mut::<BackdropRuntimeSettings>();
            settings.enabled = true;
            settings.reduced_effects = false;
            app.update();
            assert_eq!(uniforms(&app, entity), active);
        }
    }

    #[test]
    fn liquid_glass_reduced_motion_policy_zeroes_press_without_changing_geometry() {
        use crate::theme::AccessibilityVisualPolicyResource;
        let mut app = app();
        let entity = spawn(&mut app, glass_surface());
        app.update();
        app.update();
        let active = uniforms(&app, entity);
        assert_eq!(active.glass_state.y, 0.75); // optional policy absent
        app.world_mut().insert_resource(AccessibilityVisualPolicyResource {
            current: crate::theme::AccessibilityVisualPolicy { reduced_motion: true, ..default() },
        });
        app.update();
        let mut expected = active;
        expected.glass_state.y = 0.0;
        assert_eq!(uniforms(&app, entity), expected);
        assert_eq!(app.world().get::<Surface>(entity).unwrap().backdrop.unwrap()
            .liquid_glass.unwrap().press_amount, 0.75);
        app.world_mut().resource_mut::<AccessibilityVisualPolicyResource>().current.reduced_motion = false;
        app.update();
        assert_eq!(uniforms(&app, entity), active);
    }

    #[test]
    fn liquid_glass_surface_axes_preserve_exact_linear_columns_not_translation_or_dpi() {
        let mut app = app();
        let entity = spawn(&mut app, glass_surface());
        // Exact quarter-turn columns avoid trig roundoff and expose transpose,
        // absolute-value, AABB, and extra-DPI conversion mistakes independently.
        for axes in [Vec4::new(1.0, 0.0, 0.0, 1.0), Vec4::new(2.0, 0.0, 0.0, 0.5),
            Vec4::new(0.0, 1.0, -1.0, 0.0), Vec4::new(-2.0, 0.0, 0.0, 0.5)]
        {
            for translation in [Vec2::ZERO, Vec2::new(300.0, 200.0)] {
                let affine = Affine2::from_cols(axes.xy(), axes.zw(), translation);
                app.world_mut().entity_mut(entity).insert(UiGlobalTransform::from(affine));
                app.update();
                let encoded = uniforms(&app, entity);
                assert_eq!(encoded.surface_axes, axes, "translation={translation:?}");
                assert_eq!(encoded.size_and_kind.xy(), Vec2::new(200.0, 100.0));
                assert_eq!(encoded.glass_optics.y, 6.0);
            }
        }
        app.world_mut().entity_mut(entity).remove::<UiGlobalTransform>();
        app.update();
        assert_eq!(uniforms(&app, entity).surface_axes, Vec4::new(1.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn preserves_native_borders_and_explicit_surface() {
        let mut app = app();
        let node = Node { border: UiRect::all(px(3.0)), ..default() };
        let border = BorderColor {
            top: Color::WHITE, right: Color::BLACK,
            bottom: Color::srgb(1.0, 0.0, 0.0), left: Color::srgb(0.0, 1.0, 0.0),
        };
        let native = app.world_mut().spawn((node.clone(), border, BackgroundColor(Color::WHITE))).id();
        let explicit = app.world_mut().spawn((node.clone(), border, BackgroundColor(Color::WHITE), surface())).id();
        app.update();
        app.world_mut().get_mut::<BorderColor>(explicit).unwrap().top = Color::BLACK;
        app.update();
        for entity in [native, explicit] {
            assert_eq!(app.world().get::<Node>(entity).unwrap().border, node.border);
            assert_eq!(app.world().get::<BackgroundColor>(entity).unwrap().0, Color::WHITE);
        }
        assert_eq!(app.world().get::<BorderColor>(native).unwrap(), &border);
        assert!(app.world().get::<Surface>(native).is_none());
        assert!(app.world().get::<MaterialNode<UiShapeMaterial>>(native).is_none());
        assert_eq!(app.world().get::<Surface>(explicit).unwrap(), &surface());
    }

    #[test]
    fn lifecycle_removal_reinsertion_and_unmanaged_materials() {
        let mut app = app();
        let entity = spawn(&mut app, surface());
        let unmanaged = app.world_mut().spawn(MaterialNode::<UiShapeMaterial>(Handle::default())).id();
        app.update();
        let original = handle(&app, entity);
        app.world_mut().entity_mut(entity).remove::<Surface>();
        app.update();
        assert!(app.world().get::<MaterialNode<UiShapeMaterial>>(entity).is_none());
        assert!(app.world().get::<SurfaceMaterialHandle>(entity).is_none());
        assert!(app.world().get::<SurfaceMaterialManaged>(entity).is_none());
        assert!(app.world().get::<MaterialNode<UiShapeMaterial>>(unmanaged).is_some());
        // Cleanup drops component references, not assets still referenced elsewhere.
        assert!(app.world().resource::<Assets<UiShapeMaterial>>().contains(original.id()));
        app.world_mut().entity_mut(entity).insert(surface());
        app.update();
        assert_ne!(handle(&app, entity), original);
        app.world_mut().entity_mut(entity).remove::<Surface>().insert(surface());
        app.update();
        assert!(app.world().get::<SurfaceMaterialManaged>(entity).is_some());
        app.world_mut().despawn(entity);
        app.update();
    }

    #[test]
    fn recovers_components_and_missing_asset_with_render_handle_authority() {
        let mut app = app();
        let entity = spawn(&mut app, surface());
        app.update();
        let original = handle(&app, entity);
        app.world_mut().entity_mut(entity).remove::<MaterialNode<UiShapeMaterial>>();
        app.update();
        assert_eq!(handle(&app, entity), original);
        app.world_mut().entity_mut(entity).remove::<SurfaceMaterialHandle>();
        app.update();
        assert_eq!(app.world().get::<SurfaceMaterialHandle>(entity).unwrap().0, original);

        let mut other = app.world().resource::<Assets<UiShapeMaterial>>().get(&original).unwrap().clone();
        other.uniforms.size_and_kind = Vec4::ZERO;
        let rendered = app.world_mut().resource_mut::<Assets<UiShapeMaterial>>().add(other);
        app.world_mut().entity_mut(entity).insert(MaterialNode(rendered.clone()));
        app.update();
        assert_eq!(handle(&app, entity), rendered);
        assert_eq!(app.world().get::<SurfaceMaterialHandle>(entity).unwrap().0, rendered);
        assert_eq!(uniforms(&app, entity).size_and_kind.x, 200.0);
        app.world_mut().entity_mut(entity).insert(SurfaceMaterialHandle(original.clone()));
        app.update();
        assert_eq!(app.world().get::<SurfaceMaterialHandle>(entity).unwrap().0, rendered);

        app.world_mut().resource_mut::<Assets<UiShapeMaterial>>().remove(rendered.id());
        app.update();
        let recovered = handle(&app, entity);
        assert_ne!(recovered, rendered);
        assert_eq!(app.world().get::<SurfaceMaterialHandle>(entity).unwrap().0, recovered);
        assert_eq!(uniforms(&app, entity).size_and_kind.x, 200.0);
        app.world_mut().entity_mut(entity).remove::<(MaterialNode<UiShapeMaterial>, SurfaceMaterialHandle)>();
        app.update();
        assert_ne!(handle(&app, entity), recovered);
    }

    #[test]
    fn adopts_preexisting_render_or_public_handle_without_allocating_duplicates() {
        let mut app = app();
        let seed = spawn(&mut app, surface());
        app.update();
        let material = handle(&app, seed);
        app.world_mut().despawn(seed);
        let entity = spawn(&mut app, surface());
        app.world_mut().entity_mut(entity).insert(SurfaceMaterialHandle(material.clone()));
        app.update();
        assert_eq!(handle(&app, entity), material);
        app.world_mut().despawn(entity);
        let entity = spawn(&mut app, surface());
        app.world_mut().entity_mut(entity).insert(MaterialNode(material.clone()));
        app.update();
        assert_eq!(handle(&app, entity), material);
        assert_eq!(app.world().resource::<Assets<UiShapeMaterial>>().len(), 1);
    }

    #[test]
    fn sync_runs_after_post_layout_and_flushes_before_last() {
        let mut app = app();
        let entity = spawn(&mut app, surface());
        app.add_systems(PostUpdate, (|mut nodes: Query<&mut ComputedNode>| {
            for mut node in &mut nodes { node.size = Vec2::new(444.0, 222.0); }
        }).in_set(UiSystems::PostLayout));
        app.add_systems(Last, |query: Query<(&SurfaceMaterialHandle, &MaterialNode<UiShapeMaterial>)>| {
            let (mirror, render) = query.single().unwrap();
            assert_eq!(mirror.0, render.0);
        });
        app.update();
        assert_eq!(uniforms(&app, entity).size_and_kind.x, 444.0);
        assert_eq!(app.world().resource::<MaterialSyncStats>().rebuilds, 1);
    }

    #[test]
    fn material_asset_events_are_published_before_same_frame_extraction() {
        let mut app = app();
        app.add_plugins((TaskPoolPlugin::default(), AssetPlugin::default()))
            .init_asset::<UiShapeMaterial>();
        let entity = spawn(&mut app, surface());
        app.add_systems(Last, |mut events: MessageReader<AssetEvent<UiShapeMaterial>>,
            render: Query<&MaterialNode<UiShapeMaterial>>| {
            let id = render.single().unwrap().0.id();
            assert!(events.read().any(|event| matches!(event,
                AssetEvent::Added { id: changed } | AssetEvent::Modified { id: changed }
                    if *changed == id)));
        });
        app.update();
        app.world_mut().get_mut::<Surface>(entity).unwrap().fill = Color::BLACK.into();
        app.update();
    }

    #[test]
    fn invalidates_all_layout_inputs_and_resource_only_changes() {
        let mut app = app();
        let entity = spawn(&mut app, surface());
        app.update();
        app.update(); // settle the newly inserted render component
        reset_stats(&mut app);
        tick(&mut app);
        assert_eq!(app.world().resource::<MaterialSyncStats>().rebuilds, 0);
        assert_eq!(app.world().resource::<MaterialSyncStats>().writes, 0);
        app.world_mut().get_mut::<UiGlobalTransform>(entity).unwrap().clone_from(&UiGlobalTransform::from_xy(400.0, 300.0));
        app.update();
        assert_eq!(uniforms(&app, entity).backdrop_uv_rect, Vec4::new(0.3, 0.3125, 0.5, 0.4375));
        app.world_mut().entity_mut(entity).insert(target(UVec2::new(2000, 1600)));
        app.update();
        assert_eq!(uniforms(&app, entity).backdrop_uv_rect, Vec4::new(0.15, 0.15625, 0.25, 0.21875));
        app.world_mut().get_mut::<ComputedNode>(entity).unwrap().size.x = 300.0;
        app.update();
        assert_eq!(uniforms(&app, entity).size_and_kind.x, 300.0);
        app.world_mut().get_mut::<Surface>(entity).unwrap().fill = Color::BLACK.into();
        app.update();
        assert_eq!(uniforms(&app, entity).fill_paint.solid_color, Vec4::new(0.0, 0.0, 0.0, 1.0));
        app.world_mut().get_mut::<Node>(entity).unwrap().width = px(250.0);
        app.update();
        assert_eq!(app.world().resource::<MaterialSyncStats>().rebuilds, 5);
        app.world_mut().entity_mut(entity).remove::<ComputedNode>();
        app.update();
        assert_eq!(uniforms(&app, entity).size_and_kind.x, 250.0);
        app.world_mut().resource_mut::<BackdropRuntimeSettings>().enabled = false;
        app.update();
        assert_eq!(uniforms(&app, entity).backdrop_params0.x, 0.0);
        app.world_mut().resource_mut::<BackdropRuntimeSettings>().render_debug_view = UiRenderDebugView::Sdf;
        app.update();
        assert_eq!(uniforms(&app, entity).debug_view, 1.0);
        app.world_mut().resource_mut::<BackdropRuntimeSettings>().backdrop_debug_view = BackdropDebugView::Source;
        app.update();
        assert_eq!(uniforms(&app, entity).backdrop_params1.z, 1.0);
        let image = app.world_mut().resource_mut::<Assets<Image>>().add(Image::default());
        app.world_mut().resource_mut::<BackdropSourceTexture>().image = image.clone();
        app.update();
        assert_eq!(app.world().resource::<Assets<UiShapeMaterial>>().get(&handle(&app, entity)).unwrap().backdrop_texture, image);
    }

    #[test]
    fn target_transform_removal_and_window_fallback_resize_invalidate() {
        let mut app = app();
        let entity = spawn(&mut app, surface());
        let window = app.world_mut().spawn((PrimaryWindow, Window {
            resolution: (1000, 800).into(), ..default()
        })).id();
        app.update();
        app.world_mut().entity_mut(entity).remove::<ComputedUiRenderTargetInfo>();
        app.update();
        assert_eq!(uniforms(&app, entity).backdrop_uv_rect, Vec4::new(0.2, 0.1875, 0.4, 0.3125));
        app.world_mut().get_mut::<Window>(window).unwrap().resolution.set_physical_resolution(2000, 1600);
        app.update();
        assert_eq!(uniforms(&app, entity).backdrop_uv_rect, Vec4::new(0.1, 0.09375, 0.2, 0.15625));
        app.world_mut().entity_mut(entity).remove::<UiGlobalTransform>();
        app.update();
        assert_eq!(uniforms(&app, entity).backdrop_uv_rect, Vec4::new(0.0, 0.0, 1.0, 1.0));
    }

    #[test]
    fn animated_clock_updates_without_rebuilds_and_reduced_effects_skip_work() {
        let mut app = app();
        let animated = spawn(&mut app, surface().with_noise(Noise::grain(24.0, 0.5).with_animated(true).with_speed(1.0)));
        let static_entity = spawn(&mut app, surface().with_noise(Noise::grain(24.0, 0.5)));
        let zero_speed = spawn(&mut app, surface().with_noise(Noise::grain(24.0, 0.5).with_animated(true)));
        app.update();
        app.update();
        let static_uniforms = uniforms(&app, static_entity);
        reset_stats(&mut app);
        tick(&mut app);
        assert_eq!(app.world().resource::<MaterialSyncStats>().rebuilds, 0);
        assert_eq!(app.world().resource::<MaterialSyncStats>().writes, 1);
        assert!(uniforms(&app, animated).noise.params2.x > 0.0);
        assert_eq!(uniforms(&app, static_entity), static_uniforms);
        assert_eq!(uniforms(&app, zero_speed).noise.params2.x, 0.0);
        app.world_mut().resource_mut::<BackdropRuntimeSettings>().reduced_effects = true;
        tick(&mut app);
        assert_eq!(uniforms(&app, animated).noise.params0.x, 0.0);
        reset_stats(&mut app);
        tick(&mut app);
        assert_eq!(app.world().resource::<MaterialSyncStats>().rebuilds, 0);
        assert_eq!(app.world().resource::<MaterialSyncStats>().writes, 0);
        app.world_mut().resource_mut::<BackdropRuntimeSettings>().reduced_effects = false;
        tick(&mut app);
        assert_eq!(app.world().resource::<MaterialSyncStats>().rebuilds, 3);
        assert_eq!(uniforms(&app, animated).noise.params0.x, 1.0);
    }

    #[test]
    fn backdrop_physical_coordinates_no_y_flip_dpi_or_padding_stretch() {
        let node = ComputedNode { size: Vec2::new(200.0, 100.0), inverse_scale_factor: 0.5, ..default() };
        let transform = UiGlobalTransform::from_xy(300.0, 200.0);
        let region = compute_backdrop_sample_region(Some(&node), Some(&target(UVec2::new(1000, 800))), Some(&transform), None);
        assert_eq!(region.min_uv, Vec2::new(0.2, 0.1875));
        assert_eq!(region.max_uv, Vec2::new(0.4, 0.3125));
        let scaled = UiGlobalTransform::from(Affine2::from_scale_angle_translation(Vec2::new(2.0, 0.5), 0.0, Vec2::new(300.0, 200.0)));
        let region = compute_backdrop_sample_region(Some(&node), None, Some(&scaled), Some(Vec2::new(1000.0, 800.0)));
        assert_eq!(region.min_uv, Vec2::new(0.1, 0.21875));
        assert_eq!(region.max_uv, Vec2::new(0.5, 0.28125));
        let mut app = app();
        let entity = spawn(&mut app, surface());
        app.update();
        let before = uniforms(&app, entity).backdrop_uv_rect;
        app.world_mut().get_mut::<Surface>(entity).unwrap().backdrop.as_mut().unwrap().blur = 100.0;
        app.update();
        assert_eq!(uniforms(&app, entity).backdrop_uv_rect, before);
    }

    #[test]
    fn backdrop_rotation_aabb_and_unclamped_offscreen_coordinates_are_explicit() {
        let node = ComputedNode { size: Vec2::splat(100.0), ..default() };
        let rotation = UiGlobalTransform::from(Affine2::from_angle(std::f32::consts::FRAC_PI_4));
        let region = compute_backdrop_sample_region(Some(&node), None, Some(&rotation), Some(Vec2::splat(1000.0)));
        let extent = 50.0 * 2.0_f32.sqrt() / 1000.0;
        assert!((region.min_uv + Vec2::splat(extent)).length() < 1e-6);
        assert!((region.max_uv - Vec2::splat(extent)).length() < 1e-6);
        assert_eq!(compute_backdrop_sample_region(None, None, None, None), BackdropSampleRegion::default());
        assert_eq!(compute_backdrop_sample_region(Some(&node), None, Some(&rotation), Some(Vec2::ZERO)), BackdropSampleRegion::default());
    }

    #[test]
    fn startup_preserves_explicit_source_texture() {
        let mut app = app();
        let image = app.world_mut().resource_mut::<Assets<Image>>().add(Image::default());
        app.world_mut().resource_mut::<BackdropSourceTexture>().image = image.clone();
        app.add_systems(Startup, initialize_backdrop_source_texture);
        app.update();
        assert_eq!(app.world().resource::<BackdropSourceTexture>().image, image);
        assert_eq!(app.world().resource::<Assets<Image>>().len(), 1);
    }

    /// CPU-only, headless schedule + asset synchronization; no layout/GPU/capture.
    /// Run explicitly with `rendering::plugin::tests::surface_cpu_stress --ignored --nocapture`.
    /// Apple M4 arm64, Cargo test profile (opt-level=1), 100 measured passes:
    /// surfaces | attach ms | static us/pass | animated us/pass | reduced us/pass
    ///      100 |     0.644 |         20.743 |           17.189 |          17.771
    ///     1000 |     1.190 |         31.876 |           44.410 |          31.009
    ///    10000 |    21.138 |        225.182 |          371.548 |         220.254
    /// Every steady-state phase rebuilt zero uniforms; only animated wrote assets
    /// (one clock write/surface/pass). Single-run CPU timings, not GPU guarantees.
    #[test]
    #[ignore = "manual 100/1000/10000-surface CPU timing"]
    fn surface_cpu_stress() {
        for count in [100, 1000, 10_000] {
            let mut app = app();
            for _ in 0..count { spawn(&mut app, surface()); }
            let start = Instant::now();
            app.update();
            let attach = start.elapsed();
            app.update();
            for phase in ["static", "animated", "reduced"] {
                if phase == "animated" {
                    let world = app.world_mut();
                    for mut surface in world.query::<&mut Surface>().iter_mut(world) {
                        surface.noise = Some(Noise::grain(24.0, 0.5).with_animated(true).with_speed(1.0));
                    }
                }
                if phase == "reduced" {
                    app.world_mut().resource_mut::<BackdropRuntimeSettings>().reduced_effects = true;
                }
                tick(&mut app);
                reset_stats(&mut app);
                let start = Instant::now();
                for _ in 0..100 { tick(&mut app); }
                let elapsed = start.elapsed();
                let stats = app.world().resource::<MaterialSyncStats>();
                assert_eq!(stats.rebuilds, 0);
                assert_eq!(stats.writes, if phase == "animated" { count * 100 } else { 0 });
                eprintln!("surfaces={count} phase={phase} attach_ms={:.3} mean_us={:.3} rebuilds={} writes={}",
                    attach.as_secs_f64() * 1000.0, elapsed.as_secs_f64() * 10_000.0, stats.rebuilds, stats.writes);
            }
        }
    }
}

