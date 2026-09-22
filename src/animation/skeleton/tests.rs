use accesskit::{Live, Role};
use bevy::{a11y::AccessibilityNode, prelude::*};

use super::*;
use crate::rendering::{Paint, Shimmer, Surface};
use crate::theme::{
    AccessibilityContrastMode, AccessibilityVisualPolicy, AccessibilityVisualPolicyResource,
    ThemeChanged, ThemeMode, ThemePlugin, ThemeResource, dark_theme, light_theme,
};

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((ThemePlugin, SkeletonPlugin));
    // Environment-independent preferences; no window, GPU, assets, or Time resource.
    app.insert_resource(AccessibilityVisualPolicyResource { current: AccessibilityVisualPolicy::default() });
    app
}

fn surface(app: &App, entity: Entity) -> &Surface {
    app.world().get::<Surface>(entity).unwrap()
}

#[test]
fn skeleton_preserves_caller_node_and_changes_only_explicit_dimensions() {
    let mut app = app();
    let node = Node {
        width: Val::Percent(72.0), height: Val::Px(40.0),
        position_type: PositionType::Absolute, left: Val::Px(12.0),
        margin: UiRect::all(Val::Px(9.0)), padding: UiRect::all(Val::Px(2.0)),
        flex_grow: 1.0, flex_shrink: 0.0, min_width: Val::Px(20.0),
        border_radius: BorderRadius::all(Val::Px(17.0)),
        ..default()
    };
    let unchanged = app.world_mut().spawn((node.clone(), Skeleton::new())).id();
    let resized = app.world_mut().spawn((node.clone(), Skeleton::new().width(Val::Percent(30.0)).height(Val::Px(22.0)))).id();
    app.update();
    assert_eq!(app.world().get::<Node>(unchanged).unwrap(), &node);
    let mut expected = node;
    expected.width = Val::Percent(30.0);
    expected.height = Val::Px(22.0);
    assert_eq!(app.world().get::<Node>(resized).unwrap(), &expected);
}

#[test]
fn skeleton_circle_uses_canonical_surface_radius_and_required_node() {
    let mut app = app();
    let entity = app.world_mut().spawn(Skeleton::circle(48.0)).id();
    app.update();
    let node = app.world().get::<Node>(entity).unwrap();
    assert_eq!(node.width, Val::Px(48.0));
    assert_eq!(node.height, Val::Px(48.0));
    assert_eq!(surface(&app, entity).shape, Surface::rounded_rect_fill(24.0, Color::WHITE).shape);
    let pickable = app.world().get::<Pickable>(entity).unwrap();
    assert!(!pickable.should_block_lower && !pickable.is_hoverable);
    assert!(app.world().get::<AccessibilityNode>(entity).is_none());
}

#[test]
fn skeleton_text_follows_live_typography_but_preserves_caller_height() {
    let mut app = app();
    let text = app.world_mut().spawn(Skeleton::text().width(Val::Percent(80.0))).id();
    let caller = app.world_mut().spawn((Node { height: Val::Px(37.0), ..default() }, Skeleton::text())).id();
    let explicit = app.world_mut().spawn(Skeleton::text().height(Val::Px(11.0))).id();
    app.update();
    assert_eq!(app.world().get::<Node>(text).unwrap().height, Val::Px(20.0));
    let original_surface = surface(&app, text).clone();
    app.world_mut().resource_mut::<ThemeResource>().current.typography.font_size_body = 24.0;
    app.update();
    assert_eq!(app.world().get::<Node>(text).unwrap().height, Val::Px(30.0));
    assert_eq!(app.world().get::<Node>(caller).unwrap().height, Val::Px(37.0));
    assert_eq!(app.world().get::<Node>(explicit).unwrap().height, Val::Px(11.0));
    assert_eq!(surface(&app, text), &original_surface);
    app.world_mut().get_mut::<Node>(text).unwrap().height = Val::Px(51.0);
    app.update();
    app.world_mut().resource_mut::<ThemeResource>().current.typography.font_size_body = 18.0;
    app.update();
    assert_eq!(app.world().get::<Node>(text).unwrap().height, Val::Px(51.0));
}

#[test]
fn skeleton_text_lines_have_one_busy_polite_region_and_optional_final_width() {
    let mut app = app();
    let group = Skeleton::text_lines(3).label("Loading messages")
        .width(Val::Px(240.0)).gap(10.0).final_width(Val::Percent(60.0))
        .spawn(&mut app.world_mut().commands());
    app.world_mut().flush();
    app.update();
    assert!(surface_optional(&app, group).is_none());
    let node = app.world().get::<Node>(group).unwrap();
    assert_eq!(node.flex_direction, FlexDirection::Column);
    assert_eq!(node.row_gap, Val::Px(10.0));
    assert_eq!(node.width, Val::Px(240.0));
    let accessibility = &app.world().get::<AccessibilityNode>(group).unwrap().0;
    assert_eq!(accessibility.role(), Role::Group);
    assert_eq!(accessibility.label(), Some("Loading messages"));
    assert!(accessibility.is_busy());
    assert_eq!(accessibility.live(), Some(Live::Polite));
    let children: Vec<Entity> = app.world().get::<Children>(group).unwrap().iter().collect();
    assert_eq!(children.len(), 3);
    for (index, child) in children.into_iter().enumerate() {
        assert_eq!(app.world().get::<Node>(child).unwrap().width, Val::Percent(if index == 2 { 60.0 } else { 100.0 }));
        assert!(app.world().get::<AccessibilityNode>(child).is_none());
        assert!(!app.world().get::<Pickable>(child).unwrap().should_block_lower);
    }
    app.world_mut().get_mut::<SkeletonGroup>(group).unwrap().busy = false;
    app.world_mut().get_mut::<SkeletonGroup>(group).unwrap().label = "Messages loaded".into();
    app.update();
    let accessibility = &app.world().get::<AccessibilityNode>(group).unwrap().0;
    assert!(!accessibility.is_busy());
    assert_eq!(accessibility.label(), Some("Messages loaded"));
}

fn surface_optional(app: &App, entity: Entity) -> Option<&Surface> { app.world().get::<Surface>(entity) }

#[test]
fn skeleton_group_preserves_custom_layout_and_handles_empty_and_single_line() {
    let mut app = app();
    let node = Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(17.0), ..default() };
    let direct = app.world_mut().spawn((node.clone(), SkeletonGroup::new("Loading profile"))).id();
    let empty = Skeleton::text_lines(0).spawn(&mut app.world_mut().commands());
    let one = Skeleton::text_lines(1).final_width(Val::Percent(42.0)).spawn(&mut app.world_mut().commands());
    app.world_mut().flush();
    app.update();
    assert_eq!(app.world().get::<Node>(direct).unwrap(), &node);
    assert!(app.world().get::<Children>(empty).is_none_or(|children| children.is_empty()));
    let only = app.world().get::<Children>(one).unwrap()[0];
    assert_eq!(app.world().get::<Node>(only).unwrap().width, Val::Percent(42.0));
}

#[derive(Resource, Default)]
struct Writes { surfaces: usize, nodes: usize, accessibility: usize }

fn count_changes(
    surfaces: Query<(), (With<Skeleton>, Changed<Surface>)>,
    nodes: Query<(), (With<Skeleton>, Changed<Node>)>,
    accessibility: Query<(), (With<SkeletonGroup>, Changed<AccessibilityNode>)>,
    mut writes: ResMut<Writes>,
) {
    writes.surfaces = surfaces.iter().count();
    writes.nodes = nodes.iter().count();
    writes.accessibility = accessibility.iter().count();
}

#[test]
fn skeleton_theme_switch_updates_in_same_frame_and_steady_state_is_clean() {
    let mut app = app();
    app.init_resource::<Writes>().add_systems(Last, count_changes);
    let entity = app.world_mut().spawn(Skeleton::text()).id();
    app.world_mut().spawn(SkeletonGroup::new("Loading"));
    app.update();
    let light = surface(&app, entity).clone();
    app.world_mut().write_message(ThemeChanged { mode: ThemeMode::Dark });
    app.update();
    assert_ne!(surface(&app, entity), &light);
    let Paint::Shimmer(shimmer) = &surface(&app, entity).fill else { panic!("expected native shimmer") };
    assert_eq!(shimmer.base_color, dark_theme().colors.skeleton_base);
    assert_eq!(shimmer.highlight_color, dark_theme().colors.skeleton_highlight);
    assert_eq!(app.world().resource::<Writes>().surfaces, 1);
    for _ in 0..4 {
        app.update();
        let writes = app.world().resource::<Writes>();
        assert_eq!((writes.surfaces, writes.nodes, writes.accessibility), (0, 0, 0));
    }
    app.world_mut().resource_mut::<ThemeResource>().current.colors.primary = Color::BLACK;
    app.update();
    assert_eq!(app.world().resource::<Writes>().surfaces, 0);
    app.world_mut().resource_mut::<ThemeResource>().current.typography.font_size_body = 22.0;
    app.update();
    assert_eq!(app.world().resource::<Writes>().surfaces, 0);
    assert_eq!(app.world().resource::<Writes>().nodes, 1);
}

#[test]
fn skeleton_accessibility_and_shared_motion_policy_disable_shimmer_live() {
    let mut app = app();
    let entity = app.world_mut().spawn(Skeleton::new()).id();
    app.update();
    for preference in 0..4 {
        let mut policy = AccessibilityVisualPolicy::default();
        match preference {
            0 => policy.reduced_motion = true,
            1 => policy.reduced_effects = true,
            2 => policy.contrast = AccessibilityContrastMode::High,
            _ => app.world_mut().resource_mut::<ThemeResource>().current.visual_effects.reduced_effects = true,
        }
        app.world_mut().resource_mut::<AccessibilityVisualPolicyResource>().current = policy;
        app.update();
        let Paint::Solid(color) = surface(&app, entity).fill else { panic!("motion/effects preference must produce solid paint") };
        if preference == 2 { assert_eq!(color, light_theme().colors.text_muted.with_alpha(1.0)); }
        app.world_mut().resource_mut::<ThemeResource>().current.visual_effects.reduced_effects = false;
        app.world_mut().resource_mut::<AccessibilityVisualPolicyResource>().current = AccessibilityVisualPolicy::default();
        app.update();
        assert!(matches!(surface(&app, entity).fill, Paint::Shimmer(_)));
    }
    app.world_mut().get_mut::<Skeleton>(entity).unwrap().enabled = false;
    app.update();
    assert!(matches!(surface(&app, entity).fill, Paint::Solid(_)));
    assert_eq!(app.world().get::<Node>(entity).unwrap().display, Display::Flex);
}

#[test]
fn skeleton_overrides_follow_transparency_policy_and_shimmer_contract() {
    let mut app = app();
    let base = Color::srgba(0.3, 0.4, 0.5, 0.2);
    let highlight = Color::srgba(0.8, 0.9, 1.0, 0.5);
    let entity = app.world_mut().spawn(Skeleton::new().base_color(base).highlight_color(highlight)
        .direction(SkeletonDirection::RightToLeft).duration(2.3).phase(0.4)
        .highlight_width(0.3).highlight_softness(0.6).highlight_intensity(0.7)).id();
    app.update();
    let mut expected = Shimmer::new(base, highlight);
    expected.direction = SkeletonDirection::RightToLeft;
    expected.duration = 2.3; expected.phase = 0.4; expected.width = 0.3;
    expected.softness = 0.6; expected.intensity = 0.7;
    assert_eq!(surface(&app, entity).fill, Paint::Shimmer(expected.sanitized()));
    app.world_mut().resource_mut::<ThemeResource>().current = dark_theme();
    app.update();
    let Paint::Shimmer(shimmer) = &surface(&app, entity).fill else { panic!("expected shimmer") };
    assert_eq!(shimmer.base_color, base);
    app.world_mut().resource_mut::<AccessibilityVisualPolicyResource>().current.reduced_transparency = true;
    app.update();
    let Paint::Shimmer(shimmer) = &surface(&app, entity).fill else { panic!("transparency reduction need not stop motion") };
    assert_eq!(shimmer.base_color.alpha(), 1.0);
    assert_eq!(shimmer.highlight_color.alpha(), 1.0);
    let reset = app.world().get::<Skeleton>(entity).unwrap().clone().theme_colors();
    app.world_mut().entity_mut(entity).insert(reset);
    app.update();
    let Paint::Shimmer(shimmer) = &surface(&app, entity).fill else { panic!("expected shimmer") };
    assert_eq!(shimmer.base_color, dark_theme().colors.skeleton_base);
}

#[test]
fn skeleton_geometry_and_shimmer_inputs_are_sanitized() {
    let mut app = app();
    let entity = app.world_mut().spawn(Skeleton::circle(f32::NAN).radius(f32::INFINITY)
        .duration(f32::NAN).phase(f32::INFINITY).highlight_width(-1.0)
        .highlight_softness(f32::NAN).highlight_intensity(f32::INFINITY)).id();
    app.update();
    assert_eq!(app.world().get::<Node>(entity).unwrap().width, Val::Px(0.0));
    let Paint::Shimmer(shimmer) = &surface(&app, entity).fill else { panic!("expected shimmer") };
    assert!(shimmer.duration.is_finite() && shimmer.duration > 0.0);
    assert!(shimmer.phase.is_finite() && shimmer.width.is_finite());
    assert!(shimmer.softness.is_finite() && shimmer.intensity.is_finite());
    assert_eq!(surface(&app, entity).shape, Surface::rounded_rect_fill(6.0, Color::WHITE).shape);
}