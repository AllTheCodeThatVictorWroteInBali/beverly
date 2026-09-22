use bevy::ecs::query::QueryData;
use bevy::prelude::*;
use bevy::ui::{CalculatedClip, ComputedStackIndex};

use super::hit_shape::{HitShape, HitSlop};

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitBehavior {
    Auto,
    None,
    ChildrenOnly,
    SelfOnly,
    SelfAndChildren,
}

impl Default for HitBehavior {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct UiHitNode;

#[derive(Clone, Copy, Debug)]
pub struct HitTarget {
    pub entity: Entity,
    /// Logical coordinates relative to the node's content-box origin.
    pub local_position: Vec2,
    /// Legacy diagnostic only; picking uses the resolved `ComputedStackIndex`.
    pub z_hint: i32,
}

#[derive(Resource, Default, Clone, Debug)]
pub struct UiHitTargetCache {
    pub last_primary_target: Option<Entity>,
}

#[derive(Resource, Default, Clone, Debug)]
pub struct UiHitTestDebugFrame {
    pub pointer: Vec2,
    pub broad_phase_candidates: usize,
    pub narrow_phase_candidates: usize,
    pub final_target: Option<Entity>,
    pub capture_target: Option<Entity>,
    pub hover_target: Option<Entity>,
}

/// Includes ordinary UI nodes: an unmarked overlay can still block a custom target.
#[derive(QueryData)]
pub struct HitTestNode {
    entity: Entity,
    computed: &'static ComputedNode,
    transform: &'static UiGlobalTransform,
    stack: &'static ComputedStackIndex,
    interactive: Has<UiHitNode>,
    shape: Option<&'static HitShape>,
    slop: Option<&'static HitSlop>,
    behavior: Option<&'static HitBehavior>,
    z: Option<&'static ZIndex>,
    visibility: Option<&'static InheritedVisibility>,
    clip: Option<&'static CalculatedClip>,
    pickable: Option<&'static Pickable>,
}

struct Candidate {
    entity: Entity,
    stack: u32,
    interactive: bool,
    target: Option<HitTarget>,
    blocks: bool,
}

/// Picks a logical primary-window cursor against physical UI geometry.
///
/// Like Bevy's UI picking backend, the cursor must be scaled by the render
/// target's DPI, NOT by the node's layout scale (which also includes `UiScale`).
/// This entry point assumes a full-window viewport. Camera viewport offsets,
/// other render targets and cross-camera ordering are not resolved here.
/// `CalculatedClip` is Bevy's exposed, inherited physical scissor rectangle;
/// it is authoritative here, not a reconstruction of transformed ancestor clips.
pub fn hit_test_primary(
    pointer_window: Vec2,
    window_scale_factor: f32,
    query: &Query<HitTestNode>,
    parents: &Query<&ChildOf>,
) -> Option<HitTarget> {
    if !window_scale_factor.is_finite() || window_scale_factor <= 0.0 {
        return None;
    }
    let pointer_physical = pointer_window * window_scale_factor;
    if !pointer_physical.is_finite() {
        return None;
    }
    let mut candidates = Vec::new();
    for node in query.iter() {
        if matches!(node.behavior, Some(HitBehavior::None | HitBehavior::ChildrenOnly))
            || node.visibility.is_none_or(|visibility| !visibility.get())
            || node.computed.is_empty()
            || node.clip.is_some_and(|clip| {
                clip.clip.is_empty() || !clip.clip.contains(pointer_physical)
            })
        {
            continue;
        }

        let hoverable = node.pickable.is_none_or(|pickable| pickable.is_hoverable);
        let blocks = node.pickable.is_none_or(|pickable| pickable.should_block_lower);
        if !blocks && (!node.interactive || !hoverable) {
            continue;
        }
        let slop = if node.interactive {
            node.slop.copied().unwrap_or_default()
        } else {
            HitSlop::default()
        };
        let Some(centered_local) = hit_local(
            pointer_physical,
            node.computed,
            node.transform,
            slop,
        ) else {
            continue;
        };
        let inverse_scale = node.computed.inverse_scale_factor();
        // Shapes span the border box, not the inset content box. Both shape
        // parameters and slop are expressed in logical node-local pixels.
        let shape_local = (centered_local + node.computed.size() * 0.5) * inverse_scale;
        let contains = if node.interactive {
            node.shape.cloned().unwrap_or(HitShape::LayoutRect).contains_local(
                shape_local,
                node.computed.size() * inverse_scale,
                slop,
            )
        } else {
            // Ordinary nodes use Bevy's resolved border radius as well.
            node.computed.contains_point(*node.transform, pointer_physical)
        };
        if !contains {
            continue;
        }
        candidates.push(Candidate {
            entity: node.entity,
            stack: node.stack.0,
            interactive: node.interactive,
            target: (node.interactive && hoverable).then_some(HitTarget {
                entity: node.entity,
                local_position: (centered_local - node.computed.content_box().min) * inverse_scale,
                z_hint: node.z.map_or(0, |z| z.0),
            }),
            blocks,
        });
    }

    candidates.sort_unstable_by(|a, b| {
        b.stack.cmp(&a.stack).then_with(|| b.entity.cmp(&a.entity))
    });
    for candidate in &candidates {
        if candidate.target.is_some() {
            return candidate.target;
        }
        if candidate.blocks {
            if candidate.interactive {
                return None;
            }
            // A button's text/image child is an ordinary blocking UI node.
            // Resolve to its nearest eligible ancestor, but never tunnel through
            // an unrelated overlay or a blocking non-hoverable interactive node.
            let mut entity = candidate.entity;
            while let Ok(parent) = parents.get(entity) {
                entity = parent.parent();
                if let Some(ancestor) = candidates.iter().find(|hit| hit.entity == entity && hit.interactive) {
                    return ancestor.target;
                }
            }
            return None;
        }
    }
    None
}

/// Conservative physical AABB of the slop-expanded, transformed border box,
/// followed by an inverse transform. Non-invertible/non-finite nodes cannot hit.
fn hit_local(
    pointer_physical: Vec2,
    computed: &ComputedNode,
    transform: &UiGlobalTransform,
    slop: HitSlop,
) -> Option<Vec2> {
    let inverse_scale = computed.inverse_scale_factor();
    if !inverse_scale.is_finite() || inverse_scale <= 0.0 || !computed.size().is_finite() {
        return None;
    }
    let min = -computed.size() * 0.5 - Vec2::new(slop.left, slop.top) / inverse_scale;
    let max = computed.size() * 0.5 + Vec2::new(slop.right, slop.bottom) / inverse_scale;
    let mut bounds = Rect {
        min: Vec2::splat(f32::INFINITY),
        max: Vec2::splat(f32::NEG_INFINITY),
    };
    for corner in [min, Vec2::new(max.x, min.y), max, Vec2::new(min.x, max.y)] {
        let corner = transform.transform_point2(corner);
        if !corner.is_finite() {
            return None;
        }
        bounds.min = bounds.min.min(corner);
        bounds.max = bounds.max.max(corner);
    }
    if !bounds.contains(pointer_physical) {
        return None;
    }
    let local = transform.try_inverse()?.transform_point2(pointer_physical);
    local.is_finite().then_some(local)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::{RunSystemOnce, SystemState};
    use bevy::math::Affine2;

    fn spawn_node(world: &mut World, size: Vec2, transform: UiGlobalTransform, stack: u32) -> Entity {
        world.spawn((
            ComputedNode { size, ..default() },
            transform,
            ComputedStackIndex(stack),
            InheritedVisibility::VISIBLE,
        )).id()
    }

    fn target(world: &mut World, size: Vec2, transform: UiGlobalTransform, stack: u32) -> Entity {
        let entity = spawn_node(world, size, transform, stack);
        world.entity_mut(entity).insert(UiHitNode);
        entity
    }

    fn pick(world: &mut World, point: Vec2, dpi: f32) -> Option<HitTarget> {
        let mut state = SystemState::<(Query<HitTestNode>, Query<&ChildOf>)>::new(world);
        let (nodes, parents) = state.get(world).unwrap();
        hit_test_primary(point, dpi, &nodes, &parents)
    }

    fn assert_position(actual: Vec2, expected: Vec2) {
        assert!((actual - expected).length() < 0.001, "{actual:?} != {expected:?}");
    }

    #[test]
    fn dpi_two_converts_cursor_and_local_position_without_scaling_translation_twice() {
        let mut world = World::new();
        let entity = target(&mut world, Vec2::new(200.0, 80.0), UiGlobalTransform::from_xy(300.0, 160.0), 1);
        world.get_mut::<ComputedNode>(entity).unwrap().inverse_scale_factor = 0.5;
        let hit = pick(&mut world, Vec2::new(125.0, 70.0), 2.0).unwrap();
        assert_eq!(hit.entity, entity);
        assert_position(hit.local_position, Vec2::new(25.0, 10.0));
        assert!(pick(&mut world, Vec2::new(205.0, 80.0), 2.0).is_none());
    }

    #[test]
    fn dpi_and_ui_scale_are_distinct() {
        let mut world = World::new();
        let entity = target(&mut world, Vec2::new(400.0, 160.0), UiGlobalTransform::from_xy(400.0, 200.0), 1);
        // DPI 2 * UiScale 2: only DPI scales the window cursor.
        world.get_mut::<ComputedNode>(entity).unwrap().inverse_scale_factor = 0.25;
        let hit = pick(&mut world, Vec2::new(200.0, 100.0), 2.0).unwrap();
        assert_position(hit.local_position, Vec2::new(50.0, 20.0));
    }

    #[test]
    fn transformed_bounds_and_inverse_narrow_phase_handle_rotation_scale_and_reflection() {
        for scale in [Vec2::new(2.0, 0.75), Vec2::new(-2.0, 0.75)] {
            let mut world = World::new();
            let transform = UiGlobalTransform::from(Affine2::from_scale_angle_translation(
                scale, std::f32::consts::FRAC_PI_4, Vec2::splat(200.0),
            ));
            let entity = target(&mut world, Vec2::new(100.0, 20.0), transform, 1);
            let point = transform.transform_point2(Vec2::new(40.0, 0.0));
            let hit = pick(&mut world, point, 1.0).unwrap();
            assert_eq!(hit.entity, entity);
            assert_position(hit.local_position, Vec2::new(90.0, 10.0));
            // Inside the transformed AABB, outside the rotated rectangle.
            assert!(pick(&mut world, Vec2::new(200.0, 250.0), 1.0).is_none());
        }
    }

    #[test]
    fn shear_uses_all_four_transformed_corners() {
        let mut world = World::new();
        let transform = UiGlobalTransform::from(Affine2::from_cols(
            Vec2::X, Vec2::new(2.0, 1.0), Vec2::splat(100.0),
        ));
        target(&mut world, Vec2::splat(40.0), transform, 1);
        let hit = pick(&mut world, transform.transform_point2(Vec2::splat(15.0)), 1.0).unwrap();
        assert_position(hit.local_position, Vec2::splat(35.0));
    }

    #[test]
    fn asymmetric_slop_survives_broad_phase_at_dpi_two_and_under_transform() {
        let mut world = World::new();
        let transform = UiGlobalTransform::from(Affine2::from_scale_angle_translation(
            Vec2::new(1.5, 0.8), 0.6, Vec2::splat(200.0),
        ));
        let entity = target(&mut world, Vec2::splat(80.0), transform, 1);
        world.get_mut::<ComputedNode>(entity).unwrap().inverse_scale_factor = 0.5;
        world.entity_mut(entity).insert(HitSlop { left: 10.0, right: 2.0, top: 4.0, bottom: 6.0 });
        for local in [Vec2::new(-56.0, 0.0), Vec2::new(42.0, 0.0), Vec2::new(0.0, -46.0), Vec2::new(0.0, 50.0)] {
            assert!(pick(&mut world, transform.transform_point2(local) * 0.5, 2.0).is_some());
        }
        for local in [Vec2::new(-62.0, 0.0), Vec2::new(46.0, 0.0), Vec2::new(0.0, -50.0), Vec2::new(0.0, 54.0)] {
            assert!(pick(&mut world, transform.transform_point2(local) * 0.5, 2.0).is_none());
        }
    }

    #[test]
    fn shape_units_are_logical_and_narrow_phase_rejects_aabb_corners() {
        let mut world = World::new();
        let entity = target(&mut world, Vec2::splat(80.0), UiGlobalTransform::from_xy(100.0, 100.0), 1);
        world.get_mut::<ComputedNode>(entity).unwrap().inverse_scale_factor = 0.5;
        world.entity_mut(entity).insert(HitShape::Circle);
        assert!(pick(&mut world, Vec2::splat(50.0), 2.0).is_some());
        assert!(pick(&mut world, Vec2::splat(31.0), 2.0).is_none());
        world.entity_mut(entity).insert(HitShape::CustomRect(Rect::from_corners(Vec2::splat(5.0), Vec2::splat(15.0))));
        assert!(pick(&mut world, Vec2::splat(40.0), 2.0).is_some());
        assert!(pick(&mut world, Vec2::splat(50.0), 2.0).is_none());
    }

    #[test]
    fn padding_does_not_shift_shape_but_local_position_remains_content_relative() {
        let mut world = World::new();
        let entity = target(&mut world, Vec2::splat(80.0), UiGlobalTransform::from_xy(100.0, 100.0), 1);
        {
            let mut computed = world.get_mut::<ComputedNode>(entity).unwrap();
            computed.inverse_scale_factor = 0.5;
            computed.padding.min_inset = Vec2::splat(12.0);
            computed.border.min_inset = Vec2::splat(4.0);
        }
        let hit = pick(&mut world, Vec2::splat(31.0), 2.0).unwrap();
        assert_position(hit.local_position, Vec2::splat(-7.0));
    }

    #[test]
    fn inherited_physical_clip_limits_shape_and_slop() {
        let mut world = World::new();
        let entity = target(&mut world, Vec2::splat(80.0), UiGlobalTransform::from_xy(100.0, 100.0), 1);
        world.get_mut::<ComputedNode>(entity).unwrap().inverse_scale_factor = 0.5;
        world.entity_mut(entity).insert((
            HitSlop::all(20.0),
            CalculatedClip { clip: Rect::from_corners(Vec2::splat(80.0), Vec2::splat(120.0)) },
        ));
        assert!(pick(&mut world, Vec2::splat(50.0), 2.0).is_some());
        assert!(pick(&mut world, Vec2::splat(35.0), 2.0).is_none());
        assert!(pick(&mut world, Vec2::splat(25.0), 2.0).is_none());
        world.entity_mut(entity).insert(CalculatedClip { clip: Rect::default() });
        assert!(pick(&mut world, Vec2::ZERO, 2.0).is_none());
    }

    #[test]
    fn bevy_clip_propagation_crosses_visible_wrappers_and_honors_override_clip() {
        let mut world = World::new();
        let transform = UiGlobalTransform::from_xy(100.0, 100.0);
        let root = spawn_node(&mut world, Vec2::splat(40.0), transform, 0);
        let wrapper = spawn_node(&mut world, Vec2::splat(100.0), transform, 1);
        let button = target(&mut world, Vec2::splat(100.0), transform, 2);
        world.entity_mut(root).insert((Node { overflow: Overflow::hidden(), ..default() }, Pickable::IGNORE));
        world.entity_mut(wrapper).insert((Node::default(), Pickable::IGNORE, ChildOf(root)));
        world.entity_mut(button).insert((Node::default(), ChildOf(wrapper)));
        world.run_system_once(bevy::ui::update::update_clipping_system).unwrap();

        assert!(world.get::<CalculatedClip>(button).is_some());
        assert!(pick(&mut world, Vec2::splat(50.0), 2.0).is_some());
        assert!(pick(&mut world, Vec2::splat(35.0), 2.0).is_none());

        // An unbounded axis of a resolved scissor is valid, not invalid geometry.
        world.get_mut::<Node>(root).unwrap().overflow.y = OverflowAxis::Visible;
        world.run_system_once(bevy::ui::update::update_clipping_system).unwrap();
        assert!(pick(&mut world, Vec2::new(50.0, 35.0), 2.0).is_some());
        assert!(pick(&mut world, Vec2::new(35.0, 50.0), 2.0).is_none());

        world.entity_mut(button).insert(OverrideClip);
        world.run_system_once(bevy::ui::update::update_clipping_system).unwrap();
        assert!(world.get::<CalculatedClip>(button).is_none());
        assert!(pick(&mut world, Vec2::splat(35.0), 2.0).is_some());
    }

    #[test]
    fn resolved_stack_beats_local_z_and_entity_allocation_order() {
        let mut world = World::new();
        let top = target(&mut world, Vec2::splat(100.0), UiGlobalTransform::default(), 20);
        let bottom = target(&mut world, Vec2::splat(100.0), UiGlobalTransform::default(), 10);
        world.entity_mut(top).insert(ZIndex(-100));
        world.entity_mut(bottom).insert(ZIndex(1000));
        assert_eq!(pick(&mut world, Vec2::ZERO, 1.0).unwrap().entity, top);
        world.entity_mut(top).insert(CalculatedClip { clip: Rect::from_corners(Vec2::splat(10.0), Vec2::splat(20.0)) });
        assert_eq!(pick(&mut world, Vec2::ZERO, 1.0).unwrap().entity, bottom);
    }

    #[test]
    fn ordinary_overlay_blocks_unless_ignored_nonblocking_hidden_or_clipped() {
        let mut world = World::new();
        let button = target(&mut world, Vec2::splat(100.0), UiGlobalTransform::default(), 1);
        let overlay = spawn_node(&mut world, Vec2::splat(100.0), UiGlobalTransform::default(), 2);
        assert!(pick(&mut world, Vec2::ZERO, 1.0).is_none());
        for pickable in [Pickable::IGNORE, Pickable { is_hoverable: true, should_block_lower: false }] {
            world.entity_mut(overlay).insert(pickable);
            assert_eq!(pick(&mut world, Vec2::ZERO, 1.0).unwrap().entity, button);
        }
        world.entity_mut(overlay).insert(Pickable { is_hoverable: false, should_block_lower: true });
        assert!(pick(&mut world, Vec2::ZERO, 1.0).is_none());
        world.entity_mut(overlay).insert(InheritedVisibility::HIDDEN);
        assert_eq!(pick(&mut world, Vec2::ZERO, 1.0).unwrap().entity, button);
        world.entity_mut(overlay).insert((InheritedVisibility::VISIBLE, CalculatedClip { clip: Rect::default() }));
        assert_eq!(pick(&mut world, Vec2::ZERO, 1.0).unwrap().entity, button);
    }

    #[test]
    fn ordinary_label_resolves_through_wrapper_to_hit_tested_button_ancestor() {
        let mut world = World::new();
        let button = target(&mut world, Vec2::splat(100.0), UiGlobalTransform::default(), 1);
        let wrapper = spawn_node(&mut world, Vec2::splat(80.0), UiGlobalTransform::default(), 2);
        let label = spawn_node(&mut world, Vec2::splat(60.0), UiGlobalTransform::default(), 3);
        world.entity_mut(wrapper).insert(ChildOf(button));
        world.entity_mut(label).insert(ChildOf(wrapper));
        assert_eq!(pick(&mut world, Vec2::ZERO, 1.0).unwrap().entity, button);
        world.entity_mut(button).insert(HitShape::CustomRect(Rect::from_corners(Vec2::ZERO, Vec2::splat(10.0))));
        assert!(pick(&mut world, Vec2::ZERO, 1.0).is_none());
    }

    #[test]
    fn ignored_targets_pass_through_but_nonhoverable_targets_can_block() {
        let mut world = World::new();
        let bottom = target(&mut world, Vec2::splat(100.0), UiGlobalTransform::default(), 1);
        let top = target(&mut world, Vec2::splat(100.0), UiGlobalTransform::default(), 2);
        for behavior in [HitBehavior::None, HitBehavior::ChildrenOnly] {
            world.entity_mut(top).insert(behavior);
            assert_eq!(pick(&mut world, Vec2::ZERO, 1.0).unwrap().entity, bottom);
        }
        world.entity_mut(top).insert((HitBehavior::Auto, Pickable::IGNORE));
        assert_eq!(pick(&mut world, Vec2::ZERO, 1.0).unwrap().entity, bottom);
        world.entity_mut(top).insert(Pickable { is_hoverable: false, should_block_lower: true });
        assert!(pick(&mut world, Vec2::ZERO, 1.0).is_none());
    }

    #[test]
    fn missing_singular_nonfinite_hidden_and_empty_geometry_cannot_hit() {
        let mut world = World::new();
        let entity = target(&mut world, Vec2::splat(100.0), UiGlobalTransform::default(), 1);
        for transform in [UiGlobalTransform::from_scale(Vec2::ZERO), UiGlobalTransform::from_xy(f32::NAN, 0.0)] {
            world.entity_mut(entity).insert(transform);
            assert!(pick(&mut world, Vec2::ZERO, 1.0).is_none());
        }
        world.entity_mut(entity).remove::<UiGlobalTransform>();
        assert!(pick(&mut world, Vec2::ZERO, 1.0).is_none());
        world.entity_mut(entity).insert((UiGlobalTransform::default(), InheritedVisibility::HIDDEN));
        assert!(pick(&mut world, Vec2::ZERO, 1.0).is_none());
        world.entity_mut(entity).insert((InheritedVisibility::VISIBLE, HitSlop::all(20.0)));
        world.get_mut::<ComputedNode>(entity).unwrap().size = Vec2::ZERO;
        assert!(pick(&mut world, Vec2::ZERO, 1.0).is_none());
        world.get_mut::<ComputedNode>(entity).unwrap().size = Vec2::splat(100.0);
        assert!(pick(&mut world, Vec2::splat(f32::NAN), 1.0).is_none());
        assert!(pick(&mut world, Vec2::ZERO, 0.0).is_none());
    }
}
