use bevy::prelude::*;
use std::sync::Arc;

use crate::components::alert::{Alert, AlertSections};
use crate::components::avatar::{AvatarConfig, spawn_avatar_in};
use crate::components::button::{BeverlyButton, ButtonChild, ButtonChildSetup, ButtonColor};
use crate::components::card::{CardBody, CardFooter, CardHeader, CardStyle, spawn_card};
use crate::components::checkbox::{CheckboxConfig, spawn_checkbox};
use crate::components::divider::Divider;
use crate::components::dropdown::{DropdownConfig, spawn_dropdown};
use crate::components::file_input::{FileInput, spawn_file_input};
use crate::components::footer::{FooterConfig, spawn_footer};
use crate::components::form::{Form, spawn_form};
use crate::components::image::{ImageConfig, ImageSource, spawn_image};
use crate::components::input::TextInputKind;
use crate::components::input::{TextInputConfig, spawn_text_input};
use crate::components::link::Link;
use crate::components::list_item::spawn_list_item;
use crate::components::modal::{
    BasicModalContent, Modal, ModalBody, ModalFooter, ModalHeader, ModalStyle, spawn_modal,
    spawn_modal_with_surface,
};
use crate::components::navbar::spawn_navbar;
use crate::components::pagination::{PaginationConfig, spawn_pagination};
use crate::components::photo::{Photo, spawn_photo};
use crate::components::progress_bar::{ProgressBar, spawn_progress_bar_into};
use crate::components::radio::RadioGroupBuilder;
use crate::components::search::spawn_search;
use crate::components::searchbox::spawn_searchbox;
use crate::components::select::{Select, spawn_select};
use crate::components::sidebar::spawn_sidebar;
use crate::components::slider::{Slider, SliderStyle, spawn_slider};
use crate::components::spinner::Spinner;
use crate::components::table::{Table, TableConfig};
use crate::components::tabs::{TabsConfig, spawn_tabs_in};
use crate::components::textarea::{TextareaConfig, spawn_textarea};
use crate::components::toast::{Toast, ToastSections};
use crate::components::toggle::{ToggleConfig, ToggleShadowMaterial, spawn_toggle};
use crate::components::tooltip::Tooltip;
use crate::icons::Icon;
use crate::primitives::root::UiFonts;
use crate::primitives::semantic::AriaDescription;
use crate::rendering::{Paint, Surface};
use crate::theme::ThemeResource;

/// A composable UI node used by fluent application builders.
#[derive(Clone)]
pub enum UiElement {
    Container {
        node: Node,
        children: Vec<UiElement>,
        header: Vec<UiElement>,
        body: Vec<UiElement>,
        footer: Vec<UiElement>,
    },
    Text {
        value: Text,
        children: Vec<UiElement>,
    },
    Button {
        value: BeverlyButton,
        children: Vec<UiElement>,
    },
    Checkbox {
        config: CheckboxConfig,
    },
    TextInput {
        config: TextInputConfig,
    },
    Alert {
        value: Alert,
        header: Vec<UiElement>,
        body: Vec<UiElement>,
        footer: Vec<UiElement>,
    },
    Toast {
        value: Toast,
        header: Vec<UiElement>,
        body: Vec<UiElement>,
        footer: Vec<UiElement>,
    },
    Divider(Divider),
    ProgressBar(ProgressBar),
    Dropdown(DropdownConfig),
    Pagination(PaginationConfig),
    Textarea(TextareaConfig),
    Toggle(ToggleConfig),
    Slider {
        value: Slider,
        style: SliderStyle,
    },
    Avatar(AvatarConfig),
    Photo(Photo),
    Image(ImageConfig),
    Card {
        style: CardStyle,
        children: Vec<UiElement>,
        header: Vec<UiElement>,
        body: Vec<UiElement>,
        footer: Vec<UiElement>,
    },
    ListItem {
        title: String,
        subtitle: Option<String>,
        icon: Option<Icon>,
    },
    Form {
        value: Form,
        children: Vec<UiElement>,
        header: Vec<UiElement>,
        body: Vec<UiElement>,
        footer: Vec<UiElement>,
    },
    Radio(RadioGroupBuilder),
    Table(TableConfig),
    Tabs {
        config: TabsConfig,
        children: Vec<UiElement>,
        header: Vec<UiElement>,
        body: Vec<UiElement>,
        footer: Vec<UiElement>,
    },
    Modal {
        value: Modal,
        style: ModalStyle,
        content: BasicModalContent,
        header: Vec<UiElement>,
        body: Vec<UiElement>,
        footer: Vec<UiElement>,
    },
    Search,
    SearchBox {
        placeholder: String,
    },
    FileInput(FileInput),
    Navbar {
        fixed: bool,
    },
    Footer {
        fixed: bool,
        config: FooterConfig,
    },
    Sidebar {
        top_offset_px: f32,
        header: Vec<UiElement>,
        body: Vec<UiElement>,
        footer: Vec<UiElement>,
    },
    Link(Link),
    Tooltip(Tooltip),
    Spinner(Spinner),
    Select(Select<String>),
    Icon(&'static str),
    Custom(UiElementSetup),
    ButtonCustom(ButtonChildSetup),
    ContextCustom(UiContextSetup),
    TypedSelect(Arc<dyn Fn(&mut UiBuildContext<'_>) -> Entity + Send + Sync>),
    Aria {
        element: Box<UiElement>,
        description: String,
    },
    Named {
        element: Box<UiElement>,
        name: String,
    },
    Bound {
        element: Box<UiElement>,
        binding: Arc<dyn crate::primitives::binding::ErasedFieldBinding>,
    },
    WithError {
        element: Box<UiElement>,
        callback: crate::primitives::binding::ErrorCallback,
    },
    Styled {
        element: Box<UiElement>,
        style: crate::primitives::style::UiStyle,
    },
    WithFormState {
        element: Box<UiElement>,
        submitting: Option<crate::primitives::binding::FormStateCallback>,
        success: Option<crate::primitives::binding::FormStateCallback>,
    },
}

impl IntoIterator for UiElement {
    type Item = UiElement;
    type IntoIter = std::iter::Once<UiElement>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self)
    }
}

/// Custom fluent element setup for components that need application-specific
/// construction while still participating in `.children([...])`.
pub type UiElementSetup = fn(&mut ChildSpawnerCommands) -> Entity;

/// Context-aware custom setup for components that need Bevy resources.
pub type UiContextSetup = fn(&mut UiBuildContext<'_>) -> Entity;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentSection {
    Header,
    Body,
    Footer,
}

#[derive(Component)]
struct PendingContentSection(Vec<UiElement>);

pub(crate) fn spawn_deferred_content_section(
    parent: &mut ChildSpawnerCommands,
    name: &'static str,
    children: Vec<UiElement>,
) {
    if children.is_empty() {
        return;
    }
    parent.spawn((
        Name::new(name),
        PendingContentSection(children),
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            ..default()
        },
    ));
}

pub(crate) fn materialize_content_sections(world: &mut World) {
    let sections: Vec<_> = world
        .query::<(Entity, &mut PendingContentSection)>()
        .iter_mut(world)
        .map(|(entity, mut pending)| (entity, std::mem::take(&mut pending.0)))
        .collect();
    for (entity, children) in sections {
        world.entity_mut(entity).remove::<PendingContentSection>();
        let mut context = UiBuildContext {
            world,
            parent: entity,
        };
        for child in children {
            child.spawn_in_context(&mut context);
        }
    }
}

/// Fluent constructor for the standalone Footer component.
///
/// This is distinct from the `.footer(...)` section helper on Card, Toast,
/// and other sectioned fluent elements.
pub fn footer(fixed: bool, config: FooterConfig) -> UiElement {
    UiElement::Footer { fixed, config }
}

pub fn row() -> UiElement {
    UiElement::row()
}

pub fn column() -> UiElement {
    UiElement::column()
}

/// World-backed context used while a fluent UI tree is being materialized.
pub struct UiBuildContext<'w> {
    pub world: &'w mut World,
    pub parent: Entity,
}

impl UiBuildContext<'_> {
    /// Returns the active page's route data while building routed content.
    pub fn route_context(&self) -> Option<crate::primitives::routing::RouteContext> {
        self.world
            .get_resource::<crate::primitives::routing::RouteState>()
            .map(crate::primitives::routing::RouteState::context)
    }
}

fn spawn_with_children<F>(world: &mut World, parent: Entity, spawn: F) -> Entity
where
    F: FnOnce(&mut ChildSpawnerCommands) -> Entity,
{
    let mut spawned = None;
    world
        .commands()
        .entity(parent)
        .with_children(|children| spawned = Some(spawn(children)));
    world.flush();
    spawned.expect("component spawn helper did not return an entity")
}

pub(crate) fn spawn_content_section<M: Component>(
    parent: &mut ChildSpawnerCommands,
    marker: M,
    children: Vec<UiElement>,
    grow: bool,
) {
    if children.is_empty() && !grow {
        return;
    }

    parent
        .spawn((
            marker,
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                flex_grow: if grow { 1.0 } else { 0.0 },
                flex_shrink: if grow { 1.0 } else { 0.0 },
                ..default()
            },
        ))
        .with_children(|section| {
            for child in children {
                child.spawn(section);
            }
        });
}

fn spawn_section_in_context<M: Component>(
    context: &mut UiBuildContext<'_>,
    marker: M,
    children: Vec<UiElement>,
    grow: bool,
) -> Option<Entity> {
    if children.is_empty() && !grow {
        return None;
    }

    let section = context
        .world
        .spawn((
            marker,
            ChildOf(context.parent),
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                flex_grow: if grow { 1.0 } else { 0.0 },
                flex_shrink: if grow { 1.0 } else { 0.0 },
                ..default()
            },
        ))
        .id();
    for child in children {
        child.spawn_in_context(&mut UiBuildContext {
            world: context.world,
            parent: section,
        });
    }
    Some(section)
}

impl UiElement {
    pub fn row() -> Self {
        Self::Container {
            node: Node {
                width: percent(100),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            children: Vec::new(),
            header: Vec::new(),
            body: Vec::new(),
            footer: Vec::new(),
        }
    }

    pub fn column() -> Self {
        Self::Container {
            node: Node {
                width: percent(100),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            children: Vec::new(),
            header: Vec::new(),
            body: Vec::new(),
            footer: Vec::new(),
        }
    }

    fn styled(self, update: impl FnOnce(&mut crate::primitives::style::UiStyle)) -> Self {
        match self {
            Self::Styled { element, mut style } => {
                update(&mut style);
                Self::Styled { element, style }
            }
            Self::Aria {
                element,
                description,
            } => Self::Aria {
                element: Box::new(element.styled(update)),
                description,
            },
            Self::Named { element, name } => Self::Named {
                element: Box::new(element.styled(update)),
                name,
            },
            Self::Bound { element, binding } => Self::Bound {
                element: Box::new(element.styled(update)),
                binding,
            },
            Self::WithError { element, callback } => Self::WithError {
                element: Box::new(element.styled(update)),
                callback,
            },
            element => {
                let mut style = crate::primitives::style::UiStyle::default();
                update(&mut style);
                Self::Styled {
                    element: Box::new(element),
                    style,
                }
            }
        }
    }

    fn with_state_style<F>(self, state: crate::primitives::style::StyleState, callback: F) -> Self
    where
        F: FnOnce(crate::primitives::style::UiStyle) -> crate::primitives::style::UiStyle,
    {
        match self {
            Self::Styled { element, mut style } => {
                let patch = callback(crate::primitives::style::UiStyle::default());
                if let Some((_, existing)) = style
                    .state_styles
                    .iter_mut()
                    .find(|(current, _)| *current == state)
                {
                    *existing = Box::new(patch);
                } else {
                    style.state_styles.push((state, Box::new(patch)));
                }
                Self::Styled { element, style }
            }
            Self::Aria {
                element,
                description,
            } => Self::Aria {
                element: Box::new(element.with_state_style(state, callback)),
                description,
            },
            Self::Named { element, name } => Self::Named {
                element: Box::new(element.with_state_style(state, callback)),
                name,
            },
            Self::Bound { element, binding } => Self::Bound {
                element: Box::new(element.with_state_style(state, callback)),
                binding,
            },
            Self::WithError {
                element,
                callback: error,
            } => Self::WithError {
                element: Box::new(element.with_state_style(state, callback)),
                callback: error,
            },
            Self::WithFormState {
                element,
                submitting,
                success,
            } => Self::WithFormState {
                element: Box::new(element.with_state_style(state, callback)),
                submitting,
                success,
            },
            element => {
                let patch = callback(crate::primitives::style::UiStyle::default());
                let mut style = crate::primitives::style::UiStyle::default();
                style.state_styles.push((state, Box::new(patch)));
                Self::Styled {
                    element: Box::new(element),
                    style,
                }
            }
        }
    }

    pub fn hover<F>(self, style: F) -> Self
    where
        F: FnOnce(crate::primitives::style::UiStyle) -> crate::primitives::style::UiStyle,
    {
        self.with_state_style(crate::primitives::style::StyleState::Hover, style)
    }
    pub fn focused<F>(self, style: F) -> Self
    where
        F: FnOnce(crate::primitives::style::UiStyle) -> crate::primitives::style::UiStyle,
    {
        self.with_state_style(crate::primitives::style::StyleState::Focused, style)
    }
    pub fn pressed<F>(self, style: F) -> Self
    where
        F: FnOnce(crate::primitives::style::UiStyle) -> crate::primitives::style::UiStyle,
    {
        self.with_state_style(crate::primitives::style::StyleState::Pressed, style)
    }
    pub fn active<F>(self, style: F) -> Self
    where
        F: FnOnce(crate::primitives::style::UiStyle) -> crate::primitives::style::UiStyle,
    {
        self.with_state_style(crate::primitives::style::StyleState::Active, style)
    }
    pub fn inactive<F>(self, style: F) -> Self
    where
        F: FnOnce(crate::primitives::style::UiStyle) -> crate::primitives::style::UiStyle,
    {
        self.with_state_style(crate::primitives::style::StyleState::Inactive, style)
    }
    pub fn disabled_style<F>(self, style: F) -> Self
    where
        F: FnOnce(crate::primitives::style::UiStyle) -> crate::primitives::style::UiStyle,
    {
        self.with_state_style(crate::primitives::style::StyleState::Disabled, style)
    }
    pub fn submitting_style<F>(self, style: F) -> Self
    where
        F: FnOnce(crate::primitives::style::UiStyle) -> crate::primitives::style::UiStyle,
    {
        self.with_state_style(crate::primitives::style::StyleState::Submitting, style)
    }
    pub fn success_style<F>(self, style: F) -> Self
    where
        F: FnOnce(crate::primitives::style::UiStyle) -> crate::primitives::style::UiStyle,
    {
        self.with_state_style(crate::primitives::style::StyleState::Success, style)
    }

    pub fn padding(self, value: impl crate::primitives::style::IntoStyleValue) -> Self {
        self.styled(|style| style.padding = Some(UiRect::all(value.into_style_value())))
    }
    pub fn margin(self, value: impl crate::primitives::style::IntoStyleValue) -> Self {
        self.styled(|style| style.margin = Some(UiRect::all(value.into_style_value())))
    }
    pub fn width(self, value: impl crate::primitives::style::IntoStyleValue) -> Self {
        self.styled(|style| style.width = Some(value.into_style_value()))
    }
    pub fn height(self, value: impl crate::primitives::style::IntoStyleValue) -> Self {
        self.styled(|style| style.height = Some(value.into_style_value()))
    }
    pub fn gap(self, value: impl crate::primitives::style::IntoStyleValue) -> Self {
        let value = value.into_style_value();
        self.styled(|style| {
            style.row_gap = Some(value);
            style.column_gap = Some(value);
        })
    }
    pub fn radius(self, value: impl crate::primitives::style::IntoStyleNumber) -> Self {
        let value = value.into_style_number();
        self.styled(|style| style.radius = Some(value.max(0.0)))
    }
    pub fn border_width(self, value: impl crate::primitives::style::IntoStyleNumber) -> Self {
        let value = value.into_style_number();
        self.styled(|style| style.border_width = Some(value.max(0.0)))
    }
    pub fn border_color(self, value: impl crate::primitives::style::IntoStyleColor) -> Self {
        let value = value.into_style_color();
        self.styled(|style| style.border_color = Some(value))
    }
    pub fn background(self, value: impl crate::primitives::style::IntoStylePaint) -> Self {
        self.styled(|style| style.background = Some(value.into_style_paint()))
    }
    pub fn color(self, value: impl crate::primitives::style::IntoStyleColor) -> Self {
        let value = value.into_style_color();
        self.styled(|style| style.color = Some(value))
    }

    pub fn min_width(self, value: impl crate::primitives::style::IntoStyleValue) -> Self {
        self.styled(|style| style.min_width = Some(value.into_style_value()))
    }
    pub fn max_width(self, value: impl crate::primitives::style::IntoStyleValue) -> Self {
        self.styled(|style| style.max_width = Some(value.into_style_value()))
    }
    pub fn min_height(self, value: impl crate::primitives::style::IntoStyleValue) -> Self {
        self.styled(|style| style.min_height = Some(value.into_style_value()))
    }
    pub fn max_height(self, value: impl crate::primitives::style::IntoStyleValue) -> Self {
        self.styled(|style| style.max_height = Some(value.into_style_value()))
    }
    pub fn padding_x(self, value: impl crate::primitives::style::IntoStyleValue) -> Self {
        let value = value.into_style_value();
        self.styled(|style| {
            let padding = style.padding.get_or_insert(UiRect::ZERO);
            padding.left = value;
            padding.right = value;
        })
    }
    pub fn padding_y(self, value: impl crate::primitives::style::IntoStyleValue) -> Self {
        let value = value.into_style_value();
        self.styled(|style| {
            let padding = style.padding.get_or_insert(UiRect::ZERO);
            padding.top = value;
            padding.bottom = value;
        })
    }
    pub fn padding_rect(self, value: UiRect) -> Self {
        self.styled(|style| style.padding = Some(value))
    }
    pub fn margin_rect(self, value: UiRect) -> Self {
        self.styled(|style| style.margin = Some(value))
    }
    pub fn margin_x(self, value: impl crate::primitives::style::IntoStyleValue) -> Self {
        let value = value.into_style_value();
        self.styled(|style| {
            let margin = style.margin.get_or_insert(UiRect::ZERO);
            margin.left = value;
            margin.right = value;
        })
    }
    pub fn margin_y(self, value: impl crate::primitives::style::IntoStyleValue) -> Self {
        let value = value.into_style_value();
        self.styled(|style| {
            let margin = style.margin.get_or_insert(UiRect::ZERO);
            margin.top = value;
            margin.bottom = value;
        })
    }
    pub fn row_gap(self, value: impl crate::primitives::style::IntoStyleValue) -> Self {
        self.styled(|style| style.row_gap = Some(value.into_style_value()))
    }
    pub fn column_gap(self, value: impl crate::primitives::style::IntoStyleValue) -> Self {
        self.styled(|style| style.column_gap = Some(value.into_style_value()))
    }
    pub fn font_size(self, value: impl crate::primitives::style::IntoStyleNumber) -> Self {
        self.styled(|style| style.font_size = Some(value.into_style_number().max(0.0)))
    }
    pub fn align_items(self, value: AlignItems) -> Self {
        self.styled(|style| style.align_items = Some(value))
    }
    pub fn align_self(self, value: AlignSelf) -> Self {
        self.styled(|style| style.align_self = Some(value))
    }
    pub fn justify_content(self, value: JustifyContent) -> Self {
        self.styled(|style| style.justify_content = Some(value))
    }
    pub fn flex_grow(self, value: impl crate::primitives::style::IntoStyleNumber) -> Self {
        self.styled(|style| style.flex_grow = Some(value.into_style_number().max(0.0)))
    }
    pub fn flex_shrink(self, value: impl crate::primitives::style::IntoStyleNumber) -> Self {
        self.styled(|style| style.flex_shrink = Some(value.into_style_number().max(0.0)))
    }
    pub fn scale(self, value: impl crate::primitives::style::IntoStyleNumber) -> Self {
        self.scale_xy(Vec2::splat(value.into_style_number()))
    }
    pub fn scale_xy(self, value: Vec2) -> Self {
        self.styled(|style| style.scale = Some(value))
    }
    pub fn outline_width(self, value: impl crate::primitives::style::IntoStyleNumber) -> Self {
        self.styled(|style| style.outline_width = Some(value.into_style_number().max(0.0)))
    }
    pub fn outline_color(self, value: impl crate::primitives::style::IntoStyleColor) -> Self {
        self.styled(|style| style.outline_color = Some(value.into_style_color()))
    }
    pub fn opacity(self, value: impl crate::primitives::style::IntoStyleNumber) -> Self {
        let value = value.into_style_number();
        assert!(value.is_finite(), "opacity must be finite");
        self.styled(|style| style.opacity = Some(value.clamp(0.0, 1.0)))
    }

    pub fn custom(setup: UiElementSetup) -> Self {
        Self::Custom(setup)
    }

    pub fn custom_in_context(setup: UiContextSetup) -> Self {
        Self::ContextCustom(setup)
    }

    pub fn aria(self, description: impl Into<String>) -> Self {
        let description = description.into();
        match self {
            Self::Link(link) => Self::Link(link.aria(description)),
            element => Self::Aria {
                element: Box::new(element),
                description,
            },
        }
    }

    pub fn name(self, name: impl Into<String>) -> Self {
        Self::Named {
            element: Box::new(self),
            name: name.into(),
        }
    }

    pub fn to(mut self, path: impl Into<String>) -> Self {
        if let Self::Link(link) = &mut self {
            link.target_path = Some(path.into());
        }
        self
    }

    pub fn external(mut self) -> Self {
        if let Self::Link(link) = &mut self {
            link.external = true;
        }
        self
    }

    pub fn params<I, K, V>(mut self, params: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: ToString,
    {
        if let Self::Link(link) = &mut self {
            link.params = params
                .into_iter()
                .map(|(key, value)| (key.into(), value.to_string()))
                .collect();
        }
        self
    }

    /// Connects a supported value control to a resource-model field.
    ///
    /// Initialize the model before the UI updates. `BeverlyPlugin` includes
    /// `BindingPlugin`; custom apps can install it separately. Configure a
    /// concrete control builder before calling its `.bind(...)` method, which
    /// returns the same bound `UiElement` representation used here.
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use beverly::prelude::*;
    /// use beverly::components::input::TextInputConfig;
    ///
    /// #[model]
    /// struct Account {
    ///     #[setter = normalize_email]
    ///     email: String,
    /// }
    /// impl Account {
    ///     fn normalize_email(&mut self, value: String) -> Result<(), &'static str> {
    ///         let value = value.trim().to_lowercase();
    ///         if !value.contains('@') { return Err("Enter an email address"); }
    ///         self.email = value;
    ///         Ok(())
    ///     }
    /// }
    /// let mut app = App::new();
    /// app.add_plugins(BindingPlugin).insert_resource(Account {
    ///     email: "initial@example.com".to_owned(),
    /// });
    /// let parent = app.world_mut().spawn(Node::default()).id();
    /// let input = TextInputConfig::new("Email").label("Email address")
    ///     .bind(Account::email)
    ///     .spawn_in_context(&mut UiBuildContext { world: app.world_mut(), parent });
    /// app.update();
    /// assert_eq!(app.world().get::<TextInput>(input).unwrap().value, "initial@example.com");
    /// app.world_mut().get_mut::<TextInput>(input).unwrap().value = " USER@EXAMPLE.COM ".to_owned();
    /// app.update();
    /// assert_eq!(app.world().resource::<Account>().get_email(), "user@example.com");
    /// assert_eq!(app.world().get::<TextInput>(input).unwrap().value, "user@example.com");
    /// ```
    pub fn bind<Model, Value>(
        self,
        binding: crate::primitives::binding::FieldBinding<Model, Value>,
    ) -> Self
    where
        Model: Resource,
        Value: crate::primitives::binding::BindingValueType,
    {
        Self::Bound {
            element: Box::new(self),
            binding: binding.erase(),
        }
    }

    pub fn button() -> Self {
        Self::Button {
            value: BeverlyButton::new(ButtonColor::Primary, ""),
            children: Vec::new(),
        }
    }

    /// Builds error content below this control from its current validation messages.
    ///
    /// Install `BindingPlugin` (included by `BeverlyPlugin`). The callback runs
    /// only for nonempty, changed error messages. Recovery removes its content
    /// without calling it with an empty slice. Bound controls use setter errors;
    /// Forms and unbound controls use their `BindingErrorState` directly.
    /// Captures must be `Send + Sync + 'static`. Return one `UiElement`, an
    /// array, a vector, or another iterable of elements. Repeated `.error(...)`
    /// calls on the same error wrapper replace its callback.
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use beverly::prelude::*;
    /// use beverly::components::input::TextInputConfig;
    /// let mut app = App::new();
    /// app.add_plugins(BindingPlugin);
    /// let parent = app.world_mut().spawn(Node::default()).id();
    /// let input = TextInputConfig::new("Email").label("Email address")
    ///     .error(|errors| text(errors.join("\n")))
    ///     .spawn_in_context(&mut UiBuildContext { world: app.world_mut(), parent });
    /// app.world_mut().get_mut::<BindingErrorState>(input).unwrap()
    ///     .error(BindingErrors::new("Enter a valid email address"));
    /// app.update();
    /// assert!(app.world_mut().query::<&Text>().iter(app.world())
    ///     .any(|text| text.0 == "Enter a valid email address"));
    /// ```
    pub fn error<F, I>(self, callback: F) -> Self
    where
        F: Fn(&[String]) -> I + Send + Sync + 'static,
        I: IntoIterator,
        I::Item: Into<UiElement>,
    {
        let callback = Arc::new(move |errors: &[String]| {
            callback(errors).into_iter().map(Into::into).collect()
        });
        self.with_error(callback)
    }

    pub fn submitting<F, I>(self, callback: F) -> Self
    where
        F: Fn() -> I + Send + Sync + 'static,
        I: IntoIterator,
        I::Item: Into<UiElement>,
    {
        let callback = Arc::new(move || callback().into_iter().map(Into::into).collect());
        self.with_form_state(Some(callback), None)
    }

    pub fn success<F, I>(self, callback: F) -> Self
    where
        F: Fn() -> I + Send + Sync + 'static,
        I: IntoIterator,
        I::Item: Into<UiElement>,
    {
        let callback = Arc::new(move || callback().into_iter().map(Into::into).collect());
        self.with_form_state(None, Some(callback))
    }

    fn with_form_state(
        self,
        submitting: Option<crate::primitives::binding::FormStateCallback>,
        success: Option<crate::primitives::binding::FormStateCallback>,
    ) -> Self {
        match self {
            Self::WithFormState {
                element,
                submitting: old_submitting,
                success: old_success,
            } => Self::WithFormState {
                element,
                submitting: submitting.or(old_submitting),
                success: success.or(old_success),
            },
            Self::Aria {
                element,
                description,
            } => Self::Aria {
                element: Box::new(element.with_form_state(submitting, success)),
                description,
            },
            Self::Named { element, name } => Self::Named {
                element: Box::new(element.with_form_state(submitting, success)),
                name,
            },
            Self::Bound { element, binding } => Self::Bound {
                element: Box::new(element.with_form_state(submitting, success)),
                binding,
            },
            Self::WithError { element, callback } => Self::WithError {
                element: Box::new(element.with_form_state(submitting, success)),
                callback,
            },
            Self::Styled { element, style } => Self::Styled {
                element: Box::new(element.with_form_state(submitting, success)),
                style,
            },
            element => Self::WithFormState {
                element: Box::new(element),
                submitting,
                success,
            },
        }
    }

    fn with_error(self, callback: crate::primitives::binding::ErrorCallback) -> Self {
        match self {
            Self::WithError { element, .. } => Self::WithError { element, callback },
            Self::WithFormState {
                element,
                submitting,
                success,
            } => Self::WithFormState {
                element: Box::new(element.with_error(callback)),
                submitting,
                success,
            },
            Self::Styled { element, style } => Self::Styled {
                element: Box::new(element.with_error(callback)),
                style,
            },
            Self::Bound { element, binding } => Self::Bound {
                element: Box::new(element.with_error(callback)),
                binding,
            },
            Self::Aria {
                element,
                description,
            } => Self::Aria {
                element: Box::new(element.with_error(callback)),
                description,
            },
            Self::Named { element, name } => Self::Named {
                element: Box::new(element.with_error(callback)),
                name,
            },
            element => Self::WithError {
                element: Box::new(element),
                callback,
            },
        }
    }

    pub fn checkbox() -> Self {
        Self::Checkbox {
            config: CheckboxConfig::default(),
        }
    }

    pub fn input(placeholder: impl Into<String>) -> Self {
        Self::TextInput {
            config: TextInputConfig::new(placeholder),
        }
    }

    pub fn alert(value: Alert) -> Self {
        Self::Alert {
            value,
            header: Vec::new(),
            body: Vec::new(),
            footer: Vec::new(),
        }
    }

    pub fn toast(value: Toast) -> Self {
        Self::Toast {
            value,
            header: Vec::new(),
            body: Vec::new(),
            footer: Vec::new(),
        }
    }

    pub fn divider(value: Divider) -> Self {
        Self::Divider(value)
    }

    pub fn progress(value: ProgressBar) -> Self {
        Self::ProgressBar(value)
    }

    pub fn dropdown(value: DropdownConfig) -> Self {
        Self::Dropdown(value)
    }

    pub fn pagination(value: PaginationConfig) -> Self {
        Self::Pagination(value)
    }

    pub fn textarea(value: TextareaConfig) -> Self {
        Self::Textarea(value)
    }

    pub fn toggle(value: ToggleConfig) -> Self {
        Self::Toggle(value)
    }

    pub fn slider(value: Slider) -> Self {
        Self::Slider {
            value,
            style: SliderStyle::default(),
        }
    }

    pub fn avatar(value: AvatarConfig) -> Self {
        Self::Avatar(value)
    }

    pub fn photo(value: Photo) -> Self {
        Self::Photo(value)
    }

    pub fn image() -> Self {
        Self::Image(ImageConfig::default())
    }

    pub fn src(mut self, source: impl Into<ImageSource>) -> Self {
        match &mut self {
            Self::Image(config) => config.source = Some(source.into()),
            Self::Styled { element, .. }
            | Self::Aria { element, .. }
            | Self::Named { element, .. }
            | Self::Bound { element, .. }
            | Self::WithError { element, .. } => **element = element.clone().src(source),
            _ => panic!(".src(...) requires an image element"),
        }
        self
    }

    pub fn alt(mut self, alt: impl Into<String>) -> Self {
        match &mut self {
            Self::Image(config) => config.alt = Some(alt.into()),
            Self::Styled { element, .. }
            | Self::Aria { element, .. }
            | Self::Named { element, .. }
            | Self::Bound { element, .. }
            | Self::WithError { element, .. } => **element = element.clone().alt(alt),
            _ => panic!(".alt(...) requires an image element"),
        }
        self
    }

    pub fn decorative(mut self, decorative: bool) -> Self {
        match &mut self {
            Self::Image(config) => config.decorative = decorative,
            Self::Styled { element, .. }
            | Self::Aria { element, .. }
            | Self::Named { element, .. }
            | Self::Bound { element, .. }
            | Self::WithError { element, .. } => **element = element.clone().decorative(decorative),
            _ => panic!(".decorative(...) requires an image element"),
        }
        self
    }

    pub fn card() -> Self {
        Self::Card {
            style: CardStyle::default(),
            children: Vec::new(),
            header: Vec::new(),
            body: Vec::new(),
            footer: Vec::new(),
        }
    }

    pub fn list_item(title: impl Into<String>) -> Self {
        Self::ListItem {
            title: title.into(),
            subtitle: None,
            icon: None,
        }
    }

    pub fn subtitle(mut self, value: impl Into<String>) -> Self {
        if let Self::ListItem { subtitle, .. } = &mut self {
            *subtitle = Some(value.into());
        }
        self
    }

    /// Appends content to the top region of a sectioned container.
    pub fn header<I>(mut self, children: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<UiElement>,
    {
        match &mut self {
            Self::Container { header, .. }
            | Self::Alert { header, .. }
            | Self::Card { header, .. }
            | Self::Form { header, .. }
            | Self::Tabs { header, .. }
            | Self::Sidebar { header, .. }
            | Self::Toast { header, .. }
            | Self::Modal { header, .. } => {
                header.extend(children.into_iter().map(Into::into));
            }
            Self::Aria { element, .. }
            | Self::Named { element, .. }
            | Self::Bound { element, .. } => {
                **element = element.clone().header(children);
            }
            Self::WithError { element, .. } => **element = element.clone().header(children),
            Self::Styled { element, .. } => **element = element.clone().header(children),
            _ => {}
        }
        self
    }

    /// Appends content to the middle region of a sectioned container.
    pub fn body<I>(mut self, children: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<UiElement>,
    {
        match &mut self {
            Self::Container { body, .. }
            | Self::Alert { body, .. }
            | Self::Card { body, .. }
            | Self::Form { body, .. }
            | Self::Tabs { body, .. }
            | Self::Sidebar { body, .. }
            | Self::Toast { body, .. }
            | Self::Modal { body, .. } => {
                body.extend(children.into_iter().map(Into::into));
            }
            Self::Aria { element, .. }
            | Self::Named { element, .. }
            | Self::Bound { element, .. } => {
                **element = element.clone().body(children);
            }
            Self::WithError { element, .. } => **element = element.clone().body(children),
            Self::Styled { element, .. } => **element = element.clone().body(children),
            _ => {}
        }
        self
    }

    /// Appends content to the bottom region of a sectioned container.
    pub fn footer<I>(mut self, children: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<UiElement>,
    {
        match &mut self {
            Self::Container { footer, .. }
            | Self::Alert { footer, .. }
            | Self::Card { footer, .. }
            | Self::Form { footer, .. }
            | Self::Tabs { footer, .. }
            | Self::Sidebar { footer, .. }
            | Self::Toast { footer, .. }
            | Self::Modal { footer, .. } => {
                footer.extend(children.into_iter().map(Into::into));
            }
            Self::Aria { element, .. }
            | Self::Named { element, .. }
            | Self::Bound { element, .. } => {
                **element = element.clone().footer(children);
            }
            Self::WithError { element, .. } => **element = element.clone().footer(children),
            Self::Styled { element, .. } => **element = element.clone().footer(children),
            _ => {}
        }
        self
    }

    pub fn form(value: Form) -> Self {
        Self::Form {
            value,
            children: Vec::new(),
            header: Vec::new(),
            body: Vec::new(),
            footer: Vec::new(),
        }
    }

    pub fn radio(value: RadioGroupBuilder) -> Self {
        Self::Radio(value)
    }

    pub fn table(value: TableConfig) -> Self {
        Self::Table(value)
    }

    pub fn tabs(value: TabsConfig) -> Self {
        Self::Tabs {
            config: value,
            children: Vec::new(),
            header: Vec::new(),
            body: Vec::new(),
            footer: Vec::new(),
        }
    }

    pub fn modal(value: Modal, style: ModalStyle, content: BasicModalContent) -> Self {
        Self::Modal {
            value,
            style,
            content,
            header: Vec::new(),
            body: Vec::new(),
            footer: Vec::new(),
        }
    }

    pub fn search() -> Self {
        Self::Search
    }

    pub fn searchbox(placeholder: impl Into<String>) -> Self {
        Self::SearchBox {
            placeholder: placeholder.into(),
        }
    }

    pub fn file_input(value: FileInput) -> Self {
        Self::FileInput(value)
    }

    pub fn navbar(fixed: bool) -> Self {
        Self::Navbar { fixed }
    }

    pub fn footer_surface(fixed: bool, config: FooterConfig) -> Self {
        Self::Footer { fixed, config }
    }

    pub fn sidebar(top_offset_px: f32) -> Self {
        Self::Sidebar {
            top_offset_px,
            header: Vec::new(),
            body: Vec::new(),
            footer: Vec::new(),
        }
    }

    pub fn link(value: Link) -> Self {
        Self::Link(value)
    }
    pub fn tooltip(value: Tooltip) -> Self {
        Self::Tooltip(value)
    }
    pub fn spinner(value: Spinner) -> Self {
        Self::Spinner(value)
    }
    pub fn select(options: Vec<String>) -> Self {
        Self::Select(Select::new(options))
    }
    pub fn text(mut self, value: impl Into<String>) -> Self {
        match &mut self {
            Self::Text { value: text, .. } => *text = Text::new(value),
            Self::Button { value: button, .. } => {
                button.label = value.into();
                button.children.clear();
            }
            Self::Checkbox { .. }
            | Self::TextInput { .. }
            | Self::Alert { .. }
            | Self::Toast { .. }
            | Self::Divider(_)
            | Self::ProgressBar(_)
            | Self::Dropdown(_)
            | Self::Pagination(_)
            | Self::Textarea(_)
            | Self::Toggle(_)
            | Self::Slider { .. }
            | Self::Avatar(_)
            | Self::Photo(_)
            | Self::Image(_)
            | Self::Container { .. }
            | Self::Card { .. }
            | Self::ListItem { .. }
            | Self::Form { .. }
            | Self::Radio(_)
            | Self::Table(_)
            | Self::Tabs { .. }
            | Self::Modal { .. }
            | Self::Search
            | Self::SearchBox { .. }
            | Self::FileInput(_)
            | Self::Navbar { .. }
            | Self::Footer { .. }
            | Self::Sidebar { .. }
            | Self::Link(_)
            | Self::Tooltip(_)
            | Self::Spinner(_)
            | Self::Select(_)
            | Self::Icon(_)
            | Self::Custom(_)
            | Self::ContextCustom(_)
            | Self::TypedSelect(_)
            | Self::ButtonCustom(_)
            | Self::Aria { .. } => {}
            Self::Named { .. } => {}
            Self::Bound { element, .. }
            | Self::WithError { element, .. }
            | Self::Styled { element, .. }
            | Self::WithFormState { element, .. } => {
                let replacement = std::mem::replace(
                    element,
                    Box::new(Self::Text {
                        value: Text::default(),
                        children: Vec::new(),
                    }),
                );
                **element = replacement.text(value);
            }
        }
        self
    }

    pub fn icon(mut self, icon: &'static str) -> Self {
        match &mut self {
            Self::Button { value, .. } => value.icon = Some(icon),
            Self::Link(value) => value.icon = Some(icon.to_string()),
            Self::WithError { element, .. }
            | Self::Bound { element, .. }
            | Self::Styled { element, .. }
            | Self::WithFormState { element, .. } => **element = element.clone().icon(icon),
            _ => {}
        }
        self
    }
    pub fn on(
        mut self,
        event: &'static str,
        command: crate::components::button::ButtonCommand,
    ) -> Self {
        if let Self::WithError { element, .. }
        | Self::Bound { element, .. }
        | Self::Styled { element, .. }
        | Self::WithFormState { element, .. } = &mut self
        {
            **element = element.clone().on(event, command);
            return self;
        }
        if let Self::Button { value, .. } = &mut self {
            value.handlers.push((
                crate::components::button::ButtonEventType::parse(event)
                    .unwrap_or_else(|| panic!("unsupported button event type: {event}")),
                command,
            ));
            assert_eq!(
                event, "click",
                "BeverlyButton currently supports only the click event"
            );
        }
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        match &mut self {
            Self::Button { value, .. } => value.disabled = disabled,
            Self::Link(value) => value.disabled = disabled,
            Self::Checkbox { config } => config.disabled = disabled,
            Self::TextInput { config } => config.disabled = disabled,
            Self::Textarea(config) => config.disabled = disabled,
            Self::Toggle(config) => config.disabled = disabled,
            Self::Slider { value, .. } => value.disabled = disabled,
            Self::Select(value) => value.disabled = disabled,
            Self::Dropdown(config) => config.disabled = disabled,
            Self::Tabs { config, .. } => config.disabled = disabled,
            Self::Radio(config) => *config = config.clone().disabled(disabled),
            Self::FileInput(value) => value.disabled = disabled,
            Self::Aria { element, .. }
            | Self::Named { element, .. }
            | Self::Bound { element, .. } => {
                **element = element.clone().disabled(disabled);
            }
            Self::WithError { element, .. } => **element = element.clone().disabled(disabled),
            Self::WithFormState { element, .. } => **element = element.clone().disabled(disabled),
            Self::Styled { element, .. } => **element = element.clone().disabled(disabled),
            _ => {}
        }
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        let label = label.into();
        match &mut self {
            Self::Checkbox { config } => config.label = Some(label),
            Self::TextInput { config } => config.accessible_label = Some(label),
            Self::Textarea(config) => config.accessible_label = Some(label),
            Self::Toggle(config) => config.label = Some(label),
            Self::Select(value) => value.accessible_label = Some(label),
            Self::Aria { element, .. }
            | Self::Named { element, .. }
            | Self::Bound { element, .. } => {
                **element = element.clone().label(label);
            }
            Self::WithError { element, .. } => **element = element.clone().label(label),
            Self::WithFormState { element, .. } => **element = element.clone().label(label),
            Self::Styled { element, .. } => **element = element.clone().label(label),
            _ => {}
        }
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        match &mut self {
            Self::Checkbox { config } => config.checked = checked,
            Self::WithError { element, .. }
            | Self::Bound { element, .. }
            | Self::Styled { element, .. } => **element = element.clone().checked(checked),
            _ => {}
        }
        self
    }

    pub fn kind(mut self, kind: TextInputKind) -> Self {
        if let Self::WithError { element, .. }
        | Self::Bound { element, .. }
        | Self::Styled { element, .. } = &mut self
        {
            **element = element.clone().kind(kind);
            return self;
        }
        if let Self::TextInput { config } = &mut self {
            config.kind = kind;
        }
        self
    }

    pub fn invalid(mut self, invalid: bool) -> Self {
        if let Self::WithError { element, .. }
        | Self::Bound { element, .. }
        | Self::Styled { element, .. } = &mut self
        {
            **element = element.clone().invalid(invalid);
            return self;
        }
        if let Self::TextInput { config } = &mut self {
            config.invalid = invalid;
        }
        self
    }

    /// Adds child elements to this node.
    pub fn children<I>(mut self, children: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<UiElement>,
    {
        match &mut self {
            Self::Container {
                children: current, ..
            } => {
                current.extend(children.into_iter().map(Into::into));
            }
            Self::Text {
                children: current, ..
            } => {
                current.extend(children.into_iter().map(Into::into));
            }
            Self::Button {
                children: current, ..
            } => {
                current.extend(children.into_iter().map(Into::into));
            }
            Self::Card {
                children: current, ..
            }
            | Self::Form {
                children: current, ..
            }
            | Self::Tabs {
                children: current, ..
            } => {
                current.extend(children.into_iter().map(Into::into));
            }
            Self::Checkbox { .. }
            | Self::TextInput { .. }
            | Self::Alert { .. }
            | Self::Toast { .. }
            | Self::Divider(_)
            | Self::ProgressBar(_)
            | Self::Dropdown(_)
            | Self::Pagination(_)
            | Self::Textarea(_)
            | Self::Toggle(_)
            | Self::Slider { .. }
            | Self::Avatar(_)
            | Self::Photo(_)
            | Self::Image(_)
            | Self::ListItem { .. }
            | Self::Radio(_)
            | Self::Table(_)
            | Self::Modal { .. }
            | Self::Search
            | Self::SearchBox { .. }
            | Self::FileInput(_)
            | Self::Navbar { .. }
            | Self::Footer { .. }
            | Self::Sidebar { .. }
            | Self::Link(_)
            | Self::Tooltip(_)
            | Self::Spinner(_)
            | Self::Select(_)
            | Self::Icon(_)
            | Self::Custom(_)
            | Self::ContextCustom(_)
            | Self::TypedSelect(_)
            | Self::ButtonCustom(_)
            | Self::Aria { .. } => {}
            Self::Named { .. } => {}
            Self::Bound { .. } => {}
            Self::WithError { element, .. } => {
                **element = element.clone().children(children);
            }
            Self::WithFormState { element, .. } => {
                **element = element.clone().children(children);
            }
            Self::Styled { element, .. } => **element = element.clone().children(children),
        }
        self
    }

    /// Spawns this element and its descendants below a Bevy UI parent.
    pub fn spawn(self, parent: &mut ChildSpawnerCommands) -> Entity {
        match self {
            Self::Container {
                mut node,
                children,
                header,
                mut body,
                footer,
            } => {
                if header.is_empty() && body.is_empty() && footer.is_empty() {
                    let mut entity = parent.spawn(node);
                    entity.with_children(|parent| {
                        for child in children {
                            child.spawn(parent);
                        }
                    });
                    return entity.id();
                }

                node.flex_direction = FlexDirection::Column;
                body.splice(0..0, children);
                let mut entity = parent.spawn(node);
                entity.with_children(|parent| {
                    spawn_content_section(parent, ContentSection::Header, header, false);
                    spawn_content_section(parent, ContentSection::Body, body, true);
                    spawn_content_section(parent, ContentSection::Footer, footer, false);
                });
                entity.id()
            }
            Self::Text { value, children } => {
                let mut entity = parent.spawn(value);
                entity.with_children(|parent| {
                    for child in children {
                        child.spawn(parent);
                    }
                });
                entity.id()
            }
            Self::Button {
                mut value,
                children,
            } => {
                for child in children {
                    value.children.push(match child {
                        UiElement::Text { value: text, .. } => ButtonChild::Text(text.0),
                        UiElement::Icon(icon) => ButtonChild::Icon(icon),
                        UiElement::Custom(_) => continue,
                        UiElement::ContextCustom(_) => continue,
                        UiElement::ButtonCustom(setup) => ButtonChild::Custom(setup),
                        UiElement::Button { .. } => continue,
                        UiElement::Checkbox { .. }
                        | UiElement::TextInput { .. }
                        | UiElement::Alert { .. }
                        | UiElement::Toast { .. }
                        | UiElement::Divider(_)
                        | UiElement::ProgressBar(_) => continue,
                        UiElement::Dropdown(_)
                        | UiElement::Pagination(_)
                        | UiElement::Textarea(_)
                        | UiElement::Toggle(_)
                        | UiElement::Slider { .. } => continue,
                        UiElement::Avatar(_) | UiElement::Photo(_) | UiElement::Image(_) => {
                            continue;
                        }
                        UiElement::Card { .. } | UiElement::ListItem { .. } => continue,
                        UiElement::Form { .. } | UiElement::Radio(_) | UiElement::Table(_) => {
                            continue;
                        }
                        UiElement::Tabs { .. } => continue,
                        UiElement::Modal { .. } => continue,
                        UiElement::Search
                        | UiElement::SearchBox { .. }
                        | UiElement::FileInput(_) => continue,
                        UiElement::Navbar { .. }
                        | UiElement::Footer { .. }
                        | UiElement::Sidebar { .. } => continue,
                        UiElement::Link(_)
                        | UiElement::Tooltip(_)
                        | UiElement::Spinner(_)
                        | UiElement::Select(_)
                        | UiElement::Container { .. }
                        | UiElement::Aria { .. }
                        | UiElement::Named { .. }
                        | UiElement::Bound { .. } => continue,
                        UiElement::WithError { .. } => continue,
                        UiElement::TypedSelect(_) => continue,
                        UiElement::Styled { .. } | UiElement::WithFormState { .. } => continue,
                    });
                }
                parent.spawn(value).id()
            }
            Self::Checkbox { config } => spawn_checkbox(parent, config),
            Self::TextInput { config } => spawn_text_input(parent, config),
            Self::Alert {
                value,
                header,
                body,
                footer,
            } => parent
                .spawn((
                    value,
                    AlertSections {
                        header,
                        body,
                        footer,
                    },
                ))
                .id(),
            Self::Toast {
                value,
                header,
                body,
                footer,
            } => parent
                .spawn((
                    value,
                    ToastSections {
                        header,
                        body,
                        footer,
                    },
                ))
                .id(),
            Self::Divider(value) => parent.spawn(value.build()).id(),
            Self::ProgressBar(value) => spawn_progress_bar_into(parent, value),
            Self::Dropdown(config) => spawn_dropdown(parent, config),
            Self::Pagination(config) => spawn_pagination(parent, config),
            Self::Textarea(config) => spawn_textarea(parent, config),
            Self::Toggle(_) | Self::Slider { .. } => {
                panic!("resource-backed fluent element requires spawn_in_context")
            }
            Self::Avatar(_) | Self::Photo(_) => {
                panic!("asset-backed fluent element requires spawn_in_context")
            }
            Self::Image(config) => spawn_image(parent, config),
            Self::Card { .. } | Self::ListItem { .. } => {
                panic!("context-aware structural element requires spawn_in_context")
            }
            Self::Form { .. } | Self::Radio(_) | Self::Table(_) => {
                panic!("specialized element requires spawn_in_context")
            }
            Self::Tabs { .. } => panic!("tabs requires spawn_in_context"),
            Self::Modal { .. } => panic!("modal requires spawn_in_context"),
            Self::Search | Self::SearchBox { .. } | Self::FileInput(_) => {
                panic!("specialized surface requires spawn_in_context")
            }
            Self::Navbar { .. } | Self::Footer { .. } | Self::Sidebar { .. } => {
                panic!("shell surface requires spawn_in_context")
            }
            Self::Link(_) | Self::Tooltip(_) | Self::Spinner(_) | Self::Select(_) => {
                panic!("leaf surface requires spawn_in_context")
            }
            Self::Icon(icon) => parent
                .spawn((
                    crate::icons::IconNode::new(crate::icons::Icon::feather(icon)),
                    Node {
                        width: px(16.0),
                        height: px(16.0),
                        ..default()
                    },
                ))
                .id(),
            Self::Custom(setup) => setup(parent),
            Self::ContextCustom(_) => {
                panic!("context-aware custom element requires spawn_in_context")
            }
            Self::TypedSelect(_) => panic!("typed select requires spawn_in_context"),
            Self::Aria {
                element,
                description,
            } => {
                let entity = element.spawn(parent);
                parent
                    .commands()
                    .entity(entity)
                    .insert(AriaDescription(description));
                entity
            }
            Self::Named { element, name } => {
                let entity = element.spawn(parent);
                parent
                    .commands()
                    .entity(entity)
                    .insert(crate::components::form::FormFieldName(name));
                entity
            }
            Self::Bound { element, binding } => {
                let entity = element.spawn(parent);
                parent.commands().entity(entity).insert((
                    crate::primitives::binding::BoundField::new(binding),
                    crate::primitives::binding::BindingErrorState::default(),
                ));
                entity
            }
            Self::ButtonCustom(setup) => parent
                .spawn(Node::default())
                .with_children(|children| setup(children, Color::WHITE))
                .id(),
            Self::WithError { element, callback } => {
                let mut control = None;
                let host = parent
                    .spawn(Node {
                        width: percent(100),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(4),
                        ..default()
                    })
                    .with_children(|children| {
                        control = Some(element.spawn(children));
                    })
                    .id();
                let control = control.unwrap();
                parent
                    .commands()
                    .entity(host)
                    .insert(crate::primitives::binding::ErrorPresentationHost(control));
                parent.commands().entity(control).insert(
                    crate::primitives::binding::ErrorPresentation::new(callback, host),
                );
                parent
                    .commands()
                    .entity(control)
                    .insert_if_new(crate::primitives::binding::BindingErrorState::default());
                control
            }
            Self::WithFormState {
                element,
                submitting,
                success,
            } => {
                let mut control = None;
                let host = parent
                    .spawn(Node {
                        width: percent(100),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(4),
                        ..default()
                    })
                    .with_children(|children| control = Some(element.spawn(children)))
                    .id();
                let control = control.unwrap();
                parent
                    .commands()
                    .entity(control)
                    .insert(crate::primitives::binding::FormStatusState::default());
                parent.commands().entity(control).insert(
                    crate::primitives::binding::FormStatePresentation::new(
                        host, submitting, success,
                    ),
                );
                control
            }
            Self::Styled { element, style } => {
                let entity = element.spawn(parent);
                parent.commands().entity(entity).insert(style);
                entity
            }
        }
    }

    /// Materializes this element with access to Bevy resources and the current
    /// parent entity. This is the path used by `app().children(...)`.
    pub fn spawn_in_context(self, context: &mut UiBuildContext<'_>) -> Entity {
        match self {
            Self::Container {
                mut node,
                children,
                header,
                mut body,
                footer,
            } => {
                let sectioned = !header.is_empty() || !body.is_empty() || !footer.is_empty();
                if sectioned {
                    node.flex_direction = FlexDirection::Column;
                }
                let entity = context.world.spawn((node, ChildOf(context.parent))).id();
                let mut content = UiBuildContext {
                    world: context.world,
                    parent: entity,
                };
                if sectioned {
                    body.splice(0..0, children);
                    spawn_section_in_context(&mut content, ContentSection::Header, header, false);
                    spawn_section_in_context(&mut content, ContentSection::Body, body, true);
                    spawn_section_in_context(&mut content, ContentSection::Footer, footer, false);
                } else {
                    for child in children {
                        child.spawn_in_context(&mut content);
                    }
                }
                entity
            }
            Self::Text { value, children } => {
                let entity = context.world.spawn((value, ChildOf(context.parent))).id();
                for child in children {
                    child.spawn_in_context(&mut UiBuildContext {
                        world: context.world,
                        parent: entity,
                    });
                }
                entity
            }
            Self::Button {
                mut value,
                children,
            } => {
                for child in children {
                    value.children.push(match child {
                        UiElement::Text { value: text, .. } => ButtonChild::Text(text.0),
                        UiElement::Icon(icon) => ButtonChild::Icon(icon),
                        UiElement::ButtonCustom(setup) => ButtonChild::Custom(setup),
                        UiElement::ContextCustom(_) => continue,
                        _ => continue,
                    });
                }
                context.world.spawn((value, ChildOf(context.parent))).id()
            }
            Self::Checkbox { config } => {
                spawn_with_children(context.world, context.parent, |parent| {
                    spawn_checkbox(parent, config)
                })
            }
            Self::TextInput { config } => {
                spawn_with_children(context.world, context.parent, |parent| {
                    spawn_text_input(parent, config)
                })
            }
            Self::Alert {
                value,
                header,
                body,
                footer,
            } => context
                .world
                .spawn((
                    value,
                    AlertSections {
                        header,
                        body,
                        footer,
                    },
                    ChildOf(context.parent),
                ))
                .id(),
            Self::Toast {
                value,
                header,
                body,
                footer,
            } => context
                .world
                .spawn((
                    value,
                    ToastSections {
                        header,
                        body,
                        footer,
                    },
                    ChildOf(context.parent),
                ))
                .id(),
            Self::Divider(value) => context
                .world
                .spawn((value.build(), ChildOf(context.parent)))
                .id(),
            Self::ProgressBar(value) => {
                spawn_with_children(context.world, context.parent, |parent| {
                    spawn_progress_bar_into(parent, value)
                })
            }
            Self::Dropdown(config) => {
                spawn_with_children(context.world, context.parent, |parent| {
                    spawn_dropdown(parent, config)
                })
            }
            Self::Pagination(config) => {
                spawn_with_children(context.world, context.parent, |parent| {
                    spawn_pagination(parent, config)
                })
            }
            Self::Textarea(config) => {
                spawn_with_children(context.world, context.parent, |parent| {
                    spawn_textarea(parent, config)
                })
            }
            Self::Toggle(config) => {
                let theme = *context.world.resource::<ThemeResource>();
                context.world.resource_scope(
                    |world, mut materials: Mut<Assets<ToggleShadowMaterial>>| {
                        spawn_with_children(world, context.parent, |parent| {
                            spawn_toggle(parent, config, &theme, &mut materials)
                        })
                    },
                )
            }
            Self::Slider { value, style } => {
                let theme = *context.world.resource::<ThemeResource>();
                spawn_with_children(context.world, context.parent, |parent| {
                    spawn_slider(parent, value, style, &theme)
                })
            }
            Self::Avatar(config) => {
                let asset_server = context.world.resource::<AssetServer>().clone();
                spawn_with_children(context.world, context.parent, |parent| {
                    spawn_avatar_in(parent, &asset_server, config)
                })
            }
            Self::Photo(photo) => spawn_with_children(context.world, context.parent, |parent| {
                spawn_photo(parent, photo)
            }),
            Self::Image(config) => spawn_with_children(context.world, context.parent, |parent| {
                spawn_image(parent, config)
            }),
            Self::Card {
                style,
                children,
                header,
                body,
                footer,
            } => {
                let theme = *context.world.resource::<ThemeResource>();
                let sectioned = !header.is_empty() || !body.is_empty() || !footer.is_empty();
                let entity = spawn_with_children(context.world, context.parent, |parent| {
                    spawn_card(parent, style, &theme, |_| {})
                });
                let mut content = UiBuildContext {
                    world: context.world,
                    parent: entity,
                };
                if sectioned {
                    let mut body_children = children;
                    body_children.extend(body);
                    spawn_section_in_context(&mut content, CardHeader, header, false);
                    spawn_section_in_context(&mut content, CardBody, body_children, true);
                    spawn_section_in_context(&mut content, CardFooter, footer, false);
                } else {
                    for child in children {
                        child.spawn_in_context(&mut content);
                    }
                }
                entity
            }
            Self::ListItem {
                title,
                subtitle,
                icon,
            } => spawn_with_children(context.world, context.parent, |parent| {
                spawn_list_item(parent, title, subtitle, icon)
            }),
            Self::Form {
                value,
                children,
                header,
                mut body,
                footer,
            } => {
                let sectioned = !header.is_empty() || !body.is_empty() || !footer.is_empty();
                body.splice(0..0, children);
                let entity = spawn_with_children(context.world, context.parent, |parent| {
                    spawn_form(parent, value, |_| {})
                });
                let submit = context
                    .world
                    .get::<Children>(entity)
                    .and_then(|children| children.first())
                    .copied();
                let mut content = UiBuildContext {
                    world: context.world,
                    parent: entity,
                };
                if sectioned {
                    spawn_section_in_context(&mut content, ContentSection::Header, header, false);
                    spawn_section_in_context(&mut content, ContentSection::Body, body, true);
                    spawn_section_in_context(&mut content, ContentSection::Footer, footer, false);
                } else {
                    for child in body {
                        child.spawn_in_context(&mut content);
                    }
                }
                if let Some(submit) = submit {
                    if sectioned {
                        let footer =
                            context
                                .world
                                .get::<Children>(entity)
                                .unwrap()
                                .iter()
                                .find(|child| {
                                    context.world.get::<ContentSection>(*child)
                                        == Some(&ContentSection::Footer)
                                });
                        let footer = footer.unwrap_or_else(|| {
                            context
                                .world
                                .spawn((
                                    ContentSection::Footer,
                                    Node {
                                        width: percent(100),
                                        flex_direction: FlexDirection::Column,
                                        ..default()
                                    },
                                    ChildOf(entity),
                                ))
                                .id()
                        });
                        context.world.entity_mut(footer).add_child(submit);
                    } else {
                        context.world.entity_mut(entity).detach_children(&[submit]);
                        context.world.entity_mut(entity).add_child(submit);
                    }
                }
                entity
            }
            Self::Radio(builder) => spawn_with_children(context.world, context.parent, |parent| {
                builder.spawn(parent)
            }),
            Self::Table(config) => spawn_with_children(context.world, context.parent, |parent| {
                Table::spawn_into(parent, config)
            }),
            Self::Tabs {
                config,
                children,
                header,
                body,
                footer,
            } => {
                let sectioned = !header.is_empty() || !body.is_empty() || !footer.is_empty();
                let entity = spawn_with_children(context.world, context.parent, |parent| {
                    spawn_tabs_in(parent, config, |_, _, _| {})
                });
                let zones: Vec<_> = context
                    .world
                    .query::<(Entity, &crate::components::tabs::TabZone)>()
                    .iter(context.world)
                    .filter(|(_, zone)| zone.tabs_entity == entity)
                    .map(|(entity, _)| entity)
                    .collect();
                for zone in zones {
                    let mut content = UiBuildContext {
                        world: context.world,
                        parent: zone,
                    };
                    if sectioned {
                        let mut body_children = children.clone();
                        body_children.extend(body.clone());
                        spawn_section_in_context(
                            &mut content,
                            ContentSection::Header,
                            header.clone(),
                            false,
                        );
                        spawn_section_in_context(
                            &mut content,
                            ContentSection::Body,
                            body_children,
                            true,
                        );
                        spawn_section_in_context(
                            &mut content,
                            ContentSection::Footer,
                            footer.clone(),
                            false,
                        );
                    } else {
                        for child in &children {
                            child.clone().spawn_in_context(&mut content);
                        }
                    }
                }
                entity
            }
            Self::Modal {
                value,
                style,
                content,
                header,
                body,
                footer,
            } => {
                let theme = *context.world.resource::<ThemeResource>();
                let sectioned = !header.is_empty() || !body.is_empty() || !footer.is_empty();
                let entity = if sectioned {
                    spawn_modal_with_surface(
                        &mut context.world.commands(),
                        context.parent,
                        value,
                        style,
                        content.title,
                        |_| {},
                        &theme,
                    )
                } else {
                    spawn_modal(
                        &mut context.world.commands(),
                        context.parent,
                        value,
                        style,
                        content,
                        &theme,
                    )
                };
                context.world.flush();
                if sectioned {
                    let surface = context
                        .world
                        .get::<Children>(entity)
                        .unwrap()
                        .iter()
                        .find(|child| {
                            context
                                .world
                                .get::<crate::components::modal::component::ModalSurface>(*child)
                                .is_some()
                        })
                        .expect("modal surface must exist");
                    let mut content = UiBuildContext {
                        world: context.world,
                        parent: surface,
                    };
                    spawn_section_in_context(&mut content, ModalHeader, header, false);
                    spawn_section_in_context(&mut content, ModalBody, body, true);
                    spawn_section_in_context(&mut content, ModalFooter, footer, false);
                }
                entity
            }
            Self::Search => {
                let fonts = context.world.resource::<UiFonts>().clone();
                let theme = *context.world.resource::<ThemeResource>();
                spawn_with_children(context.world, context.parent, |parent| {
                    spawn_search(parent, fonts.text, &theme)
                })
            }
            Self::SearchBox { placeholder } => {
                let fonts = context.world.resource::<UiFonts>().clone();
                let theme = *context.world.resource::<ThemeResource>();
                spawn_with_children(context.world, context.parent, |parent| {
                    spawn_searchbox(parent, fonts.text, &theme, placeholder)
                })
            }
            Self::FileInput(value) => {
                let mut commands = context.world.commands();
                let entity = spawn_file_input(&mut commands, value);
                commands.entity(entity).insert(ChildOf(context.parent));
                context.world.flush();
                entity
            }
            Self::Navbar { fixed } => {
                let mut commands = context.world.commands();
                let entity = spawn_navbar(&mut commands, fixed);
                commands.entity(entity).insert(ChildOf(context.parent));
                context.world.flush();
                entity
            }
            Self::Footer { fixed, config } => {
                let mut commands = context.world.commands();
                let entity = spawn_footer(&mut commands, fixed, config);
                commands.entity(entity).insert(ChildOf(context.parent));
                context.world.flush();
                entity
            }
            Self::Sidebar {
                top_offset_px,
                header,
                body,
                footer,
            } => {
                let fonts = context.world.resource::<UiFonts>().clone();
                let entity = spawn_with_children(context.world, context.parent, |parent| {
                    spawn_sidebar(parent, &fonts, top_offset_px)
                });
                if !header.is_empty() || !body.is_empty() || !footer.is_empty() {
                    let navigation: Vec<_> = context
                        .world
                        .get::<Children>(entity)
                        .unwrap()
                        .iter()
                        .filter(|child| {
                            context
                                .world
                                .get::<crate::components::sidebar::component::SidebarDivider>(
                                    *child,
                                )
                                .is_none()
                        })
                        .collect();
                    let mut content = UiBuildContext {
                        world: context.world,
                        parent: entity,
                    };
                    spawn_section_in_context(&mut content, ContentSection::Header, header, false);
                    let body =
                        spawn_section_in_context(&mut content, ContentSection::Body, body, true)
                            .unwrap();
                    content
                        .world
                        .entity_mut(body)
                        .insert_children(0, &navigation);
                    spawn_section_in_context(&mut content, ContentSection::Footer, footer, false);
                }
                entity
            }
            Self::Link(value) => {
                let label = value.text.clone();
                context
                    .world
                    .spawn((
                        Button,
                        value,
                        Node {
                            padding: UiRect::all(px(8.0)),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                        Surface::rounded_rect_fill(4.0, Paint::solid(Color::NONE)),
                        ChildOf(context.parent),
                    ))
                    .with_children(|parent| {
                        parent.spawn((crate::components::link::LinkText, Text::new(label)));
                    })
                    .id()
            }
            Self::Tooltip(value) => context.world.spawn((value, ChildOf(context.parent))).id(),
            Self::Spinner(value) => context.world.spawn((value, ChildOf(context.parent))).id(),
            Self::Select(value) => {
                let fonts = context.world.resource::<UiFonts>().clone();
                spawn_with_children(context.world, context.parent, |parent| {
                    spawn_select(parent, value, fonts.text)
                })
            }
            Self::Icon(icon) => context
                .world
                .spawn((
                    crate::icons::IconNode::new(crate::icons::Icon::feather(icon)),
                    Node {
                        width: px(16.0),
                        height: px(16.0),
                        ..default()
                    },
                    ChildOf(context.parent),
                ))
                .id(),
            Self::Custom(setup) => spawn_with_children(context.world, context.parent, setup),
            Self::ContextCustom(setup) => setup(context),
            Self::TypedSelect(setup) => setup(context),
            Self::Aria {
                element,
                description,
            } => {
                let entity = element.spawn_in_context(context);
                context
                    .world
                    .entity_mut(entity)
                    .insert(AriaDescription(description));
                entity
            }
            Self::Named { element, name } => {
                let entity = element.spawn_in_context(context);
                context
                    .world
                    .entity_mut(entity)
                    .insert(crate::components::form::FormFieldName(name));
                entity
            }
            Self::Bound { element, binding } => {
                let entity = element.spawn_in_context(context);
                context.world.entity_mut(entity).insert((
                    crate::primitives::binding::BoundField::new(binding),
                    crate::primitives::binding::BindingErrorState::default(),
                ));
                entity
            }
            Self::ButtonCustom(setup) => {
                spawn_with_children(context.world, context.parent, |parent| {
                    let mut entity = parent.spawn(Node::default());
                    entity.with_children(|children| setup(children, Color::WHITE));
                    entity.id()
                })
            }
            Self::WithError { element, callback } => {
                let host = context
                    .world
                    .spawn((
                        Node {
                            width: percent(100),
                            flex_direction: FlexDirection::Column,
                            row_gap: px(4),
                            ..default()
                        },
                        ChildOf(context.parent),
                    ))
                    .id();
                let control = element.spawn_in_context(&mut UiBuildContext {
                    world: context.world,
                    parent: host,
                });
                let width = context
                    .world
                    .get::<Node>(control)
                    .map(|node| node.width)
                    .unwrap_or(Val::Auto);
                context.world.get_mut::<Node>(host).unwrap().width = width;
                context
                    .world
                    .entity_mut(host)
                    .insert(crate::primitives::binding::ErrorPresentationHost(control));
                context.world.entity_mut(control).insert(
                    crate::primitives::binding::ErrorPresentation::new(callback, host),
                );
                if context
                    .world
                    .get::<crate::primitives::binding::BindingErrorState>(control)
                    .is_none()
                {
                    context
                        .world
                        .entity_mut(control)
                        .insert(crate::primitives::binding::BindingErrorState::default());
                }
                control
            }
            Self::WithFormState {
                element,
                submitting,
                success,
            } => {
                let host = context
                    .world
                    .spawn((
                        Node {
                            width: percent(100),
                            flex_direction: FlexDirection::Column,
                            row_gap: px(4),
                            ..default()
                        },
                        ChildOf(context.parent),
                    ))
                    .id();
                let control = element.spawn_in_context(&mut UiBuildContext {
                    world: context.world,
                    parent: host,
                });
                context
                    .world
                    .entity_mut(control)
                    .insert(crate::primitives::binding::FormStatusState::default());
                context.world.entity_mut(control).insert(
                    crate::primitives::binding::FormStatePresentation::new(
                        host, submitting, success,
                    ),
                );
                control
            }
            Self::Styled { element, style } => {
                let entity = element.spawn_in_context(context);
                crate::primitives::style::apply_style(context.world, entity, &style);
                context.world.entity_mut(entity).insert(style);
                entity
            }
        }
    }
}

impl From<ButtonChild> for UiElement {
    fn from(child: ButtonChild) -> Self {
        match child {
            ButtonChild::Text(value) => Self::Text {
                value: Text::new(value),
                children: Vec::new(),
            },
            ButtonChild::Icon(icon) => Self::Icon(icon),
            ButtonChild::Custom(setup) => Self::ButtonCustom(setup),
        }
    }
}

impl From<Text> for UiElement {
    fn from(value: Text) -> Self {
        Self::Text {
            value,
            children: Vec::new(),
        }
    }
}

impl From<BeverlyButton> for UiElement {
    fn from(value: BeverlyButton) -> Self {
        Self::Button {
            value,
            children: Vec::new(),
        }
    }
}

impl From<ImageConfig> for UiElement {
    fn from(value: ImageConfig) -> Self {
        Self::Image(value)
    }
}

macro_rules! impl_form_control_binding {
    ($control:ty, $constructor:expr) => {
        impl From<$control> for UiElement {
            fn from(value: $control) -> Self {
                ($constructor)(value)
            }
        }

        impl $control {
            pub fn submitting<F, I>(self, callback: F) -> UiElement
            where
                F: Fn() -> I + Send + Sync + 'static,
                I: IntoIterator,
                I::Item: Into<UiElement>,
            {
                UiElement::from(self).submitting(callback)
            }

            pub fn success<F, I>(self, callback: F) -> UiElement
            where
                F: Fn() -> I + Send + Sync + 'static,
                I: IntoIterator,
                I::Item: Into<UiElement>,
            {
                UiElement::from(self).success(callback)
            }

            pub fn error<F, I>(self, callback: F) -> UiElement
            where
                F: Fn(&[String]) -> I + Send + Sync + 'static,
                I: IntoIterator,
                I::Item: Into<UiElement>,
            {
                UiElement::from(self).error(callback)
            }

            /// Binds this control to a resource-model field, preserving its configuration.
            pub fn bind<Model, Value>(
                self,
                binding: crate::primitives::binding::FieldBinding<Model, Value>,
            ) -> UiElement
            where
                Model: Resource,
                Value: crate::primitives::binding::BindingValueType,
            {
                UiElement::from(self).bind(binding)
            }
        }
    };
}

impl_form_control_binding!(TextInputConfig, |config| UiElement::TextInput { config });
impl_form_control_binding!(CheckboxConfig, |config| UiElement::Checkbox { config });
impl_form_control_binding!(TextareaConfig, UiElement::Textarea);
impl_form_control_binding!(ToggleConfig, UiElement::Toggle);
impl_form_control_binding!(RadioGroupBuilder, UiElement::Radio);
impl_form_control_binding!(Slider, UiElement::slider);
impl_form_control_binding!(DropdownConfig, UiElement::Dropdown);
impl_form_control_binding!(FileInput, UiElement::FileInput);

impl From<Select<String>> for UiElement {
    fn from(value: Select<String>) -> Self {
        Self::Select(value)
    }
}

impl<T> Select<T>
where
    T: crate::primitives::binding::BindingValueType + std::fmt::Display,
{
    pub fn bind<Model, Value>(
        self,
        binding: crate::primitives::binding::FieldBinding<Model, Value>,
    ) -> UiElement
    where
        Model: Resource,
        Value: crate::primitives::binding::BindingValueType,
    {
        if let Some(select) = (&self as &dyn std::any::Any).downcast_ref::<Select<String>>() {
            return UiElement::Select(select.clone()).bind(binding);
        }
        UiElement::TypedSelect(Arc::new(move |context| {
            let font = context.world.resource::<UiFonts>().text.clone();
            let entity = spawn_with_children(context.world, context.parent, |parent| {
                spawn_select(parent, self.clone(), font)
            });
            context
                .world
                .entity_mut(entity)
                .insert(crate::primitives::binding::SelectBindingAdapter::new::<T>());
            entity
        }))
        .bind(binding)
    }
}

impl<T> Select<T>
where
    T: crate::components::select::SelectSemanticValue
        + std::fmt::Display
        + Clone
        + Send
        + Sync
        + 'static,
{
    pub fn submitting<F, I>(self, callback: F) -> UiElement
    where
        F: Fn() -> I + Send + Sync + 'static,
        I: IntoIterator,
        I::Item: Into<UiElement>,
    {
        UiElement::TypedSelect(Arc::new(move |context| {
            let font = context.world.resource::<UiFonts>().text.clone();
            spawn_with_children(context.world, context.parent, |parent| {
                spawn_select(parent, self.clone(), font)
            })
        }))
        .submitting(callback)
    }

    pub fn success<F, I>(self, callback: F) -> UiElement
    where
        F: Fn() -> I + Send + Sync + 'static,
        I: IntoIterator,
        I::Item: Into<UiElement>,
    {
        UiElement::TypedSelect(Arc::new(move |context| {
            let font = context.world.resource::<UiFonts>().text.clone();
            spawn_with_children(context.world, context.parent, |parent| {
                spawn_select(parent, self.clone(), font)
            })
        }))
        .success(callback)
    }

    pub fn error<F, I>(self, callback: F) -> UiElement
    where
        F: Fn(&[String]) -> I + Send + Sync + 'static,
        I: IntoIterator,
        I::Item: Into<UiElement>,
    {
        UiElement::TypedSelect(Arc::new(move |context| {
            let font = context.world.resource::<UiFonts>().text.clone();
            spawn_with_children(context.world, context.parent, |parent| {
                spawn_select(parent, self.clone(), font)
            })
        }))
        .error(callback)
    }
}

impl Form {
    pub fn submitting<F, I>(self, callback: F) -> UiElement
    where
        F: Fn() -> I + Send + Sync + 'static,
        I: IntoIterator,
        I::Item: Into<UiElement>,
    {
        UiElement::form(self).submitting(callback)
    }

    pub fn success<F, I>(self, callback: F) -> UiElement
    where
        F: Fn() -> I + Send + Sync + 'static,
        I: IntoIterator,
        I::Item: Into<UiElement>,
    {
        UiElement::form(self).success(callback)
    }

    pub fn error<F, I>(self, callback: F) -> UiElement
    where
        F: Fn(&[String]) -> I + Send + Sync + 'static,
        I: IntoIterator,
        I::Item: Into<UiElement>,
    {
        UiElement::form(self).error(callback)
    }
}

impl BeverlyButton {
    pub fn submitting<F, I>(self, callback: F) -> UiElement
    where
        F: Fn() -> I + Send + Sync + 'static,
        I: IntoIterator,
        I::Item: Into<UiElement>,
    {
        UiElement::from(self).submitting(callback)
    }

    pub fn on_success<F, I>(self, callback: F) -> UiElement
    where
        F: Fn() -> I + Send + Sync + 'static,
        I: IntoIterator,
        I::Item: Into<UiElement>,
    {
        UiElement::from(self).success(callback)
    }

    pub fn error<F, I>(self, callback: F) -> UiElement
    where
        F: Fn(&[String]) -> I + Send + Sync + 'static,
        I: IntoIterator,
        I::Item: Into<UiElement>,
    {
        UiElement::from(self).error(callback)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn submitting_and_success_presentations_follow_app_owned_status() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let submitting_calls = Arc::new(AtomicUsize::new(0));
        let success_calls = Arc::new(AtomicUsize::new(0));
        let submitting_counter = submitting_calls.clone();
        let success_counter = success_calls.clone();
        let mut app = App::new();
        app.add_plugins(crate::primitives::binding::BindingPlugin);
        let parent = app.world_mut().spawn(Node::default()).id();
        let control = TextInputConfig::new("Email")
            .label("Email")
            .submitting(move || {
                submitting_counter.fetch_add(1, Ordering::SeqCst);
                [crate::components::text::text("Checking…")]
            })
            .success(move || {
                success_counter.fetch_add(1, Ordering::SeqCst);
                [crate::components::text::text("Verified")]
            })
            .spawn_in_context(&mut UiBuildContext {
                world: app.world_mut(),
                parent,
            });
        let host = app.world().get::<ChildOf>(control).unwrap().parent();
        app.update();
        assert_eq!(app.world().get::<Children>(host).unwrap().len(), 1);
        assert_eq!(submitting_calls.load(Ordering::SeqCst), 0);
        app.world_mut()
            .get_mut::<crate::primitives::binding::FormStatusState>(control)
            .unwrap()
            .submitting();
        app.update();
        let pending = app.world().get::<Children>(host).unwrap()[1];
        assert_eq!(section_text(app.world(), pending), ["Checking…"]);
        let live = app
            .world()
            .get::<crate::primitives::semantic::SemanticNode>(pending)
            .unwrap();
        assert_eq!(live.role, crate::primitives::semantic::SemanticRole::Status);
        assert_eq!(
            live.live,
            Some(crate::primitives::semantic::AnnouncementPriority::Polite)
        );
        assert!(live.state.busy);
        assert_eq!(submitting_calls.load(Ordering::SeqCst), 1);
        app.update();
        assert_eq!(submitting_calls.load(Ordering::SeqCst), 1);
        app.world_mut()
            .get_mut::<crate::primitives::binding::FormStatusState>(control)
            .unwrap()
            .success();
        app.update();
        assert!(app.world().get_entity(pending).is_err());
        let success = app.world().get::<Children>(host).unwrap()[1];
        assert_eq!(section_text(app.world(), success), ["Verified"]);
        assert_eq!(success_calls.load(Ordering::SeqCst), 1);
        app.world_mut()
            .get_mut::<crate::primitives::binding::FormStatusState>(control)
            .unwrap()
            .idle();
        app.update();
        assert_eq!(app.world().get::<Children>(host).unwrap().len(), 1);
        assert!(app.world().get_entity(success).is_err());
    }

    #[test]
    fn success_and_submitting_methods_cover_forms_and_control_builders() {
        let callback = || crate::components::text::text("Working");
        let elements = vec![
            TextInputConfig::new("Email").submitting(callback),
            CheckboxConfig::new().success(callback),
            TextareaConfig::new("Notes").submitting(callback),
            ToggleConfig::new().success(callback),
            RadioGroupBuilder::new("plan").submitting(callback),
            Select::new(vec!["free".to_owned()]).success(callback),
            Select::new(vec![1_u32, 2]).submitting(callback),
            Slider::new(0.0, 1.0).success(callback),
            DropdownConfig::new("Plan").submitting(callback),
            FileInput::new("Files").success(callback),
            Form::new().submitting(callback),
            Form::new().success(callback),
            BeverlyButton::primary("Submit").submitting(callback),
            BeverlyButton::primary("Submit").on_success(callback),
        ];
        assert_eq!(elements.len(), 14);
        assert!(
            elements
                .iter()
                .all(|element| matches!(element, UiElement::WithFormState { .. }))
        );
    }

    #[test]
    fn status_callbacks_wrap_bound_field_without_dropping_binding() {
        let mut app = App::new();
        app.add_plugins(crate::primitives::binding::BindingPlugin)
            .insert_resource(BindingModel {
                name: "Initial".to_owned(),
                enabled: false,
                age: 1,
                rating: 0.0,
                files: Vec::new(),
            });
        let parent = app.world_mut().spawn(Node::default()).id();
        let control = TextInputConfig::new("Name")
            .label("Name")
            .bind(BindingModel::name)
            .submitting(|| crate::components::text::text("Saving"))
            .success(|| crate::components::text::text("Saved"))
            .spawn_in_context(&mut UiBuildContext {
                world: app.world_mut(),
                parent,
            });
        app.update();
        assert!(
            app.world()
                .get::<crate::primitives::binding::BoundField>(control)
                .is_some()
        );
        assert_eq!(
            app.world()
                .get::<crate::components::input::TextInput>(control)
                .unwrap()
                .value,
            "Initial"
        );
        app.world_mut()
            .get_mut::<crate::primitives::binding::FormStatusState>(control)
            .unwrap()
            .success();
        app.update();
        let host = app.world().get::<ChildOf>(control).unwrap().parent();
        let content = app.world().get::<Children>(host).unwrap()[1];
        assert_eq!(section_text(app.world(), content), ["Saved"]);
    }

    #[test]
    fn error_callback_supports_typed_select_without_string_conversion() {
        let mut app = App::new();
        app.add_plugins(crate::primitives::binding::BindingPlugin)
            .insert_resource(UiFonts {
                text: Handle::default(),
            });
        let parent = app.world_mut().spawn(Node::default()).id();
        let control = Select::new(vec![10_u32, 20_u32])
            .selected(1)
            .label("Size")
            .error(|errors| crate::components::text::text(errors.join("\n")))
            .spawn_in_context(&mut UiBuildContext {
                world: app.world_mut(),
                parent,
            });
        assert_eq!(
            app.world()
                .get::<Select<u32>>(control)
                .unwrap()
                .selected_value(),
            Some(&20)
        );
        app.world_mut()
            .get_mut::<crate::primitives::binding::BindingErrorState>(control)
            .unwrap()
            .messages = vec!["Unsupported size".to_owned()];
        app.update();
        let host = app.world().get::<ChildOf>(control).unwrap().parent();
        let content = app.world().get::<Children>(host).unwrap()[1];
        assert_eq!(section_text(app.world(), content), ["Unsupported size"]);
    }

    #[test]
    fn error_callback_removal_restores_focus_from_error_content() {
        let mut app = App::new();
        app.add_plugins(crate::primitives::binding::BindingPlugin)
            .add_message::<crate::primitives::focus::FocusRequest>();
        let parent = app.world_mut().spawn(Node::default()).id();
        let control = UiElement::checkbox()
            .error(|_| UiElement::button().text("Retry"))
            .checked(true)
            .spawn_in_context(&mut UiBuildContext {
                world: app.world_mut(),
                parent,
            });
        assert!(
            app.world()
                .get::<crate::components::checkbox::CheckboxState>(control)
                .unwrap()
                .checked
        );
        app.world_mut()
            .get_mut::<crate::primitives::binding::BindingErrorState>(control)
            .unwrap()
            .messages = vec!["Problem".to_owned()];
        app.update();
        let host = app.world().get::<ChildOf>(control).unwrap().parent();
        let content = app.world().get::<Children>(host).unwrap()[1];
        let action = app.world().get::<Children>(content).unwrap()[0];
        app.insert_resource(bevy::input_focus::InputFocus::from_entity(action));
        app.world_mut()
            .get_mut::<crate::primitives::binding::BindingErrorState>(control)
            .unwrap()
            .clear();
        app.update();
        let requests: Vec<_> = app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<crate::primitives::focus::FocusRequest>>()
            .drain()
            .collect();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].target, control);
        assert!(app.world().get_entity(action).is_err());
    }

    #[test]
    fn error_callback_replacement_across_binding_keeps_one_host() {
        let mut app = App::new();
        app.add_plugins(crate::primitives::binding::BindingPlugin)
            .insert_resource(ErrorCallbackModel {
                name: "Initial".to_owned(),
            });
        let parent = app.world_mut().spawn(Node::default()).id();
        let input = UiElement::input("Name")
            .label("Name")
            .error(|_| crate::components::text::text("Old"))
            .bind(ErrorCallbackModel::name)
            .error(|errors| crate::components::text::text(format!("Latest: {}", errors.join("; "))))
            .spawn_in_context(&mut UiBuildContext {
                world: app.world_mut(),
                parent,
            });
        app.update();
        app.world_mut()
            .get_mut::<crate::components::input::TextInput>(input)
            .unwrap()
            .value
            .clear();
        app.update();
        let host = app.world().get::<ChildOf>(input).unwrap().parent();
        assert_eq!(app.world().get::<ChildOf>(host).unwrap().parent(), parent);
        let content = app.world().get::<Children>(host).unwrap()[1];
        assert_eq!(
            section_text(app.world(), content),
            ["Latest: Name is required"]
        );
    }

    #[crate::model]
    struct ErrorCallbackModel {
        #[setter = validate_name]
        name: String,
    }

    impl ErrorCallbackModel {
        fn validate_name(&mut self, value: String) -> Result<(), &'static str> {
            if value.trim().is_empty() {
                return Err("Name is required");
            }
            self.name = value.trim().to_owned();
            Ok(())
        }
    }

    #[test]
    fn error_callback_bound_field_tracks_rejection_and_recovery() {
        let mut app = App::new();
        app.add_plugins(crate::primitives::binding::BindingPlugin)
            .insert_resource(ErrorCallbackModel {
                name: "Initial".to_owned(),
            });
        let parent = app.world_mut().spawn(Node::default()).id();
        let input = TextInputConfig::new("Name")
            .label("Name")
            .error(|errors| crate::components::text::text(errors.join("\n")))
            .bind(ErrorCallbackModel::name)
            .label("Full name")
            .spawn_in_context(&mut UiBuildContext {
                world: app.world_mut(),
                parent,
            });
        let host = app.world().get::<ChildOf>(input).unwrap().parent();
        app.update();
        app.world_mut()
            .get_mut::<crate::components::input::TextInput>(input)
            .unwrap()
            .value = " ".to_owned();
        app.update();
        assert_eq!(app.world().resource::<ErrorCallbackModel>().name, "Initial");
        assert_eq!(
            app.world()
                .get::<crate::components::input::TextInput>(input)
                .unwrap()
                .accessible_label
                .as_deref(),
            Some("Full name")
        );
        let error = app.world().get::<Children>(host).unwrap()[1];
        assert_eq!(section_text(app.world(), error), ["Name is required"]);
        app.world_mut()
            .get_mut::<crate::components::input::TextInput>(input)
            .unwrap()
            .value = "  Valid  ".to_owned();
        app.update();
        assert_eq!(app.world().resource::<ErrorCallbackModel>().name, "Valid");
        assert_eq!(app.world().get::<Children>(host).unwrap().len(), 1);
        assert!(app.world().get_entity(error).is_err());
    }

    #[test]
    fn error_callback_supports_form_errors_and_child_spawner_path() {
        let mut app = App::new();
        app.add_plugins(crate::primitives::binding::BindingPlugin);
        let parent = app.world_mut().spawn(Node::default()).id();
        let field = spawn_with_children(app.world_mut(), parent, |children| {
            CheckboxConfig::new()
                .label("Consent")
                .error(|errors| crate::components::text::text(errors.join("; ")))
                .spawn(children)
        });
        let form = Form::new()
            .error(|errors| [crate::components::text::text(errors.join("; "))])
            .body(crate::components::text::text("Form body"))
            .spawn_in_context(&mut UiBuildContext {
                world: app.world_mut(),
                parent,
            });
        for entity in [field, form] {
            app.world_mut()
                .get_mut::<crate::primitives::binding::BindingErrorState>(entity)
                .unwrap()
                .error(crate::primitives::binding::BindingErrors::new(
                    "Manual error",
                ));
        }
        app.update();
        for entity in [field, form] {
            let host = app.world().get::<ChildOf>(entity).unwrap().parent();
            let content = app.world().get::<Children>(host).unwrap()[1];
            assert_eq!(section_text(app.world(), content), ["Manual error"]);
        }
    }

    #[test]
    fn error_callback_rebuilds_on_changes_clears_on_recovery_and_cleans_host() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let calls = Arc::new(AtomicUsize::new(0));
        let counted = calls.clone();
        let mut app = App::new();
        app.add_plugins(crate::primitives::binding::BindingPlugin)
            .insert_resource(ThemeResource::default());
        let parent = app.world_mut().spawn(Node::default()).id();
        let control = CheckboxConfig::new()
            .label("Consent")
            .error(move |errors| {
                counted.fetch_add(1, Ordering::SeqCst);
                vec![
                    crate::components::text::text(errors.join("; ")),
                    UiElement::card(),
                ]
            })
            .spawn_in_context(&mut UiBuildContext {
                world: app.world_mut(),
                parent,
            });
        let host = app.world().get::<ChildOf>(control).unwrap().parent();
        app.update();
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        app.world_mut()
            .get_mut::<crate::primitives::binding::BindingErrorState>(control)
            .unwrap()
            .messages = vec!["Required".to_owned(), "Must accept".to_owned()];
        app.update();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let content = app.world().get::<Children>(host).unwrap()[1];
        assert_eq!(
            section_text(app.world(), content),
            ["Required; Must accept"]
        );
        app.update();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(app.world().get::<Children>(host).unwrap()[1], content);
        app.world_mut()
            .get_mut::<crate::primitives::binding::BindingErrorState>(control)
            .unwrap()
            .messages = vec!["Changed".to_owned()];
        app.update();
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert!(app.world().get_entity(content).is_err());
        app.world_mut()
            .get_mut::<crate::primitives::binding::BindingErrorState>(control)
            .unwrap()
            .clear();
        app.update();
        assert_eq!(app.world().get::<Children>(host).unwrap().len(), 1);
        app.world_mut().despawn(control);
        app.update();
        assert!(app.world().get_entity(host).is_err());
    }

    #[test]
    fn error_callbacks_support_every_value_builder_forms_and_submit_controls() {
        let callback = |errors: &[String]| crate::components::text::text(errors.join("\n"));
        let controls = vec![
            TextInputConfig::new("Name").error(callback),
            CheckboxConfig::new().error(callback),
            TextareaConfig::new("Notes").error(callback),
            ToggleConfig::new().error(callback),
            RadioGroupBuilder::new("plan").error(callback),
            Select::new(vec!["free".to_owned()]).error(callback),
            Slider::new(0.0, 100.0).error(callback),
            DropdownConfig::new("Plan").error(callback),
            FileInput::new("Files").error(callback),
            Form::new().error(callback),
            BeverlyButton::primary("Submit").error(callback),
            UiElement::search().error(callback),
            UiElement::searchbox("Search").error(callback),
        ];
        assert_eq!(controls.len(), 13);
        assert!(
            controls
                .iter()
                .all(|control| matches!(control, UiElement::WithError { .. }))
        );
    }

    #[test]
    fn binding_concrete_value_controls_sync_in_both_directions() {
        let mut world = World::new();
        world.insert_resource(ThemeResource::default());
        world.insert_resource(UiFonts {
            text: Handle::default(),
        });
        world.insert_resource(Assets::<ToggleShadowMaterial>::default());
        world.insert_resource(BindingModel {
            name: "free".to_owned(),
            enabled: true,
            age: 30,
            rating: 5.0,
            files: Vec::new(),
        });
        let parent = world.spawn(Node::default()).id();
        let controls = vec![
            TextareaConfig::new("Notes").bind(BindingModel::name),
            ToggleConfig::new().bind(BindingModel::enabled),
            RadioGroupBuilder::new("plan")
                .option("free", "Free")
                .option("pro", "Pro")
                .bind(BindingModel::name),
            Select::new(vec!["free".to_owned(), "pro".to_owned()]).bind(BindingModel::name),
            Slider::new(0.0, 100.0).bind(BindingModel::rating),
            DropdownConfig::new("Plan")
                .option("Free", "free")
                .option("Pro", "pro")
                .bind(BindingModel::name),
            FileInput::new("Files").bind(BindingModel::files),
        ];
        let entities: Vec<_> = controls
            .into_iter()
            .map(|control| {
                control.spawn_in_context(&mut UiBuildContext {
                    world: &mut world,
                    parent,
                })
            })
            .collect();
        crate::primitives::binding::sync_bound_fields(&mut world);
        assert_eq!(
            world
                .get::<crate::components::textarea::Textarea>(entities[0])
                .unwrap()
                .value,
            "free"
        );
        assert!(
            world
                .get::<crate::components::toggle::Toggle>(entities[1])
                .unwrap()
                .checked
        );
        assert_eq!(
            world
                .get::<crate::components::radio::RadioGroup>(entities[2])
                .unwrap()
                .selected,
            "free"
        );
        assert_eq!(
            world.get::<Select<String>>(entities[3]).unwrap().selected,
            Some(0)
        );
        assert_eq!(world.get::<Slider>(entities[4]).unwrap().value, 5.0);
        assert_eq!(
            world
                .get::<crate::components::dropdown::DropdownState>(entities[5])
                .unwrap()
                .selected,
            Some(0)
        );

        world
            .get_mut::<crate::components::textarea::Textarea>(entities[0])
            .unwrap()
            .set_value("pro");
        world
            .get_mut::<crate::components::toggle::Toggle>(entities[1])
            .unwrap()
            .checked = false;
        world
            .get_mut::<crate::components::radio::RadioGroup>(entities[2])
            .unwrap()
            .selected = "pro".to_owned();
        world
            .get_mut::<Select<String>>(entities[3])
            .unwrap()
            .selected = Some(1);
        world.get_mut::<Slider>(entities[4]).unwrap().value = 25.0;
        world
            .get_mut::<crate::components::dropdown::DropdownState>(entities[5])
            .unwrap()
            .selected = Some(1);
        let file = crate::components::file_input::SelectedFile {
            path: "report.txt".into(),
            file_type: None,
        };
        world
            .get_mut::<crate::components::file_input::FileInputSelectionState>(entities[6])
            .unwrap()
            .files = vec![file.clone()];
        crate::primitives::binding::sync_bound_fields(&mut world);
        assert_eq!(world.resource::<BindingModel>().name, "pro");
        assert!(!world.resource::<BindingModel>().enabled);
        assert_eq!(world.resource::<BindingModel>().rating, 25.0);
        assert_eq!(world.resource::<BindingModel>().files, [file]);

        let mut model = world.resource_mut::<BindingModel>();
        model.name = "free".to_owned();
        model.enabled = true;
        model.rating = 50.0;
        model.files.clear();
        crate::primitives::binding::sync_bound_fields(&mut world);
        assert_eq!(
            world
                .get::<crate::components::textarea::Textarea>(entities[0])
                .unwrap()
                .value,
            "free"
        );
        assert!(
            world
                .get::<crate::components::toggle::Toggle>(entities[1])
                .unwrap()
                .checked
        );
        assert_eq!(
            world
                .get::<crate::components::radio::RadioGroup>(entities[2])
                .unwrap()
                .selected,
            "free"
        );
        assert_eq!(
            world.get::<Select<String>>(entities[3]).unwrap().selected,
            Some(0)
        );
        assert_eq!(world.get::<Slider>(entities[4]).unwrap().value, 50.0);
        assert_eq!(
            world
                .get::<crate::components::dropdown::DropdownState>(entities[5])
                .unwrap()
                .selected,
            Some(0)
        );
        assert!(
            world
                .get::<crate::components::file_input::FileInputSelectionState>(entities[6])
                .unwrap()
                .files
                .is_empty()
        );
        for entity in entities {
            assert!(
                world
                    .get::<crate::primitives::binding::BindingErrorState>(entity)
                    .unwrap()
                    .messages
                    .is_empty()
            );
        }
    }

    #[test]
    fn binding_concrete_builders_preserve_configuration_and_sync_values() {
        let mut world = World::new();
        world.insert_resource(BindingModel {
            name: "Model name".to_owned(),
            enabled: true,
            age: 30,
            rating: 5.0,
            files: Vec::new(),
        });
        let parent = world.spawn(Node::default()).id();
        let input = TextInputConfig::new("Name")
            .label("Name")
            .bind(BindingModel::name)
            .spawn_in_context(&mut UiBuildContext {
                world: &mut world,
                parent,
            });
        let checkbox = CheckboxConfig::new()
            .label("Enabled")
            .bind(BindingModel::enabled)
            .spawn_in_context(&mut UiBuildContext {
                world: &mut world,
                parent,
            });
        crate::primitives::binding::sync_bound_fields(&mut world);
        let control = world
            .get::<crate::components::input::TextInput>(input)
            .unwrap();
        assert_eq!(control.value, "Model name");
        assert_eq!(control.placeholder, "Name");
        assert_eq!(control.accessible_label.as_deref(), Some("Name"));
        assert!(
            world
                .get::<crate::components::checkbox::CheckboxState>(checkbox)
                .unwrap()
                .checked
        );
        world
            .get_mut::<crate::components::input::TextInput>(input)
            .unwrap()
            .value = "Edited".to_owned();
        world
            .get_mut::<crate::components::checkbox::CheckboxState>(checkbox)
            .unwrap()
            .checked = false;
        crate::primitives::binding::sync_bound_fields(&mut world);
        assert_eq!(world.resource::<BindingModel>().name, "Edited");
        assert!(!world.resource::<BindingModel>().enabled);
        world.resource_mut::<BindingModel>().name = "External".to_owned();
        world.resource_mut::<BindingModel>().enabled = true;
        crate::primitives::binding::sync_bound_fields(&mut world);
        assert_eq!(
            world
                .get::<crate::components::input::TextInput>(input)
                .unwrap()
                .value,
            "External"
        );
        assert!(
            world
                .get::<crate::components::checkbox::CheckboxState>(checkbox)
                .unwrap()
                .checked
        );
    }

    #[test]
    fn sections_form_footer_submit_collects_body_fields() {
        let mut app = App::new();
        app.add_plugins(crate::components::form::FormPlugin);
        let parent = app.world_mut().spawn(Node::default()).id();
        let form = UiElement::form(Form::get("/search"))
            .header(crate::components::text::text("Search"))
            .body(UiElement::input("Query").label("Query").name("q"))
            .footer(crate::components::text::text("Submit the query"))
            .spawn_in_context(&mut UiBuildContext {
                world: app.world_mut(),
                parent,
            });
        app.world_mut()
            .query::<&mut crate::components::input::TextInput>()
            .single_mut(app.world_mut())
            .unwrap()
            .value = "beverly".to_owned();
        app.update();
        let button = app
            .world_mut()
            .query_filtered::<Entity, With<crate::components::form::FormSubmitButton>>()
            .single(app.world())
            .unwrap();
        app.world_mut()
            .entity_mut(button)
            .insert(Interaction::Pressed);
        app.update();
        let submissions: Vec<_> = app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<crate::components::form::FormSubmitted>>()
            .drain()
            .collect();
        assert_eq!(submissions.len(), 1);
        assert_eq!(submissions[0].form, form);
        assert_eq!(
            submissions[0].request.fields,
            [("q".to_owned(), "beverly".to_owned())]
        );
    }

    fn sectioned(element: UiElement) -> UiElement {
        element
            .footer(crate::components::text::text("bottom"))
            .children(crate::components::text::text("ordinary"))
            .body(crate::components::text::text("middle"))
            .header(crate::components::text::text("top"))
    }

    fn section_text(world: &World, entity: Entity) -> Vec<String> {
        world
            .get::<Children>(entity)
            .unwrap()
            .iter()
            .filter_map(|child| world.get::<Text>(child).map(|text| text.0.clone()))
            .collect()
    }

    #[test]
    fn sections_order_container_and_form_content_in_both_spawn_paths() {
        for contextual in [false, true] {
            let mut world = World::new();
            let parent = world.spawn(Node::default()).id();
            let element = sectioned(UiElement::row());
            let container = if contextual {
                element.spawn_in_context(&mut UiBuildContext {
                    world: &mut world,
                    parent,
                })
            } else {
                spawn_with_children(&mut world, parent, |children| element.spawn(children))
            };
            assert_eq!(
                world.get::<Node>(container).unwrap().flex_direction,
                FlexDirection::Column
            );
            let sections: Vec<_> = world.get::<Children>(container).unwrap().iter().collect();
            assert_eq!(sections.len(), 3);
            assert_eq!(section_text(&world, sections[0]), ["top"]);
            assert_eq!(section_text(&world, sections[1]), ["ordinary", "middle"]);
            assert_eq!(section_text(&world, sections[2]), ["bottom"]);
            assert_eq!(world.get::<Node>(sections[1]).unwrap().flex_grow, 1.0);
        }
        let mut world = World::new();
        let parent = world.spawn(Node::default()).id();
        let form = sectioned(UiElement::form(Form::new())).spawn_in_context(&mut UiBuildContext {
            world: &mut world,
            parent,
        });
        let sections: Vec<_> = world.get::<Children>(form).unwrap().iter().collect();
        assert_eq!(sections.len(), 3);
        assert_eq!(section_text(&world, sections[0]), ["top"]);
        assert_eq!(section_text(&world, sections[1]), ["ordinary", "middle"]);
        assert_eq!(section_text(&world, sections[2]), ["bottom"]);
        assert!(
            world
                .get::<Children>(sections[2])
                .unwrap()
                .iter()
                .any(|child| world
                    .get::<crate::components::form::FormSubmitButton>(child)
                    .is_some())
        );
    }

    #[test]
    fn sections_card_body_accepts_nested_cards_and_ordinary_children() {
        let mut world = World::new();
        world.insert_resource(ThemeResource::default());
        let parent = world.spawn(Node::default()).id();
        let card = sectioned(UiElement::card())
            .body(UiElement::card())
            .spawn_in_context(&mut UiBuildContext {
                world: &mut world,
                parent,
            });
        let sections: Vec<_> = world
            .get::<Children>(card)
            .unwrap()
            .iter()
            .filter(|child| {
                world.get::<CardHeader>(*child).is_some()
                    || world.get::<CardBody>(*child).is_some()
                    || world.get::<CardFooter>(*child).is_some()
            })
            .collect();
        assert_eq!(sections.len(), 3);
        assert!(world.get::<CardHeader>(sections[0]).is_some());
        assert!(world.get::<CardBody>(sections[1]).is_some());
        assert!(world.get::<CardFooter>(sections[2]).is_some());
        assert_eq!(section_text(&world, sections[1]), ["ordinary", "middle"]);
        assert!(
            world
                .get::<Children>(sections[1])
                .unwrap()
                .iter()
                .any(|child| world.get::<crate::components::card::Card>(child).is_some())
        );
    }

    #[test]
    fn sections_preserve_unsectioned_row_layout() {
        let mut world = World::new();
        let parent = world.spawn(Node::default()).id();
        let row = UiElement::row()
            .children(crate::components::text::text("direct"))
            .spawn_in_context(&mut UiBuildContext {
                world: &mut world,
                parent,
            });
        assert_eq!(
            world.get::<Node>(row).unwrap().flex_direction,
            FlexDirection::Row
        );
        assert_eq!(section_text(&world, row), ["direct"]);
    }

    #[test]
    fn sections_tabs_and_sidebar_keep_content_inside_their_panels() {
        let mut world = World::new();
        world.insert_resource(UiFonts {
            text: Handle::default(),
        });
        let parent = world.spawn(Node::default()).id();
        let tabs = sectioned(UiElement::tabs(TabsConfig::new(vec![
            crate::components::tabs::Tab::new("first", "First"),
            crate::components::tabs::Tab::new("second", "Second"),
        ])))
        .spawn_in_context(&mut UiBuildContext {
            world: &mut world,
            parent,
        });
        let zones: Vec<_> = world
            .query::<(Entity, &crate::components::tabs::TabZone)>()
            .iter(&world)
            .filter(|(_, zone)| zone.tabs_entity == tabs)
            .map(|(entity, _)| entity)
            .collect();
        assert_eq!(zones.len(), 2);
        for zone in zones {
            let parts = world.get::<Children>(zone).unwrap();
            assert_eq!(parts.len(), 3);
            assert_eq!(section_text(&world, parts[0]), ["top"]);
            assert_eq!(section_text(&world, parts[1]), ["ordinary", "middle"]);
            assert_eq!(section_text(&world, parts[2]), ["bottom"]);
        }
        let sidebar = UiElement::sidebar(0.0)
            .header(crate::components::text::text("top"))
            .body(crate::components::text::text("middle"))
            .footer(crate::components::text::text("bottom"))
            .spawn_in_context(&mut UiBuildContext {
                world: &mut world,
                parent,
            });
        let parts: Vec<_> = world
            .get::<Children>(sidebar)
            .unwrap()
            .iter()
            .filter(|child| world.get::<ContentSection>(*child).is_some())
            .collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(section_text(&world, parts[0]), ["top"]);
        assert_eq!(section_text(&world, parts[2]), ["bottom"]);
        assert!(world.get::<Children>(parts[1]).unwrap().len() > 1);
    }

    #[test]
    fn sections_modal_preserves_default_content_and_composes_nested_cards() {
        let mut world = World::new();
        world.insert_resource(ThemeResource::default());
        let parent = world.spawn(Node::default()).id();
        UiElement::modal(
            Modal::new(),
            ModalStyle::default(),
            BasicModalContent::new("Details", "Default body"),
        )
        .spawn_in_context(&mut UiBuildContext {
            world: &mut world,
            parent,
        });
        assert!(
            world
                .query::<&Text>()
                .iter(&world)
                .any(|text| text.0 == "Default body")
        );
        let modal = UiElement::modal(
            Modal::new(),
            ModalStyle::default(),
            BasicModalContent::new("Details", ""),
        )
        .aria("Dialog details")
        .footer(crate::components::text::text("bottom"))
        .body(UiElement::card())
        .header(crate::components::text::text("top"))
        .spawn_in_context(&mut UiBuildContext {
            world: &mut world,
            parent,
        });
        let surface = world
            .get::<Children>(modal)
            .unwrap()
            .iter()
            .find(|child| {
                world
                    .get::<crate::components::modal::component::ModalSurface>(*child)
                    .is_some()
            })
            .unwrap();
        let parts = world.get::<Children>(surface).unwrap();
        assert!(world.get::<ModalHeader>(parts[1]).is_some());
        assert!(world.get::<ModalBody>(parts[2]).is_some());
        assert!(world.get::<ModalFooter>(parts[3]).is_some());
        assert!(
            world
                .get::<Children>(parts[2])
                .unwrap()
                .iter()
                .any(|child| world.get::<crate::components::card::Card>(child).is_some())
        );
    }

    #[test]
    fn sections_alert_preserves_message_and_orders_custom_content() {
        let mut app = App::new();
        app.init_resource::<Time>()
            .insert_resource(ThemeResource::default())
            .add_plugins(crate::components::alert::AlertPlugin);
        let parent = app.world_mut().spawn(Node::default()).id();
        UiElement::alert(Alert::info("Built-in message"))
            .header(crate::components::text::text("top"))
            .body(crate::components::text::text("middle"))
            .body(UiElement::card())
            .footer(crate::components::text::text("bottom"))
            .spawn_in_context(&mut UiBuildContext {
                world: app.world_mut(),
                parent,
            });
        app.update();
        let mut query = app.world_mut().query::<(Entity, &Name)>();
        let sections: Vec<_> = query
            .iter(app.world())
            .filter(|(_, name)| name.as_str().starts_with("alert-"))
            .map(|(entity, name)| (entity, name.as_str().to_owned()))
            .collect();
        assert_eq!(sections.len(), 3);
        let content = app.world().get::<ChildOf>(sections[0].0).unwrap().parent();
        let parts = app.world().get::<Children>(content).unwrap();
        assert_eq!(
            app.world().get::<Name>(parts[0]).unwrap().as_str(),
            "alert-header"
        );
        assert_eq!(
            app.world().get::<Text>(parts[1]).unwrap().0,
            "Built-in message"
        );
        assert_eq!(
            app.world().get::<Name>(parts[2]).unwrap().as_str(),
            "alert-body"
        );
        assert_eq!(
            app.world().get::<Name>(parts[3]).unwrap().as_str(),
            "alert-footer"
        );
    }

    #[crate::model]
    struct BindingModel {
        name: String,
        enabled: bool,
        age: u32,
        rating: f32,
        files: Vec<crate::components::file_input::SelectedFile>,
    }

    #[test]
    fn text_and_button_are_composable_siblings() {
        let content = vec![
            crate::components::text::text("Hello"),
            crate::components::button::button().text("Continue"),
        ];

        assert_eq!(content.len(), 2);
        assert!(matches!(content[0], UiElement::Text { .. }));
        assert!(matches!(content[1], UiElement::Button { .. }));
    }

    #[test]
    fn children_are_recursive() {
        let element = crate::components::text::text("Parent")
            .children([crate::components::text::text("Child")]);

        assert!(matches!(element, UiElement::Text { children, .. } if children.len() == 1));
    }

    #[test]
    fn config_components_join_the_same_children_tree() {
        let content = vec![
            UiElement::input("Email")
                .label("Email address")
                .invalid(true),
            UiElement::checkbox().label("Accept terms").checked(true),
            UiElement::alert(Alert::info("Ready")),
        ];

        assert_eq!(content.len(), 3);
        assert!(matches!(content[0], UiElement::TextInput { .. }));
        assert!(matches!(content[1], UiElement::Checkbox { .. }));
        assert!(matches!(content[2], UiElement::Alert { .. }));
    }

    #[test]
    fn structural_components_are_composable() {
        let content = vec![
            UiElement::card().children([crate::components::text::text("Summary")]),
            UiElement::list_item("Recent activity").subtitle("Just now"),
        ];

        assert!(matches!(content[0], UiElement::Card { .. }));
        assert!(matches!(content[1], UiElement::ListItem { .. }));
    }

    #[test]
    fn specialized_components_are_composable() {
        let content = vec![
            UiElement::form(Form::post("/save")),
            UiElement::radio(RadioGroupBuilder::new("mode").option("a", "A")),
            UiElement::table(TableConfig::default()),
            UiElement::tabs(TabsConfig::new(vec![crate::components::tabs::Tab::new(
                "overview", "Overview",
            )])),
            UiElement::modal(
                Modal::new(),
                ModalStyle::default(),
                BasicModalContent::new("Details", "Content"),
            ),
        ];

        assert!(matches!(content[0], UiElement::Form { .. }));
        assert!(matches!(content[1], UiElement::Radio(_)));
        assert!(matches!(content[2], UiElement::Table(_)));
        assert!(matches!(content[3], UiElement::Tabs { .. }));
        assert!(matches!(content[4], UiElement::Modal { .. }));
    }

    #[test]
    fn bind_is_available_for_each_form_value_control() {
        let controls = vec![
            UiElement::input("Name").bind(BindingModel::name),
            UiElement::search().bind(BindingModel::name),
            UiElement::searchbox("Search").bind(BindingModel::name),
            UiElement::textarea(TextareaConfig::new("Notes")).bind(BindingModel::name),
            UiElement::checkbox().bind(BindingModel::enabled),
            UiElement::toggle(crate::components::toggle::ToggleConfig::new())
                .bind(BindingModel::enabled),
            UiElement::radio(RadioGroupBuilder::new("plan").option("free", "Free"))
                .bind(BindingModel::name),
            UiElement::select(vec!["free".to_owned(), "pro".to_owned()]).bind(BindingModel::name),
            Select::new(vec![1_u32, 2]).bind(BindingModel::age),
            UiElement::slider(Slider::new(0.0, 100.0)).bind(BindingModel::rating),
            UiElement::dropdown(DropdownConfig::new("Plan").option("Free", "free"))
                .bind(BindingModel::name),
            UiElement::file_input(crate::components::file_input::FileInput::new("Files"))
                .bind(BindingModel::files),
        ];

        assert!(
            controls
                .iter()
                .all(|control| matches!(control, UiElement::Bound { .. }))
        );
    }

    #[test]
    fn binding_is_available_on_each_concrete_value_builder() {
        let controls = vec![
            TextInputConfig::new("Name").bind(BindingModel::name),
            CheckboxConfig::new().bind(BindingModel::enabled),
            TextareaConfig::new("Notes").bind(BindingModel::name),
            ToggleConfig::new().bind(BindingModel::enabled),
            RadioGroupBuilder::new("plan")
                .option("free", "Free")
                .bind(BindingModel::name),
            Select::new(vec!["free".to_owned(), "pro".to_owned()]).bind(BindingModel::name),
            Select::new(vec![1_u32, 2]).bind(BindingModel::age),
            Slider::new(0.0, 100.0).bind(BindingModel::rating),
            DropdownConfig::new("Plan")
                .option("Free", "free")
                .bind(BindingModel::name),
            FileInput::new("Files").bind(BindingModel::files),
        ];
        assert_eq!(controls.len(), 10);
        assert!(
            controls
                .iter()
                .all(|control| matches!(control, UiElement::Bound { .. }))
        );
    }

    #[test]
    fn bound_input_spawn_initializes_from_the_model_resource() {
        let mut world = World::new();
        world.insert_resource(BindingModel {
            name: "Model value".to_owned(),
            enabled: false,
            age: 30,
            rating: 5.0,
            files: Vec::new(),
        });
        let parent = world.spawn_empty().id();
        let entity = UiElement::input("Name")
            .label("Name")
            .bind(BindingModel::name)
            .spawn_in_context(&mut UiBuildContext {
                world: &mut world,
                parent,
            });

        assert!(
            world
                .get::<crate::primitives::binding::BoundField>(entity)
                .is_some()
        );
        crate::primitives::binding::sync_bound_fields(&mut world);
        assert_eq!(
            world
                .get::<crate::components::input::TextInput>(entity)
                .unwrap()
                .value,
            "Model value",
        );
    }

    #[test]
    fn named_input_spawn_attaches_its_form_field_name() {
        let mut world = World::new();
        let parent = world.spawn_empty().id();
        let entity = UiElement::input("Search")
            .name("q")
            .spawn_in_context(&mut UiBuildContext {
                world: &mut world,
                parent,
            });

        assert_eq!(
            world
                .get::<crate::components::form::FormFieldName>(entity)
                .unwrap()
                .0,
            "q",
        );
    }
}
