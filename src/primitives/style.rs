use bevy::prelude::*;

use crate::rendering::{Paint, Surface};

pub trait IntoStyleValue {
    fn into_style_value(self) -> Val;
}

pub trait IntoStyleNumber {
    fn into_style_number(self) -> f32;
}

impl IntoStyleValue for Val {
    fn into_style_value(self) -> Val {
        self
    }
}

macro_rules! numeric_style_values {
    ($($kind:ty),* $(,)?) => {$(
        impl IntoStyleValue for $kind {
            fn into_style_value(self) -> Val { px(self as f32) }
        }
        impl IntoStyleNumber for $kind {
            fn into_style_number(self) -> f32 { self as f32 }
        }
    )*};
}

numeric_style_values!(f32, f64, i32, u32, usize);

pub trait IntoStyleColor {
    fn into_style_color(self) -> Color;
}

impl IntoStyleColor for Color {
    fn into_style_color(self) -> Color {
        self
    }
}

impl IntoStyleColor for &str {
    fn into_style_color(self) -> Color {
        let color = self
            .parse::<csscolorparser::Color>()
            .unwrap_or_else(|error| panic!("invalid styling color {self:?}: {error}"));
        Color::srgba(
            color.r as f32,
            color.g as f32,
            color.b as f32,
            color.a as f32,
        )
    }
}

impl IntoStyleColor for String {
    fn into_style_color(self) -> Color {
        self.as_str().into_style_color()
    }
}

pub trait IntoStylePaint {
    fn into_style_paint(self) -> Paint;
}

impl IntoStylePaint for Paint {
    fn into_style_paint(self) -> Paint {
        self
    }
}

impl IntoStylePaint for Color {
    fn into_style_paint(self) -> Paint {
        Paint::solid(self)
    }
}

impl IntoStylePaint for &str {
    fn into_style_paint(self) -> Paint {
        Paint::solid(self.into_style_color())
    }
}

impl IntoStylePaint for String {
    fn into_style_paint(self) -> Paint {
        Paint::solid(self.into_style_color())
    }
}

#[derive(Component, Clone, Default)]
pub struct UiStyle {
    pub padding: Option<UiRect>,
    pub margin: Option<UiRect>,
    pub width: Option<Val>,
    pub height: Option<Val>,
    pub min_width: Option<Val>,
    pub max_width: Option<Val>,
    pub min_height: Option<Val>,
    pub max_height: Option<Val>,
    pub row_gap: Option<Val>,
    pub column_gap: Option<Val>,
    pub radius: Option<f32>,
    pub background: Option<Paint>,
    pub border_width: Option<f32>,
    pub border_color: Option<Color>,
    pub color: Option<Color>,
    pub font_size: Option<f32>,
    pub align_items: Option<AlignItems>,
    pub justify_content: Option<JustifyContent>,
    pub align_self: Option<AlignSelf>,
    pub flex_grow: Option<f32>,
    pub flex_shrink: Option<f32>,
    pub scale: Option<Vec2>,
    pub outline_width: Option<f32>,
    pub outline_color: Option<Color>,
    pub opacity: Option<f32>,
    pub state_styles: Vec<(StyleState, Box<UiStyle>)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StyleState {
    Hover,
    Focused,
    Pressed,
    Active,
    Inactive,
    Disabled,
    Submitting,
    Success,
}

impl UiStyle {
    pub fn padding(mut self, value: impl IntoStyleValue) -> Self {
        self.padding = Some(UiRect::all(value.into_style_value()));
        self
    }
    pub fn padding_x(mut self, value: impl IntoStyleValue) -> Self {
        let value = value.into_style_value();
        self.padding = Some(UiRect::axes(
            value,
            self.padding.unwrap_or(UiRect::ZERO).top,
        ));
        self
    }
    pub fn padding_y(mut self, value: impl IntoStyleValue) -> Self {
        let value = value.into_style_value();
        self.padding = Some(UiRect::axes(
            self.padding.unwrap_or(UiRect::ZERO).left,
            value,
        ));
        self
    }
    pub fn margin(mut self, value: impl IntoStyleValue) -> Self {
        self.margin = Some(UiRect::all(value.into_style_value()));
        self
    }
    pub fn width(mut self, value: impl IntoStyleValue) -> Self {
        self.width = Some(value.into_style_value());
        self
    }
    pub fn height(mut self, value: impl IntoStyleValue) -> Self {
        self.height = Some(value.into_style_value());
        self
    }
    pub fn radius(mut self, value: impl IntoStyleNumber) -> Self {
        self.radius = Some(value.into_style_number().max(0.0));
        self
    }
    pub fn gap(mut self, value: impl IntoStyleValue) -> Self {
        let value = value.into_style_value();
        self.row_gap = Some(value);
        self.column_gap = Some(value);
        self
    }
    pub fn row_gap(mut self, value: impl IntoStyleValue) -> Self {
        self.row_gap = Some(value.into_style_value());
        self
    }
    pub fn column_gap(mut self, value: impl IntoStyleValue) -> Self {
        self.column_gap = Some(value.into_style_value());
        self
    }
    pub fn background(mut self, value: impl IntoStylePaint) -> Self {
        self.background = Some(value.into_style_paint());
        self
    }
    pub fn border_width(mut self, value: impl IntoStyleNumber) -> Self {
        self.border_width = Some(value.into_style_number().max(0.0));
        self
    }
    pub fn border_color(mut self, value: impl IntoStyleColor) -> Self {
        self.border_color = Some(value.into_style_color());
        self
    }
    pub fn color(mut self, value: impl IntoStyleColor) -> Self {
        self.color = Some(value.into_style_color());
        self
    }
    pub fn font_size(mut self, value: impl IntoStyleNumber) -> Self {
        self.font_size = Some(value.into_style_number().max(0.0));
        self
    }
    pub fn opacity(mut self, value: impl IntoStyleNumber) -> Self {
        self.opacity = Some(value.into_style_number().clamp(0.0, 1.0));
        self
    }
    pub fn scale(mut self, value: impl IntoStyleNumber) -> Self {
        self.scale = Some(Vec2::splat(value.into_style_number()));
        self
    }
    pub fn outline_width(mut self, value: impl IntoStyleNumber) -> Self {
        self.outline_width = Some(value.into_style_number().max(0.0));
        self
    }
    pub fn outline_color(mut self, value: impl IntoStyleColor) -> Self {
        self.outline_color = Some(value.into_style_color());
        self
    }
}

fn merge_style(base: &mut UiStyle, patch: &UiStyle) {
    macro_rules! merge { ($($field:ident),* $(,)?) => { $(if patch.$field.is_some() { base.$field = patch.$field.clone(); })* }; }
    merge!(
        padding,
        margin,
        width,
        height,
        min_width,
        max_width,
        min_height,
        max_height,
        row_gap,
        column_gap,
        radius,
        background,
        border_width,
        border_color,
        color,
        font_size,
        align_items,
        justify_content,
        align_self,
        flex_grow,
        flex_shrink,
        scale,
        outline_width,
        outline_color,
        opacity
    );
}

fn active_style_states(world: &World, entity: Entity) -> Vec<StyleState> {
    let mut hovered = false;
    let mut focused = false;
    let mut pressed = false;
    let mut active = false;
    let mut inactive = false;
    let mut disabled = false;
    let focus = world
        .get_resource::<bevy::input_focus::InputFocus>()
        .and_then(|focus| focus.get());
    let mut pending = vec![entity];
    while let Some(target) = pending.pop() {
        let interaction = world
            .get::<Interaction>(target)
            .copied()
            .unwrap_or(Interaction::None);
        hovered |= interaction == Interaction::Hovered;
        pressed |= interaction == Interaction::Pressed;
        let semantic = world.get::<crate::primitives::semantic::SemanticNode>(target);
        focused |= focus == Some(target) || semantic.is_some_and(|node| node.state.focused);
        let checked = semantic
            .and_then(|node| node.state.checked)
            .or_else(|| {
                world
                    .get::<crate::components::checkbox::CheckboxState>(target)
                    .map(|state| state.checked)
            })
            .or_else(|| {
                world
                    .get::<crate::components::toggle::Toggle>(target)
                    .map(|state| state.checked)
            });
        active |= checked == Some(true)
            || semantic.is_some_and(|node| node.state.selected || node.state.pressed == Some(true));
        inactive |= checked == Some(false)
            || semantic.is_some_and(|node| node.state.pressed == Some(false));
        disabled |= world
            .get::<crate::primitives::interaction::DisabledInteraction>(target)
            .is_some()
            || semantic.is_some_and(|node| node.state.disabled)
            || world
                .get::<crate::components::checkbox::CheckboxState>(target)
                .is_some_and(|state| state.disabled)
            || world
                .get::<crate::components::toggle::Toggle>(target)
                .is_some_and(|state| state.disabled)
            || world
                .get::<crate::components::slider::Slider>(target)
                .is_some_and(|state| state.disabled)
            || world
                .get::<crate::components::input::TextInput>(target)
                .is_some_and(|state| state.disabled)
            || world
                .get::<crate::components::textarea::Textarea>(target)
                .is_some_and(|state| state.disabled)
            || world
                .get::<crate::components::select::Select<String>>(target)
                .is_some_and(|state| state.disabled)
            || world
                .get::<crate::components::dropdown::Dropdown>(target)
                .is_some_and(|state| state.disabled)
            || world
                .get::<crate::components::link::Link>(target)
                .is_some_and(|state| state.disabled)
            || world
                .get::<crate::components::file_input::FileInput>(target)
                .is_some_and(|state| state.disabled);
        if let Some(children) = world.get::<Children>(target) {
            pending.extend(children.iter());
        }
    }
    let status = world
        .get::<crate::primitives::binding::FormStatusState>(entity)
        .map(|state| state.status);
    let mut states = Vec::new();
    if inactive {
        states.push(StyleState::Inactive);
    }
    if active {
        states.push(StyleState::Active);
    }
    if hovered {
        states.push(StyleState::Hover);
    }
    if focused {
        states.push(StyleState::Focused);
    }
    if pressed {
        states.push(StyleState::Pressed);
    }
    if status == Some(crate::primitives::binding::FormStatus::Submitting) {
        states.push(StyleState::Submitting);
    }
    if status == Some(crate::primitives::binding::FormStatus::Success) {
        states.push(StyleState::Success);
    }
    if disabled {
        states.push(StyleState::Disabled);
    }
    states
}

fn resolve_style(world: &World, entity: Entity, style: &UiStyle) -> UiStyle {
    let active = active_style_states(world, entity);
    let mut resolved = style.clone();
    resolved.state_styles.clear();
    for state in active {
        if let Some((_, patch)) = style
            .state_styles
            .iter()
            .find(|(candidate, _)| *candidate == state)
        {
            merge_style(&mut resolved, patch);
        }
    }
    resolved
}

#[derive(Component, Default)]
struct OpacitySource {
    fill: Option<(Paint, Paint)>,
    border: Option<(Paint, Paint)>,
    text: Option<(Color, Color)>,
    background: Option<(Color, Color)>,
    icon: Option<(Color, Color)>,
}

fn color_alpha(color: Color, opacity: f32) -> Color {
    color.with_alpha(color.alpha() * opacity)
}

fn fade_color(current: Color, source: &mut Option<(Color, Color)>, opacity: f32) -> Color {
    let base = source
        .as_ref()
        .filter(|(_, previous)| *previous == current)
        .map_or(current, |(base, _)| *base);
    let faded = color_alpha(base, opacity);
    *source = Some((base, faded));
    faded
}

fn fade_paint(current: &Paint, source: &mut Option<(Paint, Paint)>, opacity: f32) -> Paint {
    let base = source
        .as_ref()
        .filter(|(_, previous)| previous == current)
        .map_or_else(|| current.clone(), |(base, _)| base.clone());
    let mut faded = base.clone();
    let stops = match &mut faded {
        Paint::Solid(color) => {
            *color = color_alpha(*color, opacity);
            None
        }
        Paint::Shimmer(shimmer) => {
            shimmer.base_color = color_alpha(shimmer.base_color, opacity);
            shimmer.highlight_color = color_alpha(shimmer.highlight_color, opacity);
            None
        }
        Paint::LinearGradient(gradient) => Some(&mut gradient.stops),
        Paint::RadialGradient(gradient) => Some(&mut gradient.stops),
        Paint::AngularGradient(gradient) => Some(&mut gradient.stops),
        Paint::Spinning(gradient) => Some(&mut gradient.stops),
    };
    if let Some(stops) = stops {
        for stop in stops {
            stop.color = color_alpha(stop.color, opacity);
        }
    }
    *source = Some((base, faded.clone()));
    faded
}

fn apply_opacity(world: &mut World) {
    let entities: Vec<_> = world
        .query_filtered::<Entity, Or<(
            With<Surface>,
            With<TextColor>,
            With<BackgroundColor>,
            With<crate::icons::IconNode>,
            With<OpacitySource>,
        )>>()
        .iter(world)
        .collect();
    for entity in entities {
        let mut opacity = 1.0;
        let mut current = Some(entity);
        while let Some(target) = current {
            if let Some(style) = world.get::<UiStyle>(target) {
                let style = resolve_style(world, target, style);
                if let Some(value) = style.opacity {
                    opacity *= value;
                }
            }
            current = world.get::<ChildOf>(target).map(ChildOf::parent);
        }
        if opacity == 1.0 && world.get::<OpacitySource>(entity).is_none() {
            continue;
        }
        let mut source = world
            .entity_mut(entity)
            .take::<OpacitySource>()
            .unwrap_or_default();
        if let Some(mut surface) = world.get_mut::<Surface>(entity) {
            let fill = fade_paint(&surface.fill, &mut source.fill, opacity);
            let border = surface
                .border
                .as_ref()
                .map(|border| fade_paint(&border.paint, &mut source.border, opacity));
            if surface.fill != fill {
                surface.fill = fill;
            }
            if let Some(paint) = border {
                if surface
                    .border
                    .as_ref()
                    .is_some_and(|border| border.paint != paint)
                {
                    surface.border.as_mut().unwrap().paint = paint;
                }
            }
        }
        if let Some(mut text) = world.get_mut::<TextColor>(entity) {
            let faded = fade_color(text.0, &mut source.text, opacity);
            if text.0 != faded {
                text.0 = faded;
            }
        }
        if let Some(mut background) = world.get_mut::<BackgroundColor>(entity) {
            let faded = fade_color(background.0, &mut source.background, opacity);
            if background.0 != faded {
                background.0 = faded;
            }
        }
        if let Some(mut icon) = world.get_mut::<crate::icons::IconNode>(entity) {
            let faded = fade_color(icon.color, &mut source.icon, opacity);
            if icon.color != faded {
                icon.color = faded;
            }
        }
        world.entity_mut(entity).insert(source);
    }
}

/// Applies explicit fluent styles after component updates and before UI layout.
/// Included by `BeverlyPlugin`; install separately in custom Bevy applications.
///
/// ```
/// use bevy::prelude::*;
/// use beverly::prelude::*;
/// let mut app = App::new();
/// app.add_plugins(StylingPlugin);
/// let parent = app.world_mut().spawn(Node::default()).id();
/// let form = UiElement::form(Form::new().show_submit_button(false))
///     .padding(24).radius(8).gap(16).width(400)
///     .background("#ffffff").border_width(1).border_color("#d4d4d4")
///     .color("#171717").font_size(16)
///     .body(text("Account settings"))
///     .spawn_in_context(&mut UiBuildContext { world: app.world_mut(), parent });
/// app.update();
/// assert_eq!(app.world().get::<Node>(form).unwrap().width, px(400));
/// assert_eq!(app.world().get::<Node>(form).unwrap().padding, UiRect::all(px(24)));
/// ```
pub struct StylingPlugin;

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StylingSystems;

impl Plugin for StylingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, apply_styles.in_set(StylingSystems))
            .configure_sets(
                PostUpdate,
                StylingSystems
                    .after(crate::primitives::binding::BindingSystems)
                    .before(bevy::ui::UiSystems::Layout),
            );
    }
}

pub(crate) fn apply_style(world: &mut World, entity: Entity, style: &UiStyle) {
    let Some((layout, visual)) = style_targets(world, entity) else {
        return;
    };
    if world.get::<Node>(layout).is_none() {
        world.entity_mut(layout).insert(Node::default());
    }
    let mut node = world.get::<Node>(layout).unwrap().clone();
    if let Some(value) = style.margin {
        node.margin = value;
    }
    if let Some(value) = style.width {
        node.width = value;
    }
    if let Some(value) = style.height {
        node.height = value;
    }
    if let Some(value) = style.min_width {
        node.min_width = value;
    }
    if let Some(value) = style.max_width {
        node.max_width = value;
    }
    if let Some(value) = style.min_height {
        node.min_height = value;
    }
    if let Some(value) = style.max_height {
        node.max_height = value;
    }
    if let Some(value) = style.row_gap {
        node.row_gap = value;
    }
    if let Some(value) = style.column_gap {
        node.column_gap = value;
    }
    if let Some(value) = style.align_items {
        node.align_items = value;
    }
    if let Some(value) = style.justify_content {
        node.justify_content = value;
    }
    if let Some(value) = style.align_self {
        node.align_self = value;
    }
    if let Some(value) = style.flex_grow {
        node.flex_grow = value;
    }
    if let Some(value) = style.flex_shrink {
        node.flex_shrink = value;
    }
    world.get_mut::<Node>(layout).unwrap().set_if_neq(node);
    let mut node = world.get::<Node>(visual).unwrap().clone();
    if let Some(value) = style.padding {
        node.padding = value;
    }
    if let Some(value) = style.radius {
        node.border_radius = BorderRadius::all(px(value));
    }
    if let Some(value) = style.border_width {
        node.border = UiRect::all(px(value));
    }
    world.get_mut::<Node>(visual).unwrap().set_if_neq(node);
    if let Some(scale) = style.scale {
        if world.get::<UiTransform>(visual).is_none() {
            world
                .entity_mut(visual)
                .insert(UiTransform::from_scale(scale));
        } else if let Some(mut transform) = world.get_mut::<UiTransform>(visual) {
            if transform.scale != scale {
                transform.scale = scale;
            }
        }
    }
    if style.background.is_some()
        || style.radius.is_some()
        || style.border_width.is_some()
        || style.border_color.is_some()
    {
        let mut surface = world.get::<Surface>(visual).cloned().unwrap_or_else(|| {
            let color = world
                .get::<BackgroundColor>(visual)
                .map_or(Color::NONE, |color| color.0);
            Surface::rounded_rect_fill(style.radius.unwrap_or(0.0), color)
        });
        if let Some(value) = style.radius {
            surface.shape = crate::rendering::Shape::rounded_rect(value);
        }
        if let Some(value) = &style.background {
            surface.fill = value.clone();
        }
        if style.border_width.is_some() || style.border_color.is_some() {
            let mut border = surface
                .border
                .unwrap_or_else(|| crate::rendering::Border::new(0.0, Paint::solid(Color::NONE)));
            if let Some(value) = style.border_width {
                border.width = crate::rendering::BorderWidths::all(value);
            }
            if let Some(value) = style.border_color {
                border.paint = Paint::solid(value);
            }
            surface.border = Some(border);
        }
        if world.get::<Surface>(visual) != Some(&surface) {
            world.entity_mut(visual).insert(surface);
        }
        if world
            .get::<BackgroundColor>(visual)
            .is_none_or(|color| color.0 != Color::NONE)
        {
            world
                .entity_mut(visual)
                .insert(BackgroundColor(Color::NONE));
        }
        if world
            .get::<BorderColor>(visual)
            .is_none_or(|color| *color != BorderColor::all(Color::NONE))
        {
            world
                .entity_mut(visual)
                .insert(BorderColor::all(Color::NONE));
        }
    }
    if style.outline_width.is_some() || style.outline_color.is_some() {
        let mut outline = world
            .get::<Outline>(visual)
            .cloned()
            .unwrap_or(Outline::new(px(0), px(2), Color::WHITE));
        if let Some(width) = style.outline_width {
            outline.width = px(width);
        }
        if let Some(color) = style.outline_color {
            outline.color = color;
        }
        world.entity_mut(visual).insert(outline);
    }
    if style.color.is_some() || style.font_size.is_some() {
        let mut pending = vec![layout];
        while let Some(target) = pending.pop() {
            if world.get::<Text>(target).is_some() {
                if let Some(color) = style.color {
                    if world
                        .get::<TextColor>(target)
                        .is_none_or(|current| current.0 != color)
                    {
                        world.entity_mut(target).insert(TextColor(color));
                    }
                }
                if let Some(size) = style.font_size {
                    if let Some(mut font) = world.get_mut::<TextFont>(target) {
                        if font.font_size != FontSize::Px(size) {
                            font.font_size = FontSize::Px(size);
                        }
                    } else {
                        world.entity_mut(target).insert(TextFont {
                            font_size: FontSize::Px(size),
                            ..default()
                        });
                    }
                }
            }
            if let Some(color) = style.color {
                if let Some(mut icon) = world.get_mut::<crate::icons::IconNode>(target) {
                    if icon.color != color {
                        icon.color = color;
                    }
                }
            }
            if let Some(children) = world.get::<Children>(target) {
                pending.extend(children.iter());
            }
        }
    }
    if let Some(presentation) = world.get::<crate::primitives::binding::ErrorPresentation>(entity) {
        let host = presentation.host;
        let width = world.get::<Node>(layout).unwrap().width;
        if let Some(mut node) = world.get_mut::<Node>(host) {
            if node.width != width {
                node.width = width;
            }
        }
    }
}

fn style_targets(world: &World, entity: Entity) -> Option<(Entity, Entity)> {
    let modal = world
        .get::<crate::components::modal::Modal>(entity)
        .is_some();
    let toast = world
        .get::<crate::components::toast::Toast>(entity)
        .is_some();
    let composite = world
        .get::<crate::components::dropdown::Dropdown>(entity)
        .is_some()
        || world
            .get::<crate::components::select::component::SelectRoot>(entity)
            .is_some();
    if modal || toast || composite {
        let mut pending = std::collections::VecDeque::from([entity]);
        while let Some(target) = pending.pop_front() {
            let matches = if modal {
                world
                    .get::<crate::components::modal::component::ModalSurface>(target)
                    .is_some()
            } else if toast {
                world
                    .get::<crate::components::toast::component::ToastCard>(target)
                    .is_some()
            } else {
                world
                    .get::<crate::components::dropdown::component::DropdownSurface>(target)
                    .is_some()
                    || world
                        .get::<crate::components::select::component::SelectButton>(target)
                        .is_some()
            };
            if matches {
                return Some((if composite { entity } else { target }, target));
            }
            if let Some(children) = world.get::<Children>(target) {
                pending.extend(children.iter());
            }
        }
        return None;
    }
    Some((entity, entity))
}

fn apply_styles(world: &mut World) {
    let mut styles: Vec<_> = world
        .query::<(Entity, &UiStyle)>()
        .iter(world)
        .map(|(entity, style)| (entity, style.clone()))
        .collect();
    styles.sort_by_key(|(entity, _)| {
        let mut depth = 0;
        let mut parent = world.get::<ChildOf>(*entity).map(ChildOf::parent);
        while let Some(entity) = parent {
            depth += 1;
            parent = world.get::<ChildOf>(entity).map(ChildOf::parent);
        }
        depth
    });
    let opacity = styles.iter().any(|(_, style)| {
        style.opacity.is_some()
            || style
                .state_styles
                .iter()
                .any(|(_, patch)| patch.opacity.is_some())
    });
    for (entity, style) in styles {
        let resolved = resolve_style(world, entity, &style);
        apply_style(world, entity, &resolved);
    }
    if !opacity
        && world
            .query_filtered::<Entity, With<OpacitySource>>()
            .iter(world)
            .next()
            .is_none()
    {
        return;
    }
    apply_opacity(world);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::dropdown::DropdownConfig;
    use crate::components::input::TextInputConfig;
    use crate::components::modal::{BasicModalContent, ModalStyle};
    use crate::prelude::*;

    fn root(app: &mut App) -> Entity {
        app.world_mut().spawn(Node::default()).id()
    }

    #[test]
    fn styling_child_overrides_parent_and_outline_does_not_create_a_fill() {
        let mut app = App::new();
        app.add_plugins(StylingPlugin);
        let root = root(&mut app);
        let parent = column()
            .color("red")
            .font_size(24)
            .children(
                text("Child")
                    .color("blue")
                    .font_size(14)
                    .scale(1.25)
                    .outline_width(2)
                    .outline_color("gold"),
            )
            .spawn_in_context(&mut UiBuildContext {
                world: app.world_mut(),
                parent: root,
            });
        app.update();
        let child = app.world().get::<Children>(parent).unwrap()[0];
        assert_eq!(
            app.world().get::<TextColor>(child).unwrap().0,
            Color::srgb(0.0, 0.0, 1.0)
        );
        assert_eq!(
            app.world().get::<TextFont>(child).unwrap().font_size,
            FontSize::Px(14.0)
        );
        assert_eq!(
            app.world().get::<UiTransform>(child).unwrap().scale,
            Vec2::splat(1.25)
        );
        assert_eq!(app.world().get::<Outline>(child).unwrap().width, px(2));
        assert!(app.world().get::<Surface>(child).is_none());
    }

    #[test]
    fn interaction_state_patches_resolve_with_disabled_highest_priority() {
        let mut app = App::new();
        app.add_plugins(StylingPlugin);
        let parent = root(&mut app);
        let button = UiElement::column()
            .background("white")
            .hover(|style| style.background("red"))
            .focused(|style| style.background("blue"))
            .pressed(|style| style.background("green"))
            .active(|style| style.radius(10))
            .inactive(|style| style.radius(2))
            .disabled_style(|style| style.background("gray"))
            .success_style(|style| style.border_width(3))
            .submitting_style(|style| style.opacity(0.5))
            .scale(0.9)
            .spawn_in_context(&mut UiBuildContext {
                world: app.world_mut(),
                parent,
            });
        app.world_mut().entity_mut(button).insert((
            Interaction::Hovered,
            crate::primitives::semantic::SemanticNode::new(
                crate::primitives::semantic::SemanticRole::Button,
            ),
        ));
        app.update();
        assert_eq!(
            app.world().get::<Surface>(button).unwrap().fill,
            Paint::solid(Color::srgb(1.0, 0.0, 0.0))
        );
        app.world_mut()
            .get_mut::<crate::primitives::semantic::SemanticNode>(button)
            .unwrap()
            .state
            .focused = true;
        app.update();
        assert_eq!(
            app.world().get::<Surface>(button).unwrap().fill,
            Paint::solid(Color::srgb(0.0, 0.0, 1.0))
        );
        app.world_mut()
            .get_mut::<crate::primitives::semantic::SemanticNode>(button)
            .unwrap()
            .state
            .focused = false;
        app.world_mut()
            .get_mut::<Interaction>(button)
            .unwrap()
            .set_if_neq(Interaction::None);
        app.world_mut()
            .get_mut::<crate::primitives::semantic::SemanticNode>(button)
            .unwrap()
            .state
            .checked = Some(true);
        app.update();
        assert_eq!(
            app.world().get::<Node>(button).unwrap().border_radius,
            BorderRadius::all(px(10))
        );
        app.world_mut()
            .get_mut::<crate::primitives::semantic::SemanticNode>(button)
            .unwrap()
            .state
            .checked = Some(false);
        app.update();
        assert_eq!(
            app.world().get::<Node>(button).unwrap().border_radius,
            BorderRadius::all(px(2))
        );
        app.world_mut()
            .get_mut::<Interaction>(button)
            .unwrap()
            .set_if_neq(Interaction::Pressed);
        app.update();
        assert_eq!(
            app.world().get::<Surface>(button).unwrap().fill,
            Paint::solid(Color::srgb(0.0, 128.0 / 255.0, 0.0))
        );
        app.world_mut()
            .get_mut::<crate::primitives::semantic::SemanticNode>(button)
            .unwrap()
            .state
            .disabled = true;
        app.update();
        assert_eq!(
            app.world().get::<Surface>(button).unwrap().fill,
            Paint::solid(Color::srgb(128.0 / 255.0, 128.0 / 255.0, 128.0 / 255.0))
        );
        assert_eq!(
            app.world().get::<UiTransform>(button).unwrap().scale,
            Vec2::splat(0.9)
        );
        app.world_mut()
            .get_mut::<crate::primitives::semantic::SemanticNode>(button)
            .unwrap()
            .state
            .disabled = false;
        app.world_mut()
            .entity_mut(button)
            .insert(crate::primitives::binding::FormStatusState::default());
        app.world_mut()
            .get_mut::<crate::primitives::binding::FormStatusState>(button)
            .unwrap()
            .submitting();
        app.update();
        let alpha = app
            .world()
            .get::<Surface>(button)
            .unwrap()
            .fill
            .to_linear_rgba()
            .unwrap()
            .alpha;
        assert!(
            (alpha - 0.5).abs() < 0.01,
            "expected submitting opacity 0.5, got {alpha}"
        );
        app.world_mut()
            .get_mut::<crate::primitives::binding::FormStatusState>(button)
            .unwrap()
            .success();
        app.update();
        assert_eq!(
            app.world().get::<Node>(button).unwrap().border,
            UiRect::all(px(3))
        );
    }

    #[test]
    fn styling_opacity_is_stable_composes_and_restores_gradient_and_text_alpha() {
        let mut app = App::new();
        app.add_plugins(StylingPlugin);
        let parent = app
            .world_mut()
            .spawn((
                Node::default(),
                UiStyle {
                    opacity: Some(0.5),
                    ..default()
                },
            ))
            .id();
        let child = app
            .world_mut()
            .spawn((
                Text::new("Text"),
                TextColor(Color::srgba(1.0, 0.0, 0.0, 0.8)),
                Surface::rounded_rect_fill(
                    0.0,
                    Paint::linear(crate::rendering::LinearGradient::horizontal(vec![
                        crate::rendering::GradientStop::new(0.0, Color::WHITE),
                        crate::rendering::GradientStop::new(1.0, Color::srgba(0.0, 0.0, 0.0, 0.4)),
                    ])),
                ),
                UiStyle {
                    opacity: Some(0.5),
                    ..default()
                },
                ChildOf(parent),
            ))
            .id();
        for _ in 0..4 {
            app.update();
        }
        assert!((app.world().get::<TextColor>(child).unwrap().0.alpha() - 0.2).abs() < 1e-6);
        let Paint::LinearGradient(gradient) = &app.world().get::<Surface>(child).unwrap().fill
        else {
            panic!("gradient lost")
        };
        assert!((gradient.stops[0].color.alpha() - 0.25).abs() < 1e-6);
        app.world_mut().get_mut::<UiStyle>(parent).unwrap().opacity = Some(1.0);
        app.world_mut().get_mut::<UiStyle>(child).unwrap().opacity = Some(1.0);
        app.update();
        assert!((app.world().get::<TextColor>(child).unwrap().0.alpha() - 0.8).abs() < 1e-6);
    }

    #[test]
    fn styling_form_spacing_sizes_and_paints_survive_visual_writes() {
        let mut app = App::new();
        app.add_plugins(StylingPlugin);
        let parent = root(&mut app);
        let form = UiElement::form(Form::new().show_submit_button(false))
            .padding(24)
            .padding_x(12)
            .radius(8)
            .gap(16)
            .width(400)
            .max_width(percent(100))
            .background("#123456")
            .border_width(2)
            .border_color("red")
            .color("white")
            .font_size(18)
            .header(text("Title"))
            .body(text("Content"))
            .footer(text("Actions"))
            .spawn_in_context(&mut UiBuildContext {
                world: app.world_mut(),
                parent,
            });
        app.update();
        let node = app.world().get::<Node>(form).unwrap();
        assert_eq!(node.width, px(400));
        assert_eq!(node.max_width, percent(100));
        assert_eq!(node.padding, UiRect::axes(px(12), px(24)));
        assert_eq!(node.row_gap, px(16));
        assert_eq!(node.border, UiRect::all(px(2)));
        let fill = app.world().get::<Surface>(form).unwrap().fill.clone();
        app.world_mut().get_mut::<Surface>(form).unwrap().fill = Paint::solid(Color::BLACK);
        app.update();
        assert_eq!(app.world().get::<Surface>(form).unwrap().fill, fill);
        assert_eq!(
            app.world().get::<BackgroundColor>(form).unwrap().0,
            Color::NONE
        );
        assert_eq!(
            app.world_mut().query::<&Text>().iter(app.world()).count(),
            3
        );
        for (_, color, font) in app
            .world_mut()
            .query::<(&Text, &TextColor, &TextFont)>()
            .iter(app.world())
        {
            assert_eq!(color.0, Color::srgb(1.0, 1.0, 1.0));
            assert_eq!(font.font_size, FontSize::Px(18.0));
        }
    }

    #[test]
    fn styling_targets_dropdown_and_modal_surfaces_not_overlay_roots() {
        let mut app = App::new();
        app.add_plugins(StylingPlugin)
            .insert_resource(ThemeResource::default());
        let parent = root(&mut app);
        let dropdown = UiElement::dropdown(DropdownConfig::new("Plan").option("Free", "free"))
            .width(320)
            .padding(9)
            .background("rebeccapurple")
            .spawn_in_context(&mut UiBuildContext {
                world: app.world_mut(),
                parent,
            });
        let modal = UiElement::modal(
            Modal::new(),
            ModalStyle::default(),
            BasicModalContent::new("Dialog", "Body"),
        )
        .width(360)
        .radius(8)
        .background("rgb(255, 0, 0)")
        .spawn_in_context(&mut UiBuildContext {
            world: app.world_mut(),
            parent,
        });
        app.update();
        let (_, trigger) = style_targets(app.world(), dropdown).unwrap();
        assert_ne!(trigger, dropdown);
        assert_eq!(app.world().get::<Node>(dropdown).unwrap().width, px(320));
        assert_eq!(
            app.world().get::<Node>(trigger).unwrap().padding,
            UiRect::all(px(9))
        );
        assert!(app.world().get::<Surface>(dropdown).is_none());
        let (_, surface) = style_targets(app.world(), modal).unwrap();
        assert_ne!(surface, modal);
        assert_eq!(app.world().get::<Node>(surface).unwrap().width, px(360));
        assert_eq!(app.world().get::<Node>(modal).unwrap().width, percent(100));
    }

    #[test]
    fn styling_error_wrappers_keep_host_width_and_both_spawn_paths() {
        let mut app = App::new();
        app.add_plugins((StylingPlugin, BindingPlugin));
        let parent = root(&mut app);
        let input = TextInputConfig::new("Name")
            .label("Name")
            .error(|errors| text(errors.join("\n")))
            .width(280)
            .padding(10)
            .spawn_in_context(&mut UiBuildContext {
                world: app.world_mut(),
                parent,
            });
        app.update();
        let host = app.world().get::<ChildOf>(input).unwrap().parent();
        assert_eq!(app.world().get::<Node>(input).unwrap().width, px(280));
        assert_eq!(app.world().get::<Node>(host).unwrap().width, px(280));
        let mut spawned = None;
        app.world_mut()
            .commands()
            .entity(parent)
            .with_children(|children| {
                spawned = Some(
                    column()
                        .padding(7)
                        .gap(3)
                        .children(text("Child"))
                        .spawn(children),
                );
            });
        app.world_mut().flush();
        app.update();
        let node = app.world().get::<Node>(spawned.unwrap()).unwrap();
        assert_eq!(node.padding, UiRect::all(px(7)));
        assert_eq!(node.row_gap, px(3));
    }
}
