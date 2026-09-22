use std::fmt::Debug;
pub mod spring;
use std::time::Duration;

use bevy::{
    color::LinearRgba,
    prelude::*,
};

use crate::rendering::{
    AngularGradient,
    Backdrop,
    Border,
    BorderWidths,
    CornerRadii,
    Decorations,
    Effects,
    FocusRing,
    FocusRingLayer,
    GradientStop,
    InnerShadow,
    LinearGradient,
    OuterGlow,
    OuterShadow,
    Paint,
    RadialGradient,
    RoundedRect,
    Shape,
    Surface,
};
use crate::icons::IconNode;
use crate::theme::{AccessibilityVisualPolicyResource, ThemeResource};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotionClass {
    Decorative,
    Semantic,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Smooth,
    CubicBezier { x1: f32, y1: f32, x2: f32, y2: f32 },
}

impl Easing {
    pub fn sample(self, progress: f32) -> f32 {
        let p = sanitize_unit(progress);
        let eased = match self {
            Self::Linear => p,
            Self::EaseIn => p * p * p,
            Self::EaseOut => {
                let u = 1.0 - p;
                1.0 - u * u * u
            }
            Self::EaseInOut => {
                if p < 0.5 {
                    4.0 * p * p * p
                } else {
                    let u = -2.0 * p + 2.0;
                    1.0 - (u * u * u) * 0.5
                }
            }
            Self::Smooth => p * p * (3.0 - 2.0 * p),
            Self::CubicBezier { x1, y1, x2, y2 } => sample_cubic_bezier(x1, y1, x2, y2, p),
        };

        sanitize_unit(eased)
    }
}

impl Default for Easing {
    fn default() -> Self {
        Self::EaseOut
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transition {
    pub duration: Duration,
    pub delay: Duration,
    pub easing: Easing,
    pub motion_class: MotionClass,
}

impl Transition {
    pub fn new(duration: Duration, easing: Easing) -> Self {
        Self {
            duration,
            delay: Duration::ZERO,
            easing,
            motion_class: MotionClass::Decorative,
        }
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    pub fn with_motion_class(mut self, motion_class: MotionClass) -> Self {
        self.motion_class = motion_class;
        self
    }

    fn effective_for_policy(self, policy: &AccessibilityVisualPolicyResource) -> Self {
        if policy.current.reduced_motion && self.motion_class == MotionClass::Decorative {
            Self {
                duration: Duration::ZERO,
                delay: Duration::ZERO,
                ..self
            }
        } else {
            self
        }
    }

    fn duration_secs(self) -> f32 {
        sanitize_non_negative(self.duration.as_secs_f32())
    }

    fn delay_secs(self) -> f32 {
        sanitize_non_negative(self.delay.as_secs_f32())
    }
}

impl Default for Transition {
    fn default() -> Self {
        Self {
            duration: Duration::from_millis(140),
            delay: Duration::ZERO,
            easing: Easing::EaseOut,
            motion_class: MotionClass::Decorative,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationLifecycle {
    Pending,
    Running,
    Completed,
    Cancelled,
    Replaced,
}

pub trait Animatable: Clone + Debug + PartialEq + Send + Sync + 'static {
    fn interpolate(from: &Self, to: &Self, t: f32) -> Self;
}

#[derive(Clone, Debug)]
pub struct AnimationTrack<T: Animatable> {
    start: T,
    current: T,
    target: T,
    transition: Transition,
    elapsed_secs: f32,
    state: AnimationLifecycle,
}

impl<T: Animatable> AnimationTrack<T> {
    pub fn from_value(value: T) -> Self {
        Self {
            start: value.clone(),
            current: value.clone(),
            target: value,
            transition: Transition::default(),
            elapsed_secs: 0.0,
            state: AnimationLifecycle::Completed,
        }
    }

    pub fn target(&self) -> &T {
        &self.target
    }

    pub fn current(&self) -> &T {
        &self.current
    }

    pub fn transition(&self) -> Transition {
        self.transition
    }

    pub fn elapsed_secs(&self) -> f32 {
        self.elapsed_secs
    }

    pub fn state(&self) -> AnimationLifecycle {
        self.state
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self.state,
            AnimationLifecycle::Pending | AnimationLifecycle::Running | AnimationLifecycle::Replaced
        )
    }

    pub fn cancel(&mut self) {
        self.state = AnimationLifecycle::Cancelled;
    }

    pub fn set_target(&mut self, target: T, transition: Transition) {
        if self.current == target {
            self.start = target.clone();
            self.target = target.clone();
            self.current = target;
            self.transition = transition;
            self.elapsed_secs = 0.0;
            self.state = AnimationLifecycle::Completed;
            return;
        }

        self.start = self.current.clone();
        self.target = target;
        self.transition = transition;
        self.elapsed_secs = 0.0;
        self.state = if self.is_active() {
            AnimationLifecycle::Replaced
        } else if self.transition.delay_secs() > 0.0 {
            AnimationLifecycle::Pending
        } else {
            AnimationLifecycle::Running
        };
    }

    pub fn tick(&mut self, delta_secs: f32) -> bool {
        if matches!(self.state, AnimationLifecycle::Completed | AnimationLifecycle::Cancelled) {
            return false;
        }

        if self.state == AnimationLifecycle::Replaced {
            self.state = if self.transition.delay_secs() > 0.0 {
                AnimationLifecycle::Pending
            } else {
                AnimationLifecycle::Running
            };
        }

        let delta = sanitize_non_negative(delta_secs);
        if delta <= 0.0 {
            return false;
        }

        self.elapsed_secs += delta;

        let delay = self.transition.delay_secs();
        if self.elapsed_secs < delay {
            self.state = AnimationLifecycle::Pending;
            return false;
        }

        let local = self.elapsed_secs - delay;
        let duration = self.transition.duration_secs();

        if duration <= 0.0 {
            if self.current != self.target {
                self.current = self.target.clone();
                self.state = AnimationLifecycle::Completed;
                return true;
            }
            self.state = AnimationLifecycle::Completed;
            return false;
        }

        self.state = AnimationLifecycle::Running;

        let t = sanitize_unit(local / duration);
        let eased = self.transition.easing.sample(t);
        let next = T::interpolate(&self.start, &self.target, eased);
        let changed = next != self.current;
        self.current = next;

        if t >= 1.0 {
            self.current = self.target.clone();
            self.state = AnimationLifecycle::Completed;
        }

        changed
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiAnimationDirtyKind {
    TransformDirty,
    PaintDirty,
    EffectDirty,
    LayoutDirty,
    TextLayoutDirty,
    GlyphDirty,
}

impl UiAnimationDirtyKind {
    const fn bit(self) -> u32 {
        match self {
            Self::TransformDirty => 1 << 0,
            Self::PaintDirty => 1 << 1,
            Self::EffectDirty => 1 << 2,
            Self::LayoutDirty => 1 << 3,
            Self::TextLayoutDirty => 1 << 4,
            Self::GlyphDirty => 1 << 5,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UiAnimationDirtyFlags(u32);

impl UiAnimationDirtyFlags {
    pub fn clear(&mut self) {
        self.0 = 0;
    }

    pub fn mark(&mut self, kind: UiAnimationDirtyKind) {
        self.0 |= kind.bit();
    }

    pub fn contains(&self, kind: UiAnimationDirtyKind) -> bool {
        self.0 & kind.bit() != 0
    }
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct UiAnimationClock {
    elapsed_secs: f32,
    fixed_step: Option<f32>,
    frame_delta_secs: f32,
}

impl UiAnimationClock {
    pub fn set_fixed_step(&mut self, delta_secs: f32) {
        self.fixed_step = Some(sanitize_non_negative(delta_secs));
    }

    pub fn clear_fixed_step(&mut self) {
        self.fixed_step = None;
    }

    pub fn elapsed_secs(self) -> f32 {
        self.elapsed_secs
    }

    pub fn frame_delta_secs(self) -> f32 {
        self.frame_delta_secs
    }
}

impl Default for UiAnimationClock {
    fn default() -> Self {
        Self {
            elapsed_secs: 0.0,
            fixed_step: None,
            frame_delta_secs: 0.0,
        }
    }
}

#[derive(Component, Clone, Debug, PartialEq)]
pub struct SurfaceTransitionTarget {
    pub target: Surface,
    pub transition: Transition,
    pub group: Option<u64>,
}

impl SurfaceTransitionTarget {
    pub fn new(target: Surface, transition: Transition) -> Self {
        Self {
            target,
            transition,
            group: None,
        }
    }

    pub fn in_group(mut self, group: u64) -> Self {
        self.group = Some(group);
        self
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct TextColorTransitionTarget {
    pub target: Color,
    pub transition: Transition,
    pub group: Option<u64>,
}

impl TextColorTransitionTarget {
    pub fn new(target: Color, transition: Transition) -> Self {
        Self {
            target,
            transition,
            group: None,
        }
    }

    pub fn in_group(mut self, group: u64) -> Self {
        self.group = Some(group);
        self
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub(crate) struct IconColorTransitionTarget {
    pub target: Color,
    pub transition: Transition,
    pub group: Option<u64>,
}

impl IconColorTransitionTarget {
    #[allow(dead_code)] // builder API kept for future icon-color-transition callers
    pub fn new(target: Color, transition: Transition) -> Self {
        Self {
            target,
            transition,
            group: None,
        }
    }

    #[allow(dead_code)]
    pub fn in_group(mut self, group: u64) -> Self {
        self.group = Some(group);
        self
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct TransformTransitionTarget {
    /// For `Node` entities, XY translation is in logical pixels, XY scale and
    /// Z rotation animate `UiTransform`; depth and out-of-plane rotation are
    /// unsupported. Non-UI entities retain the full 3D `Transform` target.
    pub target: Transform,
    pub transition: Transition,
    pub group: Option<u64>,
}

impl TransformTransitionTarget {
    pub fn new(target: Transform, transition: Transition) -> Self {
        Self {
            target,
            transition,
            group: None,
        }
    }

    pub fn in_group(mut self, group: u64) -> Self {
        self.group = Some(group);
        self
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct NodeLeftPercentTransitionTarget {
    pub target_percent: f32,
    pub transition: Transition,
    pub group: Option<u64>,
}

impl NodeLeftPercentTransitionTarget {
    pub fn new(target_percent: f32, transition: Transition) -> Self {
        Self {
            target_percent,
            transition,
            group: None,
        }
    }

    pub fn in_group(mut self, group: u64) -> Self {
        self.group = Some(group);
        self
    }
}

#[derive(Component, Clone, Debug)]
struct SurfaceAnimationTrack(AnimationTrack<Surface>);

#[derive(Component, Clone, Debug)]
struct TextColorAnimationTrack(AnimationTrack<Color>);

#[derive(Component, Clone, Debug)]
struct IconColorAnimationTrack(AnimationTrack<Color>);

#[derive(Component, Clone, Debug)]
struct TransformAnimationTrack {
    translation: AnimationTrack<Vec3>,
    rotation: AnimationTrack<Quat>,
    scale: AnimationTrack<Vec3>,
    transition: Transition,
}

#[derive(Component, Clone, Debug)]
struct NodeLeftPercentAnimationTrack(AnimationTrack<f32>);

#[derive(Clone, Debug)]
pub struct UiAnimationDebugEntry {
    pub entity: Entity,
    pub property: &'static str,
    pub state: AnimationLifecycle,
    pub elapsed_secs: f32,
    pub duration_secs: f32,
    pub delay_secs: f32,
    pub easing: Easing,
    pub start: String,
    pub current: String,
    pub target: String,
}

#[derive(Resource, Default)]
pub struct UiAnimationDebugRegistry {
    pub active: Vec<UiAnimationDebugEntry>,
}

impl UiAnimationDebugRegistry {
    pub fn summary(&self) -> String {
        if self.active.is_empty() {
            return "no active animations".to_string();
        }

        let mut out = format!("active animations: {}", self.active.len());
        for entry in &self.active {
            out.push_str(&format!(
                "\nentity={:?} property={} state={:?} t={:.3}s/{:.3}s delay={:.3}s easing={:?} current={} target={}",
                entry.entity,
                entry.property,
                entry.state,
                entry.elapsed_secs,
                entry.duration_secs,
                entry.delay_secs,
                entry.easing,
                entry.current,
                entry.target
            ));
        }
        out
    }
}

pub struct UiAnimationPlugin;

impl Plugin for UiAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiAnimationClock>()
            .init_resource::<UiAnimationDebugRegistry>()
            .add_systems(
                PreUpdate,
                (
                    tick_animation_clock,
                    clear_dirty_flags,
                    sync_surface_transition_targets,
                    sync_text_transition_targets,
                    sync_icon_color_transition_targets,
                    sync_transform_transition_targets,
                    sync_node_left_transition_targets,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    run_surface_animations,
                    run_text_color_animations,
                    run_icon_color_animations,
                    run_transform_animations,
                    run_node_left_animations,
                    rebuild_animation_debug_registry,
                )
                    .chain(),
            );
    }
}

fn tick_animation_clock(time: Res<Time>, mut clock: ResMut<UiAnimationClock>) {
    let delta = clock
        .fixed_step
        .unwrap_or_else(|| sanitize_non_negative(time.delta_secs()));
    clock.frame_delta_secs = delta;
    clock.elapsed_secs += delta;
}

fn clear_dirty_flags(mut query: Query<&mut UiAnimationDirtyFlags>) {
    for mut flags in &mut query {
        flags.clear();
    }
}

fn sync_surface_transition_targets(
    mut commands: Commands,
    policy: Res<AccessibilityVisualPolicyResource>,
    mut query: Query<(
        Entity,
        &Surface,
        &SurfaceTransitionTarget,
        Option<&mut SurfaceAnimationTrack>,
    )>,
) {
    for (entity, current_surface, target, existing_track) in &mut query {
        let mut target_surface = target.target.clone();
        if policy.current.reduced_transparency {
            target_surface = apply_reduced_transparency_surface(target_surface);
        }

        let transition = target.transition.effective_for_policy(&policy);

        if let Some(mut existing_track) = existing_track {
            if existing_track.0.target() != &target_surface || existing_track.0.transition() != transition {
                existing_track.0.set_target(target_surface, transition);
            }
            continue;
        }

        let mut track = AnimationTrack::from_value(current_surface.clone());
        track.set_target(target_surface, transition);
        commands.entity(entity).insert(SurfaceAnimationTrack(track));
    }
}

fn sync_text_transition_targets(
    mut commands: Commands,
    policy: Res<AccessibilityVisualPolicyResource>,
    mut query: Query<(
        Entity,
        &TextColor,
        &TextColorTransitionTarget,
        Option<&mut TextColorAnimationTrack>,
    )>,
) {
    for (entity, current, target, existing_track) in &mut query {
        let mut target_color = target.target;
        if policy.current.reduced_transparency {
            target_color = with_min_alpha(target_color, 0.92);
        }
        let transition = target.transition.effective_for_policy(&policy);

        if let Some(mut existing_track) = existing_track {
            if existing_track.0.target() != &target_color || existing_track.0.transition() != transition {
                existing_track.0.set_target(target_color, transition);
            }
            continue;
        }

        let mut track = AnimationTrack::from_value(current.0);
        track.set_target(target_color, transition);
        commands.entity(entity).insert(TextColorAnimationTrack(track));
    }
}

fn sync_icon_color_transition_targets(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &IconNode,
        &IconColorTransitionTarget,
        Option<&mut IconColorAnimationTrack>,
    )>,
) {
    for (entity, current, target, existing_track) in &mut query {
        if let Some(mut existing_track) = existing_track {
            if existing_track.0.target() != &target.target || existing_track.0.transition() != target.transition {
                existing_track.0.set_target(target.target, target.transition);
            }
            continue;
        }

        let mut track = AnimationTrack::from_value(current.color);
        track.set_target(target.target, target.transition);
        commands.entity(entity).insert(IconColorAnimationTrack(track));
    }
}

fn sync_transform_transition_targets(
    mut commands: Commands,
    policy: Res<AccessibilityVisualPolicyResource>,
    mut query: Query<(
        Entity,
        Option<&Transform>,
        Has<Node>,
        Option<(&UiTransform, &ComputedNode, &ComputedUiRenderTargetInfo)>,
        &TransformTransitionTarget,
        Option<&mut TransformAnimationTrack>,
    )>,
) {
    for (entity, transform, is_ui, ui, target, existing_track) in &mut query {
        let transition = target.transition.effective_for_policy(&policy);
        let target_transform = if is_ui {
            Transform {
                translation: target.target.translation.truncate().extend(0.0),
                rotation: Quat::from_rotation_z(target.target.rotation.to_euler(EulerRot::XYZ).2),
                scale: target.target.scale.truncate().extend(1.0),
            }
        } else {
            target.target
        };

        if let Some(mut existing_track) = existing_track {
            if existing_track.transition != transition
                || existing_track.translation.target() != &target_transform.translation
                || existing_track.rotation.target() != &target_transform.rotation
                || existing_track.scale.target() != &target_transform.scale
            {
                existing_track.transition = transition;
                existing_track.translation.set_target(target_transform.translation, transition);
                existing_track.rotation.set_target(target_transform.rotation, transition);
                existing_track.scale.set_target(target_transform.scale, transition);
            }
            continue;
        }

        // A Node's visible transform is authoritative, even if it also carries
        // a legacy Transform. Resolve responsive offsets to logical pixels once
        // when starting the pixel-based public transition.
        let current = if is_ui {
            let Some((ui, computed, render_target)) = ui else {
                continue;
            };
            Transform {
                translation: ui.translation.resolve(
                    1.0,
                    computed.size() * computed.inverse_scale_factor(),
                    render_target.logical_size(),
                ).extend(0.0),
                rotation: Quat::from_rotation_z(ui.rotation.as_radians()),
                scale: ui.scale.extend(1.0),
            }
        } else if let Some(transform) = transform {
            *transform
        } else {
            continue;
        };

        let mut translation = AnimationTrack::from_value(current.translation);
        translation.set_target(target_transform.translation, transition);

        let mut rotation = AnimationTrack::from_value(current.rotation);
        rotation.set_target(target_transform.rotation, transition);

        let mut scale = AnimationTrack::from_value(current.scale);
        scale.set_target(target_transform.scale, transition);

        commands.entity(entity).insert(TransformAnimationTrack {
            translation,
            rotation,
            scale,
            transition,
        });
    }
}

fn sync_node_left_transition_targets(
    mut commands: Commands,
    policy: Res<AccessibilityVisualPolicyResource>,
    mut query: Query<(
        Entity,
        &Node,
        &NodeLeftPercentTransitionTarget,
        Option<&mut NodeLeftPercentAnimationTrack>,
    )>,
) {
    for (entity, node, target, existing_track) in &mut query {
        let transition = target.transition.effective_for_policy(&policy);
        let current_left = match node.left {
            Val::Percent(v) => v,
            Val::Px(v) => v,
            _ => 0.0,
        };
        let target_percent = sanitize_finite(target.target_percent, current_left);

        if let Some(mut existing_track) = existing_track {
            if existing_track.0.target() != &target_percent || existing_track.0.transition() != transition {
                existing_track.0.set_target(target_percent, transition);
            }
            continue;
        }

        let mut track = AnimationTrack::from_value(current_left);
        track.set_target(target_percent, transition);
        commands.entity(entity).insert(NodeLeftPercentAnimationTrack(track));
    }
}

fn run_surface_animations(
    mut commands: Commands,
    clock: Res<UiAnimationClock>,
    mut query: Query<(
        Entity,
        &mut Surface,
        &mut SurfaceAnimationTrack,
        Option<&mut UiAnimationDirtyFlags>,
    )>,
) {
    for (entity, mut surface, mut track, mut dirty) in &mut query {
        let changed = track.0.tick(clock.frame_delta_secs());
        if changed {
            *surface = track.0.current().clone();
            mark_dirty(&mut dirty, UiAnimationDirtyKind::PaintDirty);
            mark_dirty(&mut dirty, UiAnimationDirtyKind::EffectDirty);
        }

        if matches!(track.0.state(), AnimationLifecycle::Completed | AnimationLifecycle::Cancelled)
        {
            commands.entity(entity).remove::<SurfaceAnimationTrack>();
        }
    }
}

fn run_text_color_animations(
    mut commands: Commands,
    clock: Res<UiAnimationClock>,
    mut query: Query<(
        Entity,
        &mut TextColor,
        &mut TextColorAnimationTrack,
        Option<&mut UiAnimationDirtyFlags>,
    )>,
) {
    for (entity, mut color, mut track, mut dirty) in &mut query {
        let changed = track.0.tick(clock.frame_delta_secs());
        if changed {
            color.0 = *track.0.current();
            mark_dirty(&mut dirty, UiAnimationDirtyKind::PaintDirty);
        }

        if matches!(track.0.state(), AnimationLifecycle::Completed | AnimationLifecycle::Cancelled)
        {
            commands.entity(entity).remove::<TextColorAnimationTrack>();
        }
    }
}

fn run_icon_color_animations(
    mut commands: Commands,
    clock: Res<UiAnimationClock>,
    mut query: Query<(Entity, &mut IconNode, &mut IconColorAnimationTrack)>,
) {
    for (entity, mut icon, mut track) in &mut query {
        if track.0.tick(clock.frame_delta_secs()) {
            icon.color = *track.0.current();
        }

        if matches!(track.0.state(), AnimationLifecycle::Completed | AnimationLifecycle::Cancelled)
        {
            commands.entity(entity).remove::<IconColorAnimationTrack>();
        }
    }
}

fn run_transform_animations(
    mut commands: Commands,
    clock: Res<UiAnimationClock>,
    mut query: Query<(
        Entity,
        Option<&mut Transform>,
        Has<Node>,
        Option<&mut UiTransform>,
        &mut TransformAnimationTrack,
        Option<&mut UiAnimationDirtyFlags>,
    )>,
) {
    for (entity, transform, is_ui, ui_transform, mut track, mut dirty) in &mut query {
        let translation_changed = track.translation.tick(clock.frame_delta_secs());
        let rotation_changed = track.rotation.tick(clock.frame_delta_secs());
        let scale_changed = track.scale.tick(clock.frame_delta_secs());

        // Update runs before Bevy's PostUpdate UI layout, which consumes these
        // writes directly. Do not write the ignored 3D Transform on UI nodes.
        if is_ui {
            if let Some(mut transform) = ui_transform {
                if translation_changed {
                    let translation = track.translation.current();
                    transform.translation = Val2::px(translation.x, translation.y);
                }
                if rotation_changed {
                    transform.rotation = Rot2::radians(track.rotation.current().to_euler(EulerRot::XYZ).2);
                }
                if scale_changed {
                    transform.scale = track.scale.current().truncate();
                }
            }
        } else if let Some(mut transform) = transform {
            if translation_changed {
                transform.translation = *track.translation.current();
            }
            if rotation_changed {
                transform.rotation = *track.rotation.current();
            }
            if scale_changed {
                transform.scale = *track.scale.current();
            }
        }

        if translation_changed || rotation_changed || scale_changed {
            mark_dirty(&mut dirty, UiAnimationDirtyKind::TransformDirty);
        }

        if !track.translation.is_active() && !track.rotation.is_active() && !track.scale.is_active() {
            commands.entity(entity).remove::<TransformAnimationTrack>();
        }
    }
}

fn run_node_left_animations(
    mut commands: Commands,
    clock: Res<UiAnimationClock>,
    mut query: Query<(
        Entity,
        &mut Node,
        &mut NodeLeftPercentAnimationTrack,
        Option<&mut UiAnimationDirtyFlags>,
    )>,
) {
    for (entity, mut node, mut track, mut dirty) in &mut query {
        let changed = track.0.tick(clock.frame_delta_secs());
        if changed {
            node.left = Val::Percent(*track.0.current());
            mark_dirty(&mut dirty, UiAnimationDirtyKind::TransformDirty);
        }

        if matches!(track.0.state(), AnimationLifecycle::Completed | AnimationLifecycle::Cancelled)
        {
            commands
                .entity(entity)
                .remove::<NodeLeftPercentAnimationTrack>();
        }
    }
}

fn rebuild_animation_debug_registry(
    mut registry: ResMut<UiAnimationDebugRegistry>,
    surfaces: Query<(Entity, &SurfaceAnimationTrack)>,
    text: Query<(Entity, &TextColorAnimationTrack)>,
    transforms: Query<(Entity, &TransformAnimationTrack)>,
    node_left: Query<(Entity, &NodeLeftPercentAnimationTrack)>,
) {
    registry.active.clear();

    for (entity, track) in &surfaces {
        if !track.0.is_active() {
            continue;
        }
        registry.active.push(debug_entry(entity, "surface", &track.0));
    }

    for (entity, track) in &text {
        if !track.0.is_active() {
            continue;
        }
        registry.active.push(debug_entry(entity, "text_color", &track.0));
    }

    for (entity, track) in &node_left {
        if !track.0.is_active() {
            continue;
        }
        registry.active.push(debug_entry(entity, "node_left_percent", &track.0));
    }

    for (entity, track) in &transforms {
        if track.translation.is_active() {
            registry
                .active
                .push(debug_entry(entity, "transform.translation", &track.translation));
        }
        if track.rotation.is_active() {
            registry
                .active
                .push(debug_entry(entity, "transform.rotation", &track.rotation));
        }
        if track.scale.is_active() {
            registry
                .active
                .push(debug_entry(entity, "transform.scale", &track.scale));
        }
    }
}

fn debug_entry<T: Animatable>(entity: Entity, property: &'static str, track: &AnimationTrack<T>) -> UiAnimationDebugEntry {
    UiAnimationDebugEntry {
        entity,
        property,
        state: track.state(),
        elapsed_secs: track.elapsed_secs(),
        duration_secs: track.transition().duration_secs(),
        delay_secs: track.transition().delay_secs(),
        easing: track.transition().easing,
        start: format!("{:?}", track.start),
        current: format!("{:?}", track.current()),
        target: format!("{:?}", track.target()),
    }
}

fn mark_dirty(dirty: &mut Option<Mut<UiAnimationDirtyFlags>>, kind: UiAnimationDirtyKind) {
    if let Some(flags) = dirty.as_mut() {
        flags.mark(kind);
    }
}

fn apply_reduced_transparency_surface(mut surface: Surface) -> Surface {
    surface.backdrop = None;

    surface.fill = with_min_paint_alpha(surface.fill, 0.92);

    if let Some(border) = surface.border.as_mut() {
        border.paint = with_min_paint_alpha(border.paint.clone(), 0.92);
    }

    if let Some(ring) = surface.decorations.focus_ring.as_mut() {
        ring.primary.paint = with_min_paint_alpha(ring.primary.paint.clone(), 0.92);
        if let Some(secondary) = ring.secondary.as_mut() {
            secondary.paint = with_min_paint_alpha(secondary.paint.clone(), 0.92);
        }
    }

    if let Some(noise) = surface.noise.as_mut() {
        noise.strength = 0.0;
        noise.animated = false;
    }

    surface
}

fn with_min_paint_alpha(paint: Paint, min_alpha: f32) -> Paint {
    match paint {
        Paint::Solid(color) => Paint::solid(with_min_alpha(color, min_alpha)),
        Paint::Shimmer(mut shimmer) => {
            shimmer.base_color = with_min_alpha(shimmer.base_color, min_alpha);
            shimmer.highlight_color = with_min_alpha(shimmer.highlight_color, min_alpha);
            Paint::Shimmer(shimmer)
        }
        Paint::LinearGradient(mut gradient) => {
            for stop in &mut gradient.stops {
                stop.color = with_min_alpha(stop.color, min_alpha);
            }
            Paint::linear(gradient)
        }
        Paint::RadialGradient(mut gradient) => {
            for stop in &mut gradient.stops {
                stop.color = with_min_alpha(stop.color, min_alpha);
            }
            Paint::radial(gradient)
        }
        Paint::AngularGradient(mut gradient) => {
            for stop in &mut gradient.stops {
                stop.color = with_min_alpha(stop.color, min_alpha);
            }
            Paint::angular(gradient)
        }
    }
}

fn with_min_alpha(color: Color, min_alpha: f32) -> Color {
    let linear = color.to_linear();
    Color::linear_rgba(linear.red, linear.green, linear.blue, linear.alpha.max(min_alpha))
}

fn sample_cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, p: f32) -> f32 {
    let x1 = sanitize_unit(x1);
    let y1 = sanitize_unit(y1);
    let x2 = sanitize_unit(x2);
    let y2 = sanitize_unit(y2);

    let mut t = p;
    for _ in 0..6 {
        let x = cubic_bezier(t, 0.0, x1, x2, 1.0);
        let dx = cubic_bezier_derivative(t, 0.0, x1, x2, 1.0);
        if dx.abs() < 1e-6 {
            break;
        }
        t -= (x - p) / dx;
        t = sanitize_unit(t);
    }

    // Binary fallback protects against near-flat derivatives.
    let mut lo = 0.0_f32;
    let mut hi = 1.0_f32;
    for _ in 0..8 {
        let x = cubic_bezier(t, 0.0, x1, x2, 1.0);
        if (x - p).abs() <= 1e-5 {
            break;
        }
        if x < p {
            lo = t;
        } else {
            hi = t;
        }
        t = (lo + hi) * 0.5;
    }

    cubic_bezier(t, 0.0, y1, y2, 1.0)
}

fn cubic_bezier(t: f32, p0: f32, p1: f32, p2: f32, p3: f32) -> f32 {
    let u = 1.0 - t;
    u * u * u * p0 + 3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t * p3
}

fn cubic_bezier_derivative(t: f32, p0: f32, p1: f32, p2: f32, p3: f32) -> f32 {
    let u = 1.0 - t;
    3.0 * u * u * (p1 - p0) + 6.0 * u * t * (p2 - p1) + 3.0 * t * t * (p3 - p2)
}

fn sanitize_finite(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        fallback
    }
}

fn sanitize_non_negative(value: f32) -> f32 {
    sanitize_finite(value, 0.0).max(0.0)
}

fn sanitize_unit(value: f32) -> f32 {
    sanitize_finite(value, 0.0).clamp(0.0, 1.0)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[allow(dead_code)]
fn lerp_color_linear(a: Color, b: Color, t: f32) -> Color {
    let a = a.to_linear();
    let b = b.to_linear();
    Color::linear_rgba(
        lerp(a.red, b.red, t),
        lerp(a.green, b.green, t),
        lerp(a.blue, b.blue, t),
        lerp(a.alpha, b.alpha, t),
    )
}

fn lerp_linear_rgba(a: LinearRgba, b: LinearRgba, t: f32) -> LinearRgba {
    LinearRgba::new(
        lerp(a.red, b.red, t),
        lerp(a.green, b.green, t),
        lerp(a.blue, b.blue, t),
        lerp(a.alpha, b.alpha, t),
    )
}

impl Animatable for f32 {
    fn interpolate(from: &Self, to: &Self, t: f32) -> Self {
        lerp(*from, *to, sanitize_unit(t))
    }
}

impl Animatable for Vec2 {
    fn interpolate(from: &Self, to: &Self, t: f32) -> Self {
        from.lerp(*to, sanitize_unit(t))
    }
}

impl Animatable for Vec3 {
    fn interpolate(from: &Self, to: &Self, t: f32) -> Self {
        from.lerp(*to, sanitize_unit(t))
    }
}

impl Animatable for Vec4 {
    fn interpolate(from: &Self, to: &Self, t: f32) -> Self {
        from.lerp(*to, sanitize_unit(t))
    }
}

impl Animatable for Quat {
    fn interpolate(from: &Self, to: &Self, t: f32) -> Self {
        from.slerp(*to, sanitize_unit(t))
    }
}

impl Animatable for Color {
    fn interpolate(from: &Self, to: &Self, t: f32) -> Self {
        let linear = lerp_linear_rgba(from.to_linear(), to.to_linear(), sanitize_unit(t));
        Color::LinearRgba(linear)
    }
}

impl Animatable for GradientStop {
    fn interpolate(from: &Self, to: &Self, t: f32) -> Self {
        Self {
            position: f32::interpolate(&from.position, &to.position, t),
            color: Color::interpolate(&from.color, &to.color, t),
        }
    }
}

impl Animatable for LinearGradient {
    fn interpolate(from: &Self, to: &Self, t: f32) -> Self {
        Self {
            start: Vec2::interpolate(&from.start, &to.start, t),
            end: Vec2::interpolate(&from.end, &to.end, t),
            stops: interpolate_stops(&from.stops, &to.stops, t),
            dithering: if sanitize_unit(t) >= 1.0 { to.dithering } else { from.dithering },
        }
    }
}

impl Animatable for RadialGradient {
    fn interpolate(from: &Self, to: &Self, t: f32) -> Self {
        Self {
            center: Vec2::interpolate(&from.center, &to.center, t),
            radius: Vec2::interpolate(&from.radius, &to.radius, t),
            stops: interpolate_stops(&from.stops, &to.stops, t),
            dithering: if sanitize_unit(t) >= 1.0 { to.dithering } else { from.dithering },
        }
    }
}

impl Animatable for AngularGradient {
    fn interpolate(from: &Self, to: &Self, t: f32) -> Self {
        Self {
            center: Vec2::interpolate(&from.center, &to.center, t),
            angle_radians: shortest_angle_lerp(from.angle_radians, to.angle_radians, sanitize_unit(t)),
            stops: interpolate_stops(&from.stops, &to.stops, t),
            dithering: if sanitize_unit(t) >= 1.0 { to.dithering } else { from.dithering },
        }
    }
}

impl Animatable for Paint {
    fn interpolate(from: &Self, to: &Self, t: f32) -> Self {
        match (from, to) {
            (Self::Solid(a), Self::Solid(b)) => Self::Solid(Color::interpolate(a, b, t)),
            (Self::LinearGradient(a), Self::LinearGradient(b)) => {
                Self::LinearGradient(LinearGradient::interpolate(a, b, t))
            }
            (Self::RadialGradient(a), Self::RadialGradient(b)) => {
                Self::RadialGradient(RadialGradient::interpolate(a, b, t))
            }
            (Self::AngularGradient(a), Self::AngularGradient(b)) => {
                Self::AngularGradient(AngularGradient::interpolate(a, b, t))
            }
            _ => {
                if sanitize_unit(t) >= 1.0 {
                    to.clone()
                } else {
                    from.clone()
                }
            }
        }
    }
}

impl Animatable for BorderWidths {
    fn interpolate(from: &Self, to: &Self, t: f32) -> Self {
        Self {
            top: f32::interpolate(&from.top, &to.top, t),
            right: f32::interpolate(&from.right, &to.right, t),
            bottom: f32::interpolate(&from.bottom, &to.bottom, t),
            left: f32::interpolate(&from.left, &to.left, t),
        }
    }
}

impl Animatable for Border {
    fn interpolate(from: &Self, to: &Self, t: f32) -> Self {
        Self {
            width: BorderWidths::interpolate(&from.width, &to.width, t),
            paint: Paint::interpolate(&from.paint, &to.paint, t),
        }
    }
}

impl Animatable for CornerRadii {
    fn interpolate(from: &Self, to: &Self, t: f32) -> Self {
        Self {
            top_left: f32::interpolate(&from.top_left, &to.top_left, t),
            top_right: f32::interpolate(&from.top_right, &to.top_right, t),
            bottom_right: f32::interpolate(&from.bottom_right, &to.bottom_right, t),
            bottom_left: f32::interpolate(&from.bottom_left, &to.bottom_left, t),
        }
    }
}

impl Animatable for RoundedRect {
    fn interpolate(from: &Self, to: &Self, t: f32) -> Self {
        Self {
            radii: CornerRadii::interpolate(&from.radii, &to.radii, t),
        }
    }
}

impl Animatable for Shape {
    fn interpolate(from: &Self, to: &Self, t: f32) -> Self {
        match (from, to) {
            (Self::RoundedRect(a), Self::RoundedRect(b)) => Self::RoundedRect(RoundedRect::interpolate(a, b, t)),
        }
    }
}

impl Animatable for Backdrop {
    fn interpolate(from: &Self, to: &Self, t: f32) -> Self {
        Self {
            blur: f32::interpolate(&from.blur, &to.blur, t),
            tint: Color::interpolate(&from.tint, &to.tint, t),
            tint_opacity: f32::interpolate(&from.tint_opacity, &to.tint_opacity, t),
            brightness: f32::interpolate(&from.brightness, &to.brightness, t),
            saturation: f32::interpolate(&from.saturation, &to.saturation, t),
            contrast: f32::interpolate(&from.contrast, &to.contrast, t),
            quality: if sanitize_unit(t) >= 1.0 { to.quality } else { from.quality },
            liquid_glass: match (from.liquid_glass, to.liquid_glass) {
                (Some(a), Some(b)) => Some(crate::rendering::LiquidGlass::interpolate(&a, &b, t)),
                _ => if sanitize_unit(t) >= 1.0 { to.liquid_glass } else { from.liquid_glass },
            },
        }
    }
}

impl Animatable for crate::rendering::LiquidGlass {
    fn interpolate(from: &Self, to: &Self, t: f32) -> Self {
        Self {
            thickness: f32::interpolate(&from.thickness, &to.thickness, t),
            bezel_width: f32::interpolate(&from.bezel_width, &to.bezel_width, t),
            refractive_index: f32::interpolate(&from.refractive_index, &to.refractive_index, t),
            specular_intensity: f32::interpolate(&from.specular_intensity, &to.specular_intensity, t),
            specular_width: f32::interpolate(&from.specular_width, &to.specular_width, t),
            light_direction: Vec2::interpolate(&from.light_direction, &to.light_direction, t),
            fresnel: f32::interpolate(&from.fresnel, &to.fresnel, t),
            chromatic_aberration: f32::interpolate(&from.chromatic_aberration, &to.chromatic_aberration, t),
            press_amount: f32::interpolate(&from.press_amount, &to.press_amount, t),
            ..if sanitize_unit(t) >= 1.0 { *to } else { *from }
        }.sanitized()
    }
}

impl Animatable for OuterShadow {
    fn interpolate(from: &Self, to: &Self, t: f32) -> Self {
        Self {
            color: Color::interpolate(&from.color, &to.color, t),
            offset: Vec2::interpolate(&from.offset, &to.offset, t),
            blur: f32::interpolate(&from.blur, &to.blur, t),
            spread: f32::interpolate(&from.spread, &to.spread, t),
            opacity: f32::interpolate(&from.opacity, &to.opacity, t),
            falloff: if sanitize_unit(t) >= 1.0 { to.falloff } else { from.falloff },
        }
    }
}

impl Animatable for OuterGlow {
    fn interpolate(from: &Self, to: &Self, t: f32) -> Self {
        Self {
            color: Color::interpolate(&from.color, &to.color, t),
            blur: f32::interpolate(&from.blur, &to.blur, t),
            spread: f32::interpolate(&from.spread, &to.spread, t),
            opacity: f32::interpolate(&from.opacity, &to.opacity, t),
            falloff: if sanitize_unit(t) >= 1.0 { to.falloff } else { from.falloff },
        }
    }
}

impl Animatable for InnerShadow {
    fn interpolate(from: &Self, to: &Self, t: f32) -> Self {
        Self {
            color: Color::interpolate(&from.color, &to.color, t),
            offset: Vec2::interpolate(&from.offset, &to.offset, t),
            blur: f32::interpolate(&from.blur, &to.blur, t),
            spread: f32::interpolate(&from.spread, &to.spread, t),
            opacity: f32::interpolate(&from.opacity, &to.opacity, t),
            falloff: if sanitize_unit(t) >= 1.0 { to.falloff } else { from.falloff },
        }
    }
}

impl Animatable for Effects {
    fn interpolate(from: &Self, to: &Self, t: f32) -> Self {
        Self {
            outer_shadow: interpolate_option(&from.outer_shadow, &to.outer_shadow, t),
            outer_glow: interpolate_option(&from.outer_glow, &to.outer_glow, t),
            inner_shadow: interpolate_option(&from.inner_shadow, &to.inner_shadow, t),
        }
    }
}

impl Animatable for FocusRingLayer {
    fn interpolate(from: &Self, to: &Self, t: f32) -> Self {
        Self {
            width: f32::interpolate(&from.width, &to.width, t),
            offset: f32::interpolate(&from.offset, &to.offset, t),
            opacity: f32::interpolate(&from.opacity, &to.opacity, t),
            paint: Paint::interpolate(&from.paint, &to.paint, t),
        }
    }
}

impl Animatable for FocusRing {
    fn interpolate(from: &Self, to: &Self, t: f32) -> Self {
        Self {
            placement: if sanitize_unit(t) >= 1.0 {
                to.placement
            } else {
                from.placement
            },
            primary: FocusRingLayer::interpolate(&from.primary, &to.primary, t),
            secondary: interpolate_option(&from.secondary, &to.secondary, t),
            glow: interpolate_option(&from.glow, &to.glow, t),
        }
    }
}

impl Animatable for Decorations {
    fn interpolate(from: &Self, to: &Self, t: f32) -> Self {
        Self {
            focus_ring: interpolate_option(&from.focus_ring, &to.focus_ring, t),
            selection: if sanitize_unit(t) >= 1.0 {
                to.selection.clone()
            } else {
                from.selection.clone()
            },
            validation: if sanitize_unit(t) >= 1.0 {
                to.validation.clone()
            } else {
                from.validation.clone()
            },
        }
    }
}

impl Animatable for Surface {
    fn interpolate(from: &Self, to: &Self, t: f32) -> Self {
        Self {
            shape: Shape::interpolate(&from.shape, &to.shape, t),
            fill: Paint::interpolate(&from.fill, &to.fill, t),
            border: interpolate_option(&from.border, &to.border, t),
            decorations: Decorations::interpolate(&from.decorations, &to.decorations, t),
            effects: Effects::interpolate(&from.effects, &to.effects, t),
            noise: if sanitize_unit(t) >= 1.0 { to.noise } else { from.noise },
            clip: if sanitize_unit(t) >= 1.0 {
                to.clip.clone()
            } else {
                from.clip.clone()
            },
            mask: if sanitize_unit(t) >= 1.0 {
                to.mask.clone()
            } else {
                from.mask.clone()
            },
            backdrop: interpolate_option(&from.backdrop, &to.backdrop, t),
        }
    }
}

fn interpolate_option<T: Animatable>(from: &Option<T>, to: &Option<T>, t: f32) -> Option<T> {
    match (from, to) {
        (Some(a), Some(b)) => Some(T::interpolate(a, b, t)),
        (None, Some(b)) => {
            if sanitize_unit(t) >= 1.0 {
                Some(b.clone())
            } else {
                None
            }
        }
        (Some(a), None) => {
            if sanitize_unit(t) >= 1.0 {
                None
            } else {
                Some(a.clone())
            }
        }
        (None, None) => None,
    }
}

fn interpolate_stops(from: &[GradientStop], to: &[GradientStop], t: f32) -> Vec<GradientStop> {
    let len = from.len().max(to.len());
    if len == 0 {
        return Vec::new();
    }

    let mut out = Vec::with_capacity(len);
    for i in 0..len {
        let a = from
            .get(i)
            .cloned()
            .or_else(|| from.last().cloned())
            .unwrap_or_else(|| GradientStop::new(0.0, Color::NONE));
        let b = to
            .get(i)
            .cloned()
            .or_else(|| to.last().cloned())
            .unwrap_or_else(|| GradientStop::new(0.0, Color::NONE));
        out.push(GradientStop::interpolate(&a, &b, t));
    }
    out
}

fn shortest_angle_lerp(from: f32, to: f32, t: f32) -> f32 {
    let mut delta = to - from;
    while delta > std::f32::consts::PI {
        delta -= std::f32::consts::TAU;
    }
    while delta < -std::f32::consts::PI {
        delta += std::f32::consts::TAU;
    }
    from + delta * t
}

pub fn should_animate_target<T: PartialEq>(current: &Option<T>, next: &T) -> bool {
    !matches!(current, Some(existing) if existing == next)
}

pub fn themed_transition(theme: &ThemeResource, picker: fn(&crate::theme::ThemeTransitions) -> Transition) -> Transition {
    picker(&theme.current.transitions)
}

/// Headless fixture running Bevy's real camera propagation and UI layout.
#[cfg(any(test, feature = "test-support"))]
pub mod transform_layout_test_support {
    use super::*;
    use bevy::{
        app::{HierarchyPropagatePlugin, PropagateSet},
        camera::{ComputedCameraValues, RenderTargetInfo},
        text::FontCx,
        ui::{ui_surface::UiSurface, ui_layout_system, update::propagate_ui_target_cameras},
    };

    pub fn app(scale_factor: f32) -> App {
        let mut app = App::new();
        app.add_plugins((
            TaskPoolPlugin::default(),
            HierarchyPropagatePlugin::<ComputedUiTargetCamera>::new(PostUpdate),
            HierarchyPropagatePlugin::<ComputedUiRenderTargetInfo>::new(PostUpdate),
        ))
        .init_resource::<Time>()
        .init_resource::<UiScale>()
        .init_resource::<UiSurface>()
        .init_resource::<FontCx>()
        .add_systems(PostUpdate, (propagate_ui_target_cameras, ui_layout_system).chain())
        .configure_sets(
            PostUpdate,
            PropagateSet::<ComputedUiTargetCamera>::default()
                .after(propagate_ui_target_cameras)
                .before(ui_layout_system),
        )
        .configure_sets(
            PostUpdate,
            PropagateSet::<ComputedUiRenderTargetInfo>::default()
                .after(propagate_ui_target_cameras)
                .before(ui_layout_system),
        );
        app.world_mut().spawn((
            Camera2d,
            Camera {
                computed: ComputedCameraValues {
                    target_info: Some(RenderTargetInfo {
                        physical_size: UVec2::new(800, 600),
                        scale_factor,
                    }),
                    ..default()
                },
                ..default()
            },
        ));
        app
    }

    pub fn assert_global(app: &App, entity: Entity, expected: bevy::math::Affine2) {
        let actual = app.world().get::<UiGlobalTransform>(entity).unwrap();
        // Check the center AND basis, so a translation-only fix cannot pass.
        for point in [Vec2::ZERO, Vec2::X, Vec2::Y] {
            let actual = actual.transform_point2(point);
            let expected = expected.transform_point2(point);
            assert!((actual - expected).length() < 0.002, "{actual:?} != {expected:?}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::transform_layout_test_support::{app as layout_app, assert_global};
    use bevy::math::Affine2;

    fn transform_app(scale_factor: f32) -> App {
        let mut app = layout_app(scale_factor);
        app.init_resource::<AccessibilityVisualPolicyResource>()
            .add_plugins(UiAnimationPlugin);
        app
    }

    fn advance(app: &mut App, delta: f32) {
        app.world_mut().resource_mut::<UiAnimationClock>().set_fixed_step(delta);
        app.update();
    }

    #[test]
    fn transform_ui_layout_is_frame_rate_independent_and_handles_dropped_frames() {
        let mut schedules: Vec<Vec<f32>> = [30, 60, 120]
            .into_iter()
            .map(|fps| vec![1.0 / fps as f32; fps / 2])
            .collect();
        schedules.push(vec![0.1, 0.25, 0.15]);

        for scale_factor in [1.0, 2.0] {
            for deltas in &schedules {
                let mut app = transform_app(scale_factor);
                let target = Transform {
                    translation: Vec3::new(40.0, -20.0, 0.0),
                    rotation: Quat::from_rotation_z(0.6),
                    scale: Vec3::new(1.4, 0.8, 1.0),
                };
                let entity = app.world_mut().spawn((
                    Node { width: px(100.0), height: px(80.0), ..default() },
                    TransformTransitionTarget::new(target, transition(1000, Easing::Linear)),
                    UiAnimationDirtyFlags::default(),
                )).id();
                // Node requires UiTransform, not Transform: no legacy component needed.
                assert!(app.world().get::<Transform>(entity).is_none());
                for delta in deltas {
                    advance(&mut app, *delta);
                }
                assert_eq!(app.world().get::<ComputedNode>(entity).unwrap().size(), Vec2::new(100.0, 80.0) * scale_factor);
                assert_global(&app, entity, Affine2::from_scale_angle_translation(
                    Vec2::new(1.2, 0.9), 0.3,
                    Vec2::new(70.0, 30.0) * scale_factor,
                ));
                assert!(app.world().get::<UiAnimationDirtyFlags>(entity).unwrap().contains(UiAnimationDirtyKind::TransformDirty));

                // A dropped frame past the end must apply the exact final pose
                // before removing the track; layout must see it in this frame.
                advance(&mut app, 2.0);
                let final_pose = Affine2::from_scale_angle_translation(
                    target.scale.truncate(), 0.6,
                    Vec2::new(90.0, 20.0) * scale_factor,
                );
                assert_global(&app, entity, final_pose);
                assert!(app.world().get::<TransformAnimationTrack>(entity).is_none());
                advance(&mut app, 0.1);
                assert_global(&app, entity, final_pose);
            }
        }
    }

    #[test]
    fn transform_ui_starts_from_visible_pose_and_retargets_without_jump() {
        let mut app = transform_app(2.0);
        let legacy = Transform::from_xyz(900.0, 800.0, 7.0);
        let entity = app.world_mut().spawn((
            Node { width: px(100.0), height: px(80.0), ..default() },
            UiTransform::from_translation(Val2::percent(20.0, 25.0)),
            legacy,
        )).id();
        advance(&mut app, 0.0); // Resolve responsive starting offsets via layout.
        app.world_mut().entity_mut(entity).insert(TransformTransitionTarget::new(
            Transform::from_xyz(60.0, 40.0, 0.0),
            transition(1000, Easing::Linear).with_delay(Duration::from_millis(100)),
        ));
        advance(&mut app, 0.05);
        assert_global(&app, entity, Affine2::from_translation(Vec2::new(140.0, 120.0)));
        advance(&mut app, 0.55);
        assert_global(&app, entity, Affine2::from_translation(Vec2::new(180.0, 140.0)));
        app.world_mut().entity_mut(entity).insert(TransformTransitionTarget::new(
            Transform::IDENTITY, transition(1000, Easing::Linear),
        ));
        advance(&mut app, 0.0);
        assert_global(&app, entity, Affine2::from_translation(Vec2::new(180.0, 140.0)));
        advance(&mut app, 0.5);
        assert_global(&app, entity, Affine2::from_translation(Vec2::new(140.0, 110.0)));
        assert_eq!(*app.world().get::<Transform>(entity).unwrap(), legacy);

        app.world_mut().resource_mut::<AccessibilityVisualPolicyResource>().current.reduced_motion = true;
        advance(&mut app, 0.01);
        assert_global(&app, entity, Affine2::from_translation(Vec2::new(100.0, 80.0)));
    }

    #[test]
    fn transform_non_ui_retains_full_3d_animation() {
        let mut app = transform_app(1.0);
        let target = Transform {
            translation: Vec3::new(10.0, 20.0, 30.0),
            rotation: Quat::from_rotation_x(0.8),
            scale: Vec3::new(2.0, 3.0, 4.0),
        };
        let entity = app.world_mut().spawn((
            Transform::IDENTITY,
            TransformTransitionTarget::new(target, transition(1000, Easing::Linear)),
        )).id();
        advance(&mut app, 0.5);
        let actual = app.world().get::<Transform>(entity).unwrap();
        assert_eq!(actual.translation, target.translation * 0.5);
        assert_eq!(actual.scale, Vec3::new(1.5, 2.0, 2.5));
        assert!(actual.rotation.angle_between(Quat::from_rotation_x(0.4)) < 0.001);
        advance(&mut app, 0.6);
        assert_eq!(*app.world().get::<Transform>(entity).unwrap(), target);
        assert!(app.world().get::<UiTransform>(entity).is_none());
    }

    fn transition(duration_ms: u64, easing: Easing) -> Transition {
        Transition::new(Duration::from_millis(duration_ms), easing)
    }

    #[test]
    fn f32_interpolation_is_linear() {
        assert!((f32::interpolate(&0.0, &10.0, 0.5) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn vec_interpolation_is_supported() {
        let v = Vec3::interpolate(&Vec3::ZERO, &Vec3::new(2.0, 4.0, 6.0), 0.5);
        assert_eq!(v, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn color_interpolation_uses_linear_space() {
        let a = Color::srgb(0.0, 0.0, 0.0);
        let b = Color::srgb(1.0, 1.0, 1.0);
        let mid = Color::interpolate(&a, &b, 0.5).to_linear();

        assert!((mid.red - 0.5).abs() < 1e-6);
        assert!((mid.green - 0.5).abs() < 1e-6);
        assert!((mid.blue - 0.5).abs() < 1e-6);
    }

    #[test]
    fn zero_duration_resolves_immediately() {
        let mut track = AnimationTrack::from_value(0.0_f32);
        track.set_target(1.0, Transition::new(Duration::ZERO, Easing::Linear));

        assert!(track.tick(0.001));
        assert_eq!(*track.current(), 1.0);
        assert_eq!(track.state(), AnimationLifecycle::Completed);
    }

    #[test]
    fn easing_outputs_are_stable_and_clamped() {
        let easing = Easing::CubicBezier {
            x1: -2.0,
            y1: 3.0,
            x2: 4.0,
            y2: -1.0,
        };

        for p in [f32::NEG_INFINITY, f32::NAN, -5.0, 0.0, 0.3, 0.7, 1.0, 5.0, f32::INFINITY] {
            let out = easing.sample(p);
            assert!(out.is_finite());
            assert!((0.0..=1.0).contains(&out));
        }
    }

    #[test]
    fn interruption_restarts_from_current_value() {
        let mut track = AnimationTrack::from_value(0.0_f32);
        track.set_target(1.0, transition(100, Easing::Linear));
        let _ = track.tick(0.05);
        let mid = *track.current();

        track.set_target(0.0, transition(100, Easing::Linear));
        let _ = track.tick(0.05);

        assert!(mid > 0.0 && mid < 1.0);
        assert!(*track.current() < mid);
    }

    #[test]
    fn reversal_is_smooth() {
        let mut track = AnimationTrack::from_value(0.0_f32);
        track.set_target(1.0, transition(200, Easing::EaseOut));
        let _ = track.tick(0.08);
        let forward = *track.current();

        track.set_target(0.0, transition(200, Easing::EaseOut));
        let _ = track.tick(0.02);

        assert!(*track.current() <= forward);
    }

    #[test]
    fn delay_is_respected() {
        let mut track = AnimationTrack::from_value(0.0_f32);
        track.set_target(
            1.0,
            transition(100, Easing::Linear).with_delay(Duration::from_millis(50)),
        );

        assert!(!track.tick(0.03));
        assert_eq!(*track.current(), 0.0);
        assert_eq!(track.state(), AnimationLifecycle::Pending);

        let _ = track.tick(0.03);
        assert!(*track.current() > 0.0);
    }

    #[test]
    fn cancelled_track_stops_progression() {
        let mut track = AnimationTrack::from_value(0.0_f32);
        track.set_target(1.0, transition(100, Easing::Linear));
        track.cancel();
        let changed = track.tick(1.0);

        assert!(!changed);
        assert_eq!(track.state(), AnimationLifecycle::Cancelled);
        assert_eq!(*track.current(), 0.0);
    }

    #[test]
    fn gradient_stops_are_interpolated() {
        let a = LinearGradient::horizontal(vec![
            GradientStop::new(0.0, Color::BLACK),
            GradientStop::new(1.0, Color::WHITE),
        ]);
        let b = LinearGradient::horizontal(vec![
            GradientStop::new(0.0, Color::WHITE),
            GradientStop::new(1.0, Color::BLACK),
        ]);

        let m = LinearGradient::interpolate(&a, &b, 0.5);
        assert_eq!(m.stops.len(), 2);
        let mid = m.stops[0].color.to_linear();
        assert!((mid.red - 0.5).abs() < 1e-6);
    }

    #[test]
    fn surface_transition_tracks_shadow_backdrop_and_focus() {
        let mut from = Surface::rounded_rect_fill(8.0, Paint::solid(Color::BLACK));
        from.effects.outer_shadow = Some(OuterShadow::new(Color::BLACK).with_opacity(0.1));
        from.backdrop = Some(Backdrop::new().with_blur(0.0));

        let mut to = Surface::rounded_rect_fill(18.0, Paint::solid(Color::WHITE));
        to.effects.outer_shadow = Some(OuterShadow::new(Color::WHITE).with_opacity(0.8));
        to.backdrop = Some(Backdrop::new().with_blur(20.0));

        let mid = Surface::interpolate(&from, &to, 0.5);
        if let Some(backdrop) = mid.backdrop {
            assert!(backdrop.blur > 0.0 && backdrop.blur < 20.0);
        } else {
            panic!("expected animated backdrop");
        }
    }

    #[test]
    fn node_left_track_is_frame_rate_independent() {
        let mut track_a = AnimationTrack::from_value(100.0_f32);
        track_a.set_target(0.0, transition(120, Easing::EaseOut));
        for _ in 0..4 {
            let _ = track_a.tick(0.03);
        }

        let mut track_b = AnimationTrack::from_value(100.0_f32);
        track_b.set_target(0.0, transition(120, Easing::EaseOut));
        for _ in 0..12 {
            let _ = track_b.tick(0.01);
        }

        assert!((track_a.current() - track_b.current()).abs() < 1e-3);
    }

    #[test]
    fn reduced_motion_collapses_decorative_duration() {
        let mut policy = AccessibilityVisualPolicyResource::default();
        policy.current.reduced_motion = true;

        let decorative = transition(200, Easing::EaseOut).with_motion_class(MotionClass::Decorative);
        let semantic = transition(200, Easing::EaseOut).with_motion_class(MotionClass::Semantic);

        assert_eq!(decorative.effective_for_policy(&policy).duration, Duration::ZERO);
        assert_eq!(semantic.effective_for_policy(&policy).duration, Duration::from_millis(200));
    }

    #[test]
    fn reduced_transparency_disables_backdrop() {
        let mut surface = Surface::rounded_rect_fill(10.0, Paint::solid(Color::srgba(1.0, 1.0, 1.0, 0.2)));
        surface.backdrop = Some(Backdrop::new().with_blur(12.0));

        let next = apply_reduced_transparency_surface(surface);
        assert!(next.backdrop.is_none());
        if let Paint::Solid(color) = next.fill {
            assert!(color.to_linear().alpha >= 0.92);
        } else {
            panic!("expected solid fill");
        }
    }

    #[test]
    fn large_delta_completes_without_nan() {
        let mut track = AnimationTrack::from_value(Vec2::ZERO);
        track.set_target(Vec2::new(5.0, 5.0), transition(150, Easing::Smooth));
        let _ = track.tick(10_000.0);

        assert_eq!(*track.current(), Vec2::new(5.0, 5.0));
        assert_eq!(track.state(), AnimationLifecycle::Completed);
    }

    #[test]
    fn animation_groups_share_transition_spec() {
        let t = transition(180, Easing::EaseInOut)
            .with_delay(Duration::from_millis(20))
            .with_motion_class(MotionClass::Decorative);

        let surface_target = SurfaceTransitionTarget::new(
            Surface::rounded_rect_fill(8.0, Paint::solid(Color::WHITE)),
            t,
        )
        .in_group(42);
        let text_target = TextColorTransitionTarget::new(Color::WHITE, t).in_group(42);

        assert_eq!(surface_target.group, text_target.group);
        assert_eq!(surface_target.transition, text_target.transition);
    }
}
