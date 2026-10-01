use bevy::prelude::*;

use crate::components::button::{
    BeverlyButton, ButtonChild, ButtonChildSetup, ButtonColor,
};
use crate::components::alert::Alert;
use crate::components::checkbox::{spawn_checkbox, CheckboxConfig};
use crate::components::input::{spawn_text_input, TextInputConfig};
use crate::components::input::TextInputKind;
use crate::components::toast::{Toast, ToastSections};
use crate::components::divider::Divider;
use crate::components::progress_bar::{spawn_progress_bar_into, ProgressBar};
use crate::components::dropdown::{spawn_dropdown, DropdownConfig};
use crate::components::pagination::{spawn_pagination, PaginationConfig};
use crate::components::textarea::{spawn_textarea, TextareaConfig};
use crate::components::toggle::{spawn_toggle, ToggleConfig, ToggleShadowMaterial};
use crate::components::slider::{spawn_slider, Slider, SliderStyle};
use crate::components::avatar::{spawn_avatar_in, AvatarConfig};
use crate::components::photo::{spawn_photo, Photo};
use crate::components::card::{spawn_card, CardBody, CardFooter, CardHeader, CardStyle};
use crate::components::list_item::spawn_list_item;
use crate::components::form::{spawn_form, Form};
use crate::components::radio::RadioGroupBuilder;
use crate::components::table::{Table, TableConfig};
use crate::components::tabs::{spawn_tabs_in, TabsConfig};
use crate::components::modal::{
    spawn_modal_with_surface, BasicModalContent, Modal, ModalBody, ModalFooter,
    ModalHeader, ModalStyle,
};
use crate::components::search::spawn_search;
use crate::components::searchbox::spawn_searchbox;
use crate::components::file_input::{FileInput, FileInputDragState};
use crate::primitives::root::UiFonts;
use crate::components::navbar::spawn_navbar;
use crate::components::footer::{spawn_footer, FooterConfig};
use crate::components::sidebar::spawn_sidebar;
use crate::components::link::Link;
use crate::components::tooltip::Tooltip;
use crate::components::spinner::Spinner;
use crate::components::select::{spawn_select, Select};
use crate::icons::Icon;
use crate::theme::ThemeResource;
use crate::primitives::semantic::AriaDescription;
use crate::rendering::{Paint, Surface};

/// A composable UI node used by fluent application builders.
#[derive(Clone)]
pub enum UiElement {
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
    Slider { value: Slider, style: SliderStyle },
    Avatar(AvatarConfig),
    Photo(Photo),
    Card {
        style: CardStyle,
        children: Vec<UiElement>,
        header: Vec<UiElement>,
        body: Vec<UiElement>,
        footer: Vec<UiElement>,
    },
    ListItem { title: String, subtitle: Option<String>, icon: Option<Icon> },
    Form { value: Form, children: Vec<UiElement> },
    Radio(RadioGroupBuilder),
    Table(TableConfig),
    Tabs { config: TabsConfig, children: Vec<UiElement> },
    Modal {
        value: Modal,
        style: ModalStyle,
        content: BasicModalContent,
        header: Vec<UiElement>,
        body: Vec<UiElement>,
        footer: Vec<UiElement>,
    },
    Search,
    SearchBox { placeholder: String },
    FileInput(FileInput),
    Navbar { fixed: bool },
    Footer { fixed: bool, config: FooterConfig },
    Sidebar { top_offset_px: f32 },
    Link(Link),
    Tooltip(Tooltip),
    Spinner(Spinner),
    Select(Select<String>),
    Icon(&'static str),
    Custom(UiElementSetup),
    ButtonCustom(ButtonChildSetup),
    ContextCustom(UiContextSetup),
    Aria { element: Box<UiElement>, description: String },
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

/// Fluent constructor for the standalone Footer component.
///
/// This is distinct from the `.footer(...)` section helper on Card, Toast,
/// and other sectioned fluent elements.
pub fn footer(fixed: bool, config: FooterConfig) -> UiElement {
    UiElement::Footer { fixed, config }
}

/// World-backed context used while a fluent UI tree is being materialized.
pub struct UiBuildContext<'w> {
    pub world: &'w mut World,
    pub parent: Entity,
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

fn spawn_card_section<M: Component>(
    parent: &mut ChildSpawnerCommands,
    marker: M,
    children: Vec<UiElement>,
) {
    if children.is_empty() {
        return;
    }

    parent
        .spawn((
            marker,
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                ..default()
            },
        ))
        .with_children(|section| {
            for child in children {
                child.spawn(section);
            }
        });
}

fn spawn_modal_section<M: Component>(
    parent: &mut ChildSpawnerCommands,
    marker: M,
    children: Vec<UiElement>,
) {
    if children.is_empty() {
        return;
    }

    parent
        .spawn((
            marker,
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                ..default()
            },
        ))
        .with_children(|section| {
            for child in children {
                child.spawn(section);
            }
        });
}

impl UiElement {
    pub fn custom(setup: UiElementSetup) -> Self {
        Self::Custom(setup)
    }

    pub fn custom_in_context(setup: UiContextSetup) -> Self {
        Self::ContextCustom(setup)
    }

    pub fn aria(self, description: impl Into<String>) -> Self {
        Self::Aria {
            element: Box::new(self),
            description: description.into(),
        }
    }

    pub fn button() -> Self {
        Self::Button {
            value: BeverlyButton::new(ButtonColor::Primary, ""),
            children: Vec::new(),
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
        Self::Alert { value }
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
        Self::Slider { value, style: SliderStyle::default() }
    }

    pub fn avatar(value: AvatarConfig) -> Self {
        Self::Avatar(value)
    }

    pub fn photo(value: Photo) -> Self {
        Self::Photo(value)
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
        Self::ListItem { title: title.into(), subtitle: None, icon: None }
    }

    pub fn subtitle(mut self, value: impl Into<String>) -> Self {
        if let Self::ListItem { subtitle, .. } = &mut self {
            *subtitle = Some(value.into());
        }
        self
    }

    pub fn header<I>(mut self, children: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<UiElement>,
    {
        match &mut self {
            Self::Card { header, .. }
            | Self::Toast { header, .. }
            | Self::Modal { header, .. } => {
                header.extend(children.into_iter().map(Into::into));
            }
            _ => {}
        }
        self
    }

    pub fn body<I>(mut self, children: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<UiElement>,
    {
        match &mut self {
            Self::Card { body, .. }
            | Self::Toast { body, .. }
            | Self::Modal { body, .. } => {
                body.extend(children.into_iter().map(Into::into));
            }
            _ => {}
        }
        self
    }

    pub fn footer<I>(mut self, children: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<UiElement>,
    {
        match &mut self {
            Self::Card { footer, .. }
            | Self::Toast { footer, .. }
            | Self::Modal { footer, .. } => {
                footer.extend(children.into_iter().map(Into::into));
            }
            _ => {}
        }
        self
    }

    pub fn form(value: Form) -> Self {
        Self::Form { value, children: Vec::new() }
    }

    pub fn radio(value: RadioGroupBuilder) -> Self {
        Self::Radio(value)
    }

    pub fn table(value: TableConfig) -> Self {
        Self::Table(value)
    }

    pub fn tabs(value: TabsConfig) -> Self {
        Self::Tabs { config: value, children: Vec::new() }
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
        Self::SearchBox { placeholder: placeholder.into() }
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
        Self::Sidebar { top_offset_px }
    }

    pub fn link(value: Link) -> Self { Self::Link(value) }
    pub fn tooltip(value: Tooltip) -> Self { Self::Tooltip(value) }
    pub fn spinner(value: Spinner) -> Self { Self::Spinner(value) }
    pub fn select(options: Vec<String>) -> Self { Self::Select(Select::new(options)) }
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
            | Self::ButtonCustom(_)
            | Self::Aria { .. } => {}
        }
        self
    }

    pub fn icon(mut self, icon: &'static str) -> Self {
        if let Self::Button { value, .. } = &mut self {
            value.icon = Some(icon);
        }
        self
    }
    pub fn on(mut self, event: &'static str, command: crate::components::button::ButtonCommand) -> Self {
        if let Self::Button { value, .. } = &mut self {
            value.handlers.push((
                crate::components::button::ButtonEventType::parse(event)
                    .unwrap_or_else(|| panic!("unsupported button event type: {event}")),
                command,
            ));
            assert_eq!(event, "click", "BeverlyButton currently supports only the click event");
        }
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        match &mut self {
            Self::Button { value, .. } => value.disabled = disabled,
            Self::Checkbox { config } => config.disabled = disabled,
            Self::TextInput { config } => config.disabled = disabled,
            _ => {}
        }
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        let label = label.into();
        match &mut self {
            Self::Checkbox { config } => config.label = Some(label),
            Self::TextInput { config } => config.accessible_label = Some(label),
            _ => {}
        }
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        match &mut self {
            Self::Checkbox { config } => config.checked = checked,
            _ => {}
        }
        self
    }

    pub fn kind(mut self, kind: TextInputKind) -> Self {
        if let Self::TextInput { config } = &mut self {
            config.kind = kind;
        }
        self
    }

    pub fn invalid(mut self, invalid: bool) -> Self {
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
            Self::Text { children: current, .. } => {
                current.extend(children.into_iter().map(Into::into));
            }
            Self::Button { children: current, .. } => {
                current.extend(children.into_iter().map(Into::into));
            }
            Self::Card { children: current, .. }
            | Self::Form { children: current, .. }
            | Self::Tabs { children: current, .. } => {
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
            | Self::ButtonCustom(_)
            | Self::Aria { .. } => {}
        }
        self
    }

    /// Spawns this element and its descendants below a Bevy UI parent.
    pub fn spawn(self, parent: &mut ChildSpawnerCommands) -> Entity {
        match self {
            Self::Text { value, children } => {
                let mut entity = parent.spawn(value);
                entity.with_children(|parent| {
                    for child in children {
                        child.spawn(parent);
                    }
                });
                entity.id()
            }
            Self::Button { mut value, children } => {
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
                        UiElement::Avatar(_) | UiElement::Photo(_) => continue,
                        UiElement::Card { .. } | UiElement::ListItem { .. } => continue,
                        UiElement::Form { .. } | UiElement::Radio(_) | UiElement::Table(_) => continue,
                        UiElement::Tabs { .. } => continue,
                        UiElement::Modal { .. } => continue,
                        UiElement::Search | UiElement::SearchBox { .. } | UiElement::FileInput(_) => continue,
                        UiElement::Navbar { .. } | UiElement::Footer { .. } | UiElement::Sidebar { .. } => continue,
                        UiElement::Link(_)
                        | UiElement::Tooltip(_)
                        | UiElement::Spinner(_)
                        | UiElement::Select(_)
                        | UiElement::Aria { .. } => continue,
                    });
                }
                parent.spawn(value).id()
            }
            Self::Checkbox { config } => spawn_checkbox(parent, config),
            Self::TextInput { config } => spawn_text_input(parent, config),
            Self::Alert { value } => parent.spawn(value).id(),
            Self::Toast { value, .. } => parent.spawn(value).id(),
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
            Self::Icon(icon) => {
                parent.spawn((crate::icons::IconNode::new(crate::icons::Icon::feather(icon)),
                              Node { width: px(16.0), height: px(16.0), ..default() })).id()
            }
            Self::Custom(setup) => setup(parent),
            Self::ContextCustom(_) => {
                panic!("context-aware custom element requires spawn_in_context")
            }
            Self::Aria { element, description } => {
                let entity = element.spawn(parent);
                parent.commands().entity(entity).insert(AriaDescription(description));
                entity
            }
            Self::ButtonCustom(setup) => parent
                .spawn(Node::default())
                .with_children(|children| setup(children, Color::WHITE))
                .id(),
        }
    }

    /// Materializes this element with access to Bevy resources and the current
    /// parent entity. This is the path used by `app().children(...)`.
    pub fn spawn_in_context(self, context: &mut UiBuildContext<'_>) -> Entity {
        match self {
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
            Self::Button { mut value, children } => {
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
            Self::Alert { value } => context.world.spawn((value, ChildOf(context.parent))).id(),
            Self::Toast { value, header, body, footer } => context
                .world
                .spawn((
                    value,
                    ToastSections { header, body, footer },
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
            Self::Dropdown(config) => spawn_with_children(context.world, context.parent, |parent| {
                spawn_dropdown(parent, config)
            }),
            Self::Pagination(config) => spawn_with_children(context.world, context.parent, |parent| {
                spawn_pagination(parent, config)
            }),
            Self::Textarea(config) => spawn_with_children(context.world, context.parent, |parent| {
                spawn_textarea(parent, config)
            }),
            Self::Toggle(config) => {
                let theme = *context.world.resource::<ThemeResource>();
                context.world.resource_scope(|world, mut materials: Mut<Assets<ToggleShadowMaterial>>| {
                    spawn_with_children(world, context.parent, |parent| {
                        spawn_toggle(parent, config, &theme, &mut materials)
                    })
                })
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
            Self::Card { style, children, header, body, footer } => {
                let theme = *context.world.resource::<ThemeResource>();
                spawn_with_children(context.world, context.parent, |parent| {
                    spawn_card(parent, style, &theme, |content| {
                        for child in children {
                            child.spawn(content);
                        }
                        spawn_card_section(content, CardHeader, header);
                        spawn_card_section(content, CardBody, body);
                        spawn_card_section(content, CardFooter, footer);
                    })
                })
            }
            Self::ListItem { title, subtitle, icon } => {
                spawn_with_children(context.world, context.parent, |parent| {
                    spawn_list_item(parent, title, subtitle, icon)
                })
            }
            Self::Form { value, children } => {
                spawn_with_children(context.world, context.parent, |parent| {
                    spawn_form(parent, value, |content| {
                        for child in children {
                            child.spawn(content);
                        }
                    })
                })
            }
            Self::Radio(builder) => spawn_with_children(context.world, context.parent, |parent| {
                builder.spawn(parent)
            }),
            Self::Table(config) => spawn_with_children(context.world, context.parent, |parent| {
                Table::spawn_into(parent, config)
            }),
            Self::Tabs { config, children } => {
                spawn_with_children(context.world, context.parent, |parent| {
                    spawn_tabs_in(parent, config, |zone, _, _| {
                        for child in &children {
                            child.clone().spawn(zone);
                        }
                    })
                })
            }
            Self::Modal { value, style, content, header, body, footer } => {
                let theme = *context.world.resource::<ThemeResource>();
                let mut commands = context.world.commands();
                let title = content.title.clone();
                spawn_modal_with_surface(
                    &mut commands,
                    context.parent,
                    value,
                    style,
                    title,
                    move |surface| {
                        spawn_modal_section(surface, ModalHeader, header);
                        spawn_modal_section(surface, ModalBody, body);
                        spawn_modal_section(surface, ModalFooter, footer);
                    },
                    &theme,
                )
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
            Self::FileInput(value) => context
                .world
                .spawn((value, FileInputDragState::default(), ChildOf(context.parent)))
                .id(),
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
            Self::Sidebar { top_offset_px } => {
                let fonts = context.world.resource::<UiFonts>().clone();
                spawn_with_children(context.world, context.parent, |parent| {
                    spawn_sidebar(parent, &fonts, top_offset_px)
                })
            }
            Self::Link(value) => {
                let label = value.text.clone();
                context.world.spawn((
                    Button,
                    value,
                    Node { padding: UiRect::all(px(8.0)), ..default() },
                    BackgroundColor(Color::NONE),
                    Surface::rounded_rect_fill(4.0, Paint::solid(Color::NONE)),
                    ChildOf(context.parent),
                )).with_children(|parent| {
                    parent.spawn((crate::components::link::LinkText, Text::new(label)));
                }).id()
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
                    Node { width: px(16.0), height: px(16.0), ..default() },
                    ChildOf(context.parent),
                ))
                .id(),
            Self::Custom(setup) => spawn_with_children(context.world, context.parent, setup),
            Self::ContextCustom(setup) => setup(context),
            Self::Aria { element, description } => {
                let entity = element.spawn_in_context(context);
                context.world.entity_mut(entity).insert(AriaDescription(description));
                entity
            }
            Self::ButtonCustom(setup) => spawn_with_children(context.world, context.parent, |parent| {
                let mut entity = parent.spawn(Node::default());
                entity.with_children(|children| setup(children, Color::WHITE));
                entity.id()
            }),
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

#[cfg(test)]
mod tests {
    use super::*;

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
            UiElement::input("Email").label("Email address").invalid(true),
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
            UiElement::tabs(TabsConfig::new(vec![
                crate::components::tabs::Tab::new("overview", "Overview"),
            ])),
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
}
