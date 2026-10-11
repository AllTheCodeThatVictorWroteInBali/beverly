//! Commonly used Beverly items, meant to be glob-imported:
//!
//! ```
//! use beverly::prelude::*;
//! ```

pub use crate::BeverlyPlugin;
pub use crate::app::{BeverlyApp, BeverlyAppExt, Ui, app, ui};
pub use crate::controller;
pub use crate::event;
pub use crate::model;

// Theme.
pub use crate::theme::{ThemeMode, ThemePlugin, ThemeResource, dark_theme, light_theme};

// Icons.
pub use crate::icons::{FeatherIconsPlugin, Icon, IconNode};

// Rendering / styling primitives.
pub use crate::rendering::{
    Backdrop, BackdropQuality, Border, BorderWidths, GradientStop, InnerShadow, LinearGradient,
    LiquidGlass, Mask, OuterGlow, OuterShadow, Paint, Shimmer, SpinningGradient, Surface,
};

// Cross-cutting primitives.
pub use crate::primitives::a11y::A11yPlugin;
pub use crate::primitives::binding::{
    BindingErrorState, BindingErrors, BindingPlugin, BindingValue, BindingValueType, FieldBinding,
    SetterBinding,
};
pub use crate::primitives::clipboard::ClipboardPlugin;
pub use crate::primitives::composition::{
    UiBuildContext, UiContextSetup, UiElement, UiElementSetup, column, footer, row,
};
pub use crate::primitives::focus::FocusPlugin;
pub use crate::primitives::interaction::InteractionPlugin;
pub use crate::primitives::keyboard::KeyboardPlugin;
pub use crate::primitives::root::{AppRootSurface, ContentRoot, UiFonts};
pub use crate::primitives::routing::{
    Layout, Page, RouteContext, RouteState, RouterPlugin, layout, page, page_outlet,
};
pub use crate::primitives::semantic::{
    AnnouncementPriority, AriaDescription, SemanticNode, SemanticPlugin, SemanticRelationships,
    SemanticRole, SemanticSnapshotNode, SemanticState, SemanticTreeSnapshot, SemanticValue,
};

// Animation.
pub use crate::animation::animation::UiAnimationPlugin;
pub use crate::animation::blur::BackdropBlurPlugin;
pub use crate::animation::loading::{AppState, LoadingConfig, LoadingPlugin};
pub use crate::animation::motion::UiMotionPlugin;
pub use crate::animation::skeleton::{Skeleton, SkeletonPlugin};

// Components.
pub use crate::components::alert::{Alert, AlertPlugin, AlertVariant};
pub use crate::components::avatar::{Avatar, AvatarConfig, AvatarPlugin, AvatarSize, spawn_avatar_in};
pub use crate::components::badge::{Badge, BadgeSize, spawn_badge};
pub use crate::components::button::{
    BeverlyButton, ButtonChild, ButtonChildSetup, ButtonColor, ButtonCommand, ButtonEventType,
    ButtonPlugin, ButtonSize, button,
};
pub use crate::components::button_group::{ButtonGroup, ButtonGroupPlugin};
pub use crate::components::card::{
    Card, CardBody, CardFooter, CardHeader, CardPlugin, CardStyle, spawn_card,
};
pub use crate::components::checkbox::{Checkbox, CheckboxConfig, CheckboxPlugin, spawn_checkbox};
pub use crate::components::container::Container;
pub use crate::components::divider::{Divider, DividerPlugin};
pub use crate::components::dots::{Dots, DotsPlugin};
pub use crate::components::dropdown::{
    Dropdown, DropdownConfig, DropdownPlugin, spawn_dropdown,
};
pub use crate::components::file_input::{
    FileInput, FileInputCleared, FileInputPlugin, FileInputSelectionState, FileInputVariant, FileType, SelectedFile,
    spawn_file_input_in,
};
pub use crate::components::footer::{
    Footer, FooterConfig, FooterPlugin, FooterSections, add_to_center, add_to_left, add_to_right,
    spawn_footer_with_sections,
};
pub use crate::components::form::{
    Form, FormEncoding, FormFieldName, FormMethod, FormPlugin, FormRequest, FormSubmitted,
};
pub use crate::components::input::{
    TextInput, TextInputConfig, TextInputKind, TextInputPlugin, spawn_text_input,
};
pub use crate::components::link::{Link, LinkPlugin, link};
pub use crate::components::list_item::{ListItem, ListItemPlugin};
pub use crate::components::modal::{Modal, ModalPlugin};
pub use crate::components::nav_button::NavButton;
pub use crate::components::navbar::{Navbar, NavbarPlugin};
pub use crate::components::pagination::{Pagination, PaginationPlugin};
pub use crate::components::photo::{Photo, PhotoPlugin};
pub use crate::components::progress_bar::{ProgressBar, ProgressBarPlugin};
pub use crate::components::radio::{RadioButton, RadioGroup};
pub use crate::components::search::{Search, SearchPlugin};
pub use crate::components::select::{Select, SelectPlugin, SelectSemanticValue};
pub use crate::components::sidebar::{Sidebar, SidebarPlugin};
pub use crate::components::slider::{Slider, SliderPlugin};
pub use crate::components::spinner::{Spinner, SpinnerPlugin};
pub use crate::components::table::{Table, TablePlugin};
pub use crate::components::tabs::{Tab, Tabs, TabsPlugin};
pub use crate::components::theme_toggle::{
    ThemeTogglePlugin, ThemedPage, spawn_theme_toggle, spawn_themed_page, theme_from_cli_args,
    toggle_theme,
};
pub use crate::components::text::{TextRole, ThemedText, ThemedTextPlugin, text};
pub use crate::components::textarea::{Textarea, TextareaPlugin};
pub use crate::components::title::{ThemedTitle, TitleLevel, TitlePlugin};
pub use crate::components::toast::{Toast, ToastKind, ToastPlugin, ToastPosition};
pub use crate::components::toggle::{Toggle, TogglePlugin};
pub use crate::components::tooltip::{Tooltip, TooltipPlugin};
