use bevy::prelude::*;
use std::{any::type_name, fmt, sync::Arc};

/// Validation errors returned by a form-field setter.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BindingErrors {
    pub messages: Vec<String>,
}

impl BindingErrors {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            messages: vec![message.into()],
        }
    }

    pub fn many<I, S>(messages: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            messages: messages.into_iter().map(Into::into).collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

impl From<String> for BindingErrors {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

impl From<&str> for BindingErrors {
    fn from(message: &str) -> Self {
        Self::new(message)
    }
}

impl fmt::Display for BindingErrors {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.messages.join("; "))
    }
}

impl std::error::Error for BindingErrors {}

#[derive(Clone, Debug, PartialEq)]
pub enum BindingValue {
    Text(String),
    Bool(bool),
    Number(f64),
    Files(Vec<crate::components::file_input::SelectedFile>),
}

pub trait BindingValueType: Clone + PartialEq + Send + Sync + 'static {
    fn to_binding_value(&self) -> BindingValue;
    fn from_binding_value(value: BindingValue) -> Result<Self, BindingErrors>;
}

impl BindingValueType for String {
    fn to_binding_value(&self) -> BindingValue {
        BindingValue::Text(self.clone())
    }

    fn from_binding_value(value: BindingValue) -> Result<Self, BindingErrors> {
        match value {
            BindingValue::Text(value) => Ok(value),
            BindingValue::Bool(value) => Ok(value.to_string()),
            BindingValue::Number(value) => Ok(value.to_string()),
            BindingValue::Files(_) => Err(BindingErrors::new("file selection cannot bind to text")),
        }
    }
}

impl BindingValueType for bool {
    fn to_binding_value(&self) -> BindingValue {
        BindingValue::Bool(*self)
    }

    fn from_binding_value(value: BindingValue) -> Result<Self, BindingErrors> {
        match value {
            BindingValue::Bool(value) => Ok(value),
            BindingValue::Text(value) => value
                .parse()
                .map_err(|_| BindingErrors::new(format!("{value:?} is not a boolean"))),
            BindingValue::Number(_) => Err(BindingErrors::new("numeric value cannot bind to bool")),
            BindingValue::Files(_) => Err(BindingErrors::new("file selection cannot bind to bool")),
        }
    }
}

impl BindingValueType for Vec<crate::components::file_input::SelectedFile> {
    fn to_binding_value(&self) -> BindingValue {
        BindingValue::Files(self.clone())
    }

    fn from_binding_value(value: BindingValue) -> Result<Self, BindingErrors> {
        match value {
            BindingValue::Files(files) => Ok(files),
            _ => Err(BindingErrors::new(
                "non-file value cannot bind to a file input",
            )),
        }
    }
}

macro_rules! impl_numeric_binding_value {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl BindingValueType for $ty {
                fn to_binding_value(&self) -> BindingValue {
                    BindingValue::Number(*self as f64)
                }

                fn from_binding_value(value: BindingValue) -> Result<Self, BindingErrors> {
                    let text = match value {
                        BindingValue::Number(value) => value.to_string(),
                        BindingValue::Text(value) => value,
                        BindingValue::Bool(_) => {
                            return Err(BindingErrors::new("boolean value cannot bind to a number"));
                        }
                        BindingValue::Files(_) => {
                            return Err(BindingErrors::new("file selection cannot bind to a number"));
                        }
                    };
                    text.parse().map_err(|_| {
                        BindingErrors::new(format!("{text:?} is not a valid {}", stringify!($ty)))
                    })
                }
            }
        )+
    };
}

impl_numeric_binding_value!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, f32, f64);

/// A generated field handle used with `UiElement::bind`.
pub struct FieldBinding<Model, Value> {
    getter: fn(&Model) -> Value,
    setter: fn(&mut Model, Value) -> Result<(), BindingErrors>,
}

impl<Model, Value> Copy for FieldBinding<Model, Value> {}

impl<Model, Value> Clone for FieldBinding<Model, Value> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Model, Value> FieldBinding<Model, Value> {
    pub const fn new(
        getter: fn(&Model) -> Value,
        setter: fn(&mut Model, Value) -> Result<(), BindingErrors>,
    ) -> Self {
        Self { getter, setter }
    }

    pub fn erase(self) -> Arc<dyn ErasedFieldBinding>
    where
        Model: Resource,
        Value: BindingValueType,
    {
        Arc::new(TypedFieldBinding {
            getter: self.getter,
            setter: self.setter,
            marker: std::marker::PhantomData,
        })
    }
}

pub trait ErasedFieldBinding: Send + Sync {
    fn read(&self, world: &World) -> Result<BindingValue, BindingErrors>;
    fn write(&self, world: &mut World, value: BindingValue) -> Result<BindingValue, BindingErrors>;
}

struct TypedFieldBinding<Model, Value> {
    getter: fn(&Model) -> Value,
    setter: fn(&mut Model, Value) -> Result<(), BindingErrors>,
    marker: std::marker::PhantomData<fn() -> (Model, Value)>,
}

impl<Model, Value> ErasedFieldBinding for TypedFieldBinding<Model, Value>
where
    Model: Resource,
    Value: BindingValueType,
{
    fn read(&self, world: &World) -> Result<BindingValue, BindingErrors> {
        let model = world.get_resource::<Model>().ok_or_else(|| {
            BindingErrors::new(format!(
                "model resource {} is not present",
                type_name::<Model>()
            ))
        })?;
        Ok((self.getter)(&model).to_binding_value())
    }

    fn write(&self, world: &mut World, value: BindingValue) -> Result<BindingValue, BindingErrors> {
        let value = Value::from_binding_value(value)?;
        world.resource_scope(|_, mut model: Mut<Model>| (self.setter)(&mut model, value))?;
        self.read(world)
    }
}

#[derive(Component, Clone)]
pub struct BoundField {
    binding: Arc<dyn ErasedFieldBinding>,
    previous_model: Option<BindingValue>,
    previous_control: Option<BindingValue>,
    error: Option<BindingErrors>,
}

impl BoundField {
    pub fn new(binding: Arc<dyn ErasedFieldBinding>) -> Self {
        Self {
            binding,
            previous_model: None,
            previous_control: None,
            error: None,
        }
    }
}

pub struct BindingPlugin;

pub type ErrorCallback =
    Arc<dyn Fn(&[String]) -> Vec<crate::primitives::composition::UiElement> + Send + Sync>;

pub type FormStateCallback =
    Arc<dyn Fn() -> Vec<crate::primitives::composition::UiElement> + Send + Sync>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FormStatus {
    #[default]
    Idle,
    Submitting,
    Success,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FormStatusState {
    pub status: FormStatus,
}

impl FormStatusState {
    pub fn idle(&mut self) {
        self.status = FormStatus::Idle;
    }

    pub fn submitting(&mut self) {
        self.status = FormStatus::Submitting;
    }

    pub fn success(&mut self) {
        self.status = FormStatus::Success;
    }
}

#[derive(Component)]
pub(crate) struct FormStatePresentation {
    pub host: Entity,
    pub submitting: Option<FormStateCallback>,
    pub success: Option<FormStateCallback>,
    previous: Option<FormStatus>,
    content: Option<Entity>,
}

impl FormStatePresentation {
    pub(crate) fn new(
        host: Entity,
        submitting: Option<FormStateCallback>,
        success: Option<FormStateCallback>,
    ) -> Self {
        Self {
            host,
            submitting,
            success,
            previous: None,
            content: None,
        }
    }
}

#[derive(Component)]
pub(crate) struct ErrorPresentation {
    pub callback: ErrorCallback,
    pub host: Entity,
    previous: Vec<String>,
    content: Option<Entity>,
}

impl ErrorPresentation {
    pub(crate) fn new(callback: ErrorCallback, host: Entity) -> Self {
        Self {
            callback,
            host,
            previous: Vec::new(),
            content: None,
        }
    }
}

#[derive(Component)]
pub(crate) struct ErrorPresentationHost(pub Entity);

fn render_error_presentations(world: &mut World) {
    let orphaned: Vec<_> = world
        .query::<(Entity, &ErrorPresentationHost)>()
        .iter(world)
        .filter(|(_, host)| world.get_entity(host.0).is_err())
        .map(|(entity, _)| entity)
        .collect();
    for entity in orphaned {
        world.despawn(entity);
    }
    let entities: Vec<_> = world
        .query_filtered::<Entity, With<ErrorPresentation>>()
        .iter(world)
        .collect();
    for entity in entities {
        let messages = world
            .get::<BindingErrorState>(entity)
            .map(|state| state.messages.clone())
            .unwrap_or_default();
        let presentation = world.get::<ErrorPresentation>(entity).unwrap();
        if presentation.previous == messages {
            continue;
        }
        let callback = presentation.callback.clone();
        let host = presentation.host;
        let content = presentation.content;
        if let Some(content) = content {
            let focused = world
                .get_resource::<bevy::input_focus::InputFocus>()
                .and_then(|focus| focus.get());
            let mut ancestor = focused;
            while let Some(target) = ancestor {
                if target == content {
                    if world.contains_resource::<bevy::ecs::message::Messages<crate::primitives::focus::FocusRequest>>() {
                        world.write_message(crate::primitives::focus::FocusRequest { target: entity, origin: crate::primitives::focus::FocusOrigin::Keyboard });
                    }
                    break;
                }
                ancestor = world.get::<ChildOf>(target).map(ChildOf::parent);
            }
            world.despawn(content);
        }
        let content = if messages.is_empty() {
            None
        } else {
            let children = callback(&messages);
            let content = world
                .spawn((
                    Name::new("form-error-content"),
                    Node {
                        width: percent(100),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(4),
                        ..default()
                    },
                    ChildOf(host),
                ))
                .id();
            let mut context = crate::primitives::composition::UiBuildContext {
                world,
                parent: content,
            };
            for child in children {
                let plain_text = matches!(
                    child,
                    crate::primitives::composition::UiElement::Text { .. }
                );
                let child = child.spawn_in_context(&mut context);
                if plain_text {
                    let color = context
                        .world
                        .get_resource::<crate::theme::ThemeResource>()
                        .map_or_else(
                            || crate::theme::light_theme().colors.error,
                            |theme| theme.current.colors.error,
                        );
                    context.world.entity_mut(child).insert((
                        TextColor(color),
                        crate::components::text::ThemedText::new(
                            crate::components::text::TextRole::Body,
                        )
                        .color(color),
                    ));
                }
            }
            Some(content)
        };
        let mut presentation = world.get_mut::<ErrorPresentation>(entity).unwrap();
        presentation.previous = messages;
        presentation.content = content;
    }
}

fn render_form_state_presentations(world: &mut World) {
    let entities: Vec<_> = world
        .query_filtered::<Entity, With<FormStatePresentation>>()
        .iter(world)
        .collect();
    for entity in entities {
        let status = world
            .get::<FormStatusState>(entity)
            .copied()
            .unwrap_or_default()
            .status;
        let presentation = world.get::<FormStatePresentation>(entity).unwrap();
        if presentation.previous == Some(status) {
            continue;
        }
        let host = presentation.host;
        let callback = match status {
            FormStatus::Idle => None,
            FormStatus::Submitting => presentation.submitting.clone(),
            FormStatus::Success => presentation.success.clone(),
        };
        if let Some(content) = presentation.content {
            if let Some(focused) = world
                .get_resource::<bevy::input_focus::InputFocus>()
                .and_then(|focus| focus.get())
            {
                let mut ancestor = Some(focused);
                while let Some(target) = ancestor {
                    if target == content {
                        if world.contains_resource::<
                            bevy::ecs::message::Messages<crate::primitives::focus::FocusRequest>,
                        >() {
                            world.write_message(crate::primitives::focus::FocusRequest {
                                target: entity,
                                origin: crate::primitives::focus::FocusOrigin::Keyboard,
                            });
                        }
                        break;
                    }
                    ancestor = world.get::<ChildOf>(target).map(ChildOf::parent);
                }
            }
            world.despawn(content);
        }
        let content = callback.map(|callback| {
            let mut semantic = crate::primitives::semantic::SemanticNode::new(
                crate::primitives::semantic::SemanticRole::Status,
            );
            semantic.live = Some(crate::primitives::semantic::AnnouncementPriority::Polite);
            semantic.state.busy = status == FormStatus::Submitting;
            let content = world
                .spawn((
                    Name::new("form-state-content"),
                    semantic,
                    Node {
                        width: percent(100),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(4),
                        ..default()
                    },
                    ChildOf(host),
                ))
                .id();
            let mut context = crate::primitives::composition::UiBuildContext {
                world,
                parent: content,
            };
            for child in callback() {
                child.spawn_in_context(&mut context);
            }
            content
        });
        let mut presentation = world.get_mut::<FormStatePresentation>(entity).unwrap();
        presentation.previous = Some(status);
        presentation.content = content;
    }
}

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct BindingSystems;

impl Plugin for BindingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                sync_bound_fields,
                render_error_presentations,
                render_form_state_presentations,
            )
                .chain()
                .in_set(BindingSystems),
        )
        .configure_sets(
            PostUpdate,
            BindingSystems.before(bevy::ui::UiSystems::Layout),
        );
    }
}

pub(crate) fn sync_bound_fields(world: &mut World) {
    let mut query = world.query_filtered::<Entity, With<BoundField>>();
    let entities = query.iter(world).collect::<Vec<_>>();

    for entity in entities {
        let Some(bound) = world.get::<BoundField>(entity).cloned() else {
            continue;
        };
        let model_value = bound.binding.read(world);
        let control_value = read_control_value(world, entity);
        let mut previous_model = bound.previous_model;
        let mut previous_control = bound.previous_control;
        let mut error = bound.error;

        match (model_value, control_value) {
            (Ok(model_value), Ok(control_value)) => {
                if previous_model.is_none() || previous_control.is_none() {
                    match write_control_value(world, entity, &model_value) {
                        Ok(()) => {
                            error = None;
                            previous_model = Some(model_value);
                            previous_control = read_control_value(world, entity).ok();
                        }
                        Err(write_error) => error = Some(write_error),
                    }
                } else if previous_control.as_ref() != Some(&control_value) {
                    match bound.binding.write(world, control_value.clone()) {
                        Ok(normalized_value) => {
                            match write_control_value(world, entity, &normalized_value) {
                                Ok(()) => {
                                    error = None;
                                    previous_model = Some(normalized_value);
                                    previous_control = read_control_value(world, entity).ok();
                                }
                                Err(write_error) => error = Some(write_error),
                            }
                        }
                        Err(setter_error) => {
                            previous_model = Some(model_value);
                            previous_control = Some(control_value);
                            error = Some(setter_error);
                        }
                    }
                } else if previous_model.as_ref() != Some(&model_value) {
                    match write_control_value(world, entity, &model_value) {
                        Ok(()) => {
                            error = None;
                            previous_model = Some(model_value);
                            previous_control = read_control_value(world, entity).ok();
                        }
                        Err(write_error) => error = Some(write_error),
                    }
                }
            }
            (Err(binding_error), _) | (_, Err(binding_error)) => error = Some(binding_error),
        }

        if let Some(mut bound) = world.get_mut::<BoundField>(entity) {
            bound.previous_model = previous_model;
            bound.previous_control = previous_control;
            bound.error = error.clone();
        }
        if let Some(mut state) = world.get_mut::<BindingErrorState>(entity) {
            state.set_if_neq(BindingErrorState {
                messages: error.map(|error| error.messages).unwrap_or_default(),
            });
        }
    }
}

#[derive(Component, Clone, Copy)]
pub(crate) struct SelectBindingAdapter {
    read: fn(&World, Entity) -> Result<BindingValue, BindingErrors>,
    write: fn(&mut World, Entity, &BindingValue) -> Result<(), BindingErrors>,
}

impl SelectBindingAdapter {
    pub(crate) fn new<T: BindingValueType>() -> Self {
        Self {
            read: |world, entity| {
                let select = world
                    .get::<crate::components::select::Select<T>>(entity)
                    .unwrap();
                Ok(select.selected_value().map_or_else(
                    || BindingValue::Text(String::new()),
                    BindingValueType::to_binding_value,
                ))
            },
            write: |world, entity, value| {
                let value = T::from_binding_value(value.clone())?;
                let mut select = world
                    .get_mut::<crate::components::select::Select<T>>(entity)
                    .unwrap();
                let selected = select
                    .options
                    .iter()
                    .position(|option| option == &value)
                    .ok_or_else(|| BindingErrors::new("model value is not a select option"))?;
                select.selected = Some(selected);
                Ok(())
            },
        }
    }
}

fn read_control_value(world: &World, entity: Entity) -> Result<BindingValue, BindingErrors> {
    if let Some(adapter) = world.get::<SelectBindingAdapter>(entity) {
        return (adapter.read)(world, entity);
    }
    if let Some(search) = world.get::<crate::components::search::Search>(entity) {
        let nested_input = world.get::<Children>(entity).and_then(|children| {
            children
                .iter()
                .find_map(|child| world.get::<crate::components::input::TextInput>(child))
        });
        return Ok(BindingValue::Text(nested_input.map_or_else(
            || search.query.clone(),
            |input| input.value.clone(),
        )));
    }
    if let Some(input) = world.get::<crate::components::input::TextInput>(entity) {
        return Ok(BindingValue::Text(input.value.clone()));
    }
    if let Some(textarea) = world.get::<crate::components::textarea::Textarea>(entity) {
        return Ok(BindingValue::Text(textarea.value.clone()));
    }
    if let Some(state) = world.get::<crate::components::checkbox::CheckboxState>(entity) {
        return Ok(BindingValue::Bool(state.checked));
    }
    if let Some(toggle) = world.get::<crate::components::toggle::Toggle>(entity) {
        return Ok(BindingValue::Bool(toggle.checked));
    }
    if let Some(group) = world.get::<crate::components::radio::RadioGroup>(entity) {
        return Ok(BindingValue::Text(group.selected.clone()));
    }
    if let Some(select) = world.get::<crate::components::select::Select<String>>(entity) {
        return Ok(BindingValue::Text(
            select.selected_value().cloned().unwrap_or_default(),
        ));
    }
    if let Some(slider) = world.get::<crate::components::slider::Slider>(entity) {
        return Ok(BindingValue::Number(slider.value as f64));
    }
    if let Some(files) = world.get::<crate::components::file_input::FileInputSelectionState>(entity)
    {
        return Ok(BindingValue::Files(files.files.clone()));
    }
    if let (Some(dropdown), Some(state)) = (
        world.get::<crate::components::dropdown::Dropdown>(entity),
        world.get::<crate::components::dropdown::DropdownState>(entity),
    ) {
        let value = state
            .selected
            .and_then(|index| dropdown.options.get(index))
            .map(|option| option.value.clone())
            .unwrap_or_default();
        return Ok(BindingValue::Text(value));
    }
    Err(BindingErrors::new(
        "`.bind(...)` requires a supported form control",
    ))
}

fn write_control_value(
    world: &mut World,
    entity: Entity,
    value: &BindingValue,
) -> Result<(), BindingErrors> {
    if let Some(adapter) = world.get::<SelectBindingAdapter>(entity).copied() {
        return (adapter.write)(world, entity, value);
    }
    if world
        .get::<crate::components::search::Search>(entity)
        .is_some()
    {
        let value = binding_value_to_text(value)?;
        if let Some(mut search) = world.get_mut::<crate::components::search::Search>(entity) {
            search.query = value.clone();
        }
        if let Some(child) = world.get::<Children>(entity).and_then(|children| {
            children.iter().find_map(|child| {
                world
                    .get::<crate::components::input::TextInput>(child)
                    .map(|_| child)
            })
        }) {
            if let Some(mut input) = world.get_mut::<crate::components::input::TextInput>(child) {
                input.value = value.clone();
                input.cursor = value.chars().count();
            }
        }
        return Ok(());
    }
    if let Some(mut input) = world.get_mut::<crate::components::input::TextInput>(entity) {
        input.value = binding_value_to_text(value)?;
        input.cursor = input.value.chars().count();
        return Ok(());
    }
    if let Some(mut textarea) = world.get_mut::<crate::components::textarea::Textarea>(entity) {
        textarea.set_value(binding_value_to_text(value)?);
        return Ok(());
    }
    if let Some(mut state) = world.get_mut::<crate::components::checkbox::CheckboxState>(entity) {
        state.checked = binding_value_to_bool(value)?;
        state.indeterminate = false;
        return Ok(());
    }
    if let Some(mut toggle) = world.get_mut::<crate::components::toggle::Toggle>(entity) {
        toggle.checked = binding_value_to_bool(value)?;
        return Ok(());
    }
    if let Some(mut group) = world.get_mut::<crate::components::radio::RadioGroup>(entity) {
        group.selected = binding_value_to_text(value)?;
        return Ok(());
    }
    if let Some(mut select) = world.get_mut::<crate::components::select::Select<String>>(entity) {
        let value = binding_value_to_text(value)?;
        select.selected = select.options.iter().position(|option| option == &value);
        return Ok(());
    }
    if let Some(mut slider) = world.get_mut::<crate::components::slider::Slider>(entity) {
        slider.value = (binding_value_to_number(value)? as f32).clamp(slider.min, slider.max);
        return Ok(());
    }
    if let Some(mut files) =
        world.get_mut::<crate::components::file_input::FileInputSelectionState>(entity)
    {
        match value {
            BindingValue::Files(value) => files.files = value.clone(),
            _ => {
                return Err(BindingErrors::new(
                    "non-file value cannot bind to a file input",
                ));
            }
        }
        return Ok(());
    }
    let dropdown_options = world
        .get::<crate::components::dropdown::Dropdown>(entity)
        .map(|dropdown| dropdown.options.clone());
    if let (Some(options), Some(mut state)) = (
        dropdown_options,
        world.get_mut::<crate::components::dropdown::DropdownState>(entity),
    ) {
        let value = binding_value_to_text(value)?;
        state.selected = options.iter().position(|option| option.value == value);
        return Ok(());
    }
    Err(BindingErrors::new(
        "`.bind(...)` requires a supported form control",
    ))
}

fn binding_value_to_text(value: &BindingValue) -> Result<String, BindingErrors> {
    match value {
        BindingValue::Text(value) => Ok(value.clone()),
        BindingValue::Bool(value) => Ok(value.to_string()),
        BindingValue::Number(value) => Ok(value.to_string()),
        BindingValue::Files(_) => Err(BindingErrors::new("file selection cannot bind to text")),
    }
}

fn binding_value_to_bool(value: &BindingValue) -> Result<bool, BindingErrors> {
    match value {
        BindingValue::Bool(value) => Ok(*value),
        BindingValue::Text(value) => value
            .parse()
            .map_err(|_| BindingErrors::new(format!("{value:?} is not a boolean"))),
        BindingValue::Number(_) => Err(BindingErrors::new("numeric value cannot bind to bool")),
        BindingValue::Files(_) => Err(BindingErrors::new("file selection cannot bind to bool")),
    }
}

fn binding_value_to_number(value: &BindingValue) -> Result<f64, BindingErrors> {
    match value {
        BindingValue::Number(value) => Ok(*value),
        BindingValue::Text(value) => value
            .parse()
            .map_err(|_| BindingErrors::new(format!("{value:?} is not a number"))),
        BindingValue::Bool(_) => Err(BindingErrors::new("boolean value cannot bind to a number")),
        BindingValue::Files(_) => Err(BindingErrors::new("file selection cannot bind to a number")),
    }
}

/// Internal representation of a write-only typed model setter supplied
/// directly to a form control with `.bind(Model::setter)`.
///
/// The model owns its fields and validation. Beverly only receives the model
/// resource and invokes the setter method supplied by the application.
#[derive(Clone, Copy)]
pub struct SetterBinding<Model, Value> {
    setter: fn(&mut Model, Value) -> Result<(), BindingErrors>,
}

impl<Model, Value> SetterBinding<Model, Value>
where
    Model: Resource,
{
    pub fn new(setter: fn(&mut Model, Value) -> Result<(), BindingErrors>) -> Self {
        Self { setter }
    }

    pub fn apply(&self, world: &mut World, value: Value) -> Result<(), BindingErrors> {
        world.resource_scope(|_, mut model: Mut<Model>| (self.setter)(&mut model, value))
    }
}

/// Field-level validation state produced by a bound setter.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq)]
pub struct BindingErrorState {
    pub messages: Vec<String>,
}

impl BindingErrorState {
    pub fn error(&mut self, errors: BindingErrors) {
        self.messages = errors.messages;
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::input::{TextInput, TextInputKind};
    use crate::components::search::Search;
    use std::fmt;

    #[derive(Debug)]
    struct NameError;

    impl fmt::Display for NameError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("name must not be empty")
        }
    }

    #[crate::model]
    struct UserModel {
        #[setter = normalize_name]
        name: String,
        age: u32,
    }

    impl UserModel {
        fn normalize_name(&mut self, value: String) -> Result<(), NameError> {
            let value = value.trim().to_owned();
            if value.is_empty() {
                return Err(NameError);
            }
            self.name = value;
            Ok(())
        }
    }

    #[test]
    fn generated_custom_setter_normalizes_and_rejects_values() {
        let mut world = World::new();
        world.insert_resource(UserModel {
            name: "Initial".to_owned(),
            age: 30,
        });
        let name = UserModel::name.erase();

        assert_eq!(
            name.read(&world).unwrap(),
            BindingValue::Text("Initial".to_owned())
        );
        assert_eq!(
            name.write(&mut world, BindingValue::Text("  Beverly  ".to_owned()))
                .unwrap(),
            BindingValue::Text("Beverly".to_owned()),
        );
        assert_eq!(
            name.write(&mut world, BindingValue::Text("   ".to_owned()))
                .unwrap_err()
                .messages,
            vec!["name must not be empty"],
        );
        assert_eq!(world.resource::<UserModel>().get_name(), "Beverly");
    }

    #[test]
    fn generated_default_setter_supports_numeric_conversion() {
        let mut world = World::new();
        world.insert_resource(UserModel {
            name: "Initial".to_owned(),
            age: 30,
        });
        let age = UserModel::age.erase();

        assert_eq!(
            age.write(&mut world, BindingValue::Text("42".to_owned()))
                .unwrap(),
            BindingValue::Number(42.0),
        );
        assert_eq!(world.resource::<UserModel>().age, 42);
        assert!(
            age.write(&mut world, BindingValue::Text("not a number".to_owned()))
                .is_err()
        );
    }

    fn bound_name_world() -> (World, Entity) {
        let mut world = World::new();
        world.insert_resource(UserModel {
            name: "Initial".to_owned(),
            age: 30,
        });
        let entity = world
            .spawn((
                TextInput {
                    value: String::new(),
                    placeholder: String::new(),
                    floating_label: None,
                    accessible_label: Some("Name".to_owned()),
                    kind: TextInputKind::Text,
                    max_length: None,
                    cursor: 0,
                    disabled: false,
                    read_only: false,
                    required: false,
                    invalid: false,
                },
                BoundField::new(UserModel::name.erase()),
                BindingErrorState::default(),
            ))
            .id();
        (world, entity)
    }

    #[test]
    fn bound_text_input_syncs_model_and_control_in_both_directions() {
        let (mut world, entity) = bound_name_world();

        sync_bound_fields(&mut world);
        assert_eq!(world.get::<TextInput>(entity).unwrap().value, "Initial");

        world.get_mut::<TextInput>(entity).unwrap().value = "  From UI  ".to_owned();
        sync_bound_fields(&mut world);
        assert_eq!(world.resource::<UserModel>().name, "From UI");
        assert_eq!(world.get::<TextInput>(entity).unwrap().value, "From UI");

        world.resource_mut::<UserModel>().name = "From model".to_owned();
        sync_bound_fields(&mut world);
        assert_eq!(world.get::<TextInput>(entity).unwrap().value, "From model");
    }

    #[test]
    fn bound_field_retains_validation_error_until_a_new_valid_edit() {
        let (mut world, entity) = bound_name_world();

        sync_bound_fields(&mut world);
        world.get_mut::<TextInput>(entity).unwrap().value = "   ".to_owned();
        sync_bound_fields(&mut world);
        assert_eq!(world.resource::<UserModel>().name, "Initial");
        assert_eq!(
            world.get::<BindingErrorState>(entity).unwrap().messages,
            vec!["name must not be empty"],
        );

        sync_bound_fields(&mut world);
        assert_eq!(
            world.get::<BindingErrorState>(entity).unwrap().messages,
            vec!["name must not be empty"],
        );

        world.get_mut::<TextInput>(entity).unwrap().value = "Valid".to_owned();
        sync_bound_fields(&mut world);
        assert!(
            world
                .get::<BindingErrorState>(entity)
                .unwrap()
                .messages
                .is_empty()
        );
        assert_eq!(world.resource::<UserModel>().name, "Valid");
    }

    #[test]
    fn bound_search_syncs_its_nested_input_and_model() {
        let mut world = World::new();
        world.insert_resource(UserModel {
            name: "Model query".to_owned(),
            age: 30,
        });
        let search = world.spawn(Search::new("global")).id();
        let input = world
            .spawn((
                TextInput {
                    value: String::new(),
                    placeholder: "Search".to_owned(),
                    floating_label: None,
                    accessible_label: Some("Search".to_owned()),
                    kind: TextInputKind::Search,
                    max_length: None,
                    cursor: 0,
                    disabled: false,
                    read_only: false,
                    required: false,
                    invalid: false,
                },
                ChildOf(search),
            ))
            .id();
        world.entity_mut(search).insert((
            BoundField::new(UserModel::name.erase()),
            BindingErrorState::default(),
        ));

        sync_bound_fields(&mut world);
        assert_eq!(world.get::<Search>(search).unwrap().query, "Model query");
        assert_eq!(world.get::<TextInput>(input).unwrap().value, "Model query");

        world.get_mut::<TextInput>(input).unwrap().value = "User query".to_owned();
        sync_bound_fields(&mut world);
        assert_eq!(world.resource::<UserModel>().name, "User query");
        assert_eq!(world.get::<Search>(search).unwrap().query, "User query");
    }

    #[test]
    fn bound_typed_select_syncs_values_and_reports_missing_options() {
        let mut world = World::new();
        world.insert_resource(UserModel {
            name: "Name".into(),
            age: 30,
        });
        let entity = world
            .spawn((
                crate::components::select::Select::new(vec![30u32, 42]),
                SelectBindingAdapter::new::<u32>(),
                BoundField::new(UserModel::age.erase()),
                BindingErrorState::default(),
            ))
            .id();
        sync_bound_fields(&mut world);
        assert_eq!(
            world
                .get::<crate::components::select::Select<u32>>(entity)
                .unwrap()
                .selected_value(),
            Some(&30)
        );
        world
            .get_mut::<crate::components::select::Select<u32>>(entity)
            .unwrap()
            .select(1);
        sync_bound_fields(&mut world);
        assert_eq!(world.resource::<UserModel>().age, 42);
        world.resource_mut::<UserModel>().age = 99;
        sync_bound_fields(&mut world);
        assert_eq!(
            world.get::<BindingErrorState>(entity).unwrap().messages,
            ["model value is not a select option"]
        );
        world.resource_mut::<UserModel>().age = 30;
        sync_bound_fields(&mut world);
        assert!(
            world
                .get::<BindingErrorState>(entity)
                .unwrap()
                .messages
                .is_empty()
        );
        assert_eq!(
            world
                .get::<crate::components::select::Select<u32>>(entity)
                .unwrap()
                .selected_value(),
            Some(&30)
        );
    }
}
