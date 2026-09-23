//! Commonly used Beverly items, meant to be glob-imported:
//!
//! ```
//! use beverly::prelude::*;
//! ```

pub use crate::BeverlyPlugin;

// Theme.
pub use crate::theme::{ThemeMode, ThemePlugin, ThemeResource, dark_theme, light_theme};

// Icons.
pub use crate::icons::{FeatherIconsPlugin, Icon, IconNode};

// Rendering / styling primitives.
pub use crate::rendering::{
    Backdrop, BackdropQuality, Border, BorderWidths, GradientStop, InnerShadow, LinearGradient,
    LiquidGlass, Mask, OuterGlow, OuterShadow, Paint, Shimmer, Surface,
};

// Cross-cutting primitives.
pub use crate::primitives::a11y::A11yPlugin;
pub use crate::primitives::clipboard::ClipboardPlugin;
pub use crate::primitives::focus::FocusPlugin;
pub use crate::primitives::interaction::InteractionPlugin;
pub use crate::primitives::keyboard::KeyboardPlugin;
pub use crate::primitives::root::{AppRootSurface, ContentRoot, UiFonts};
pub use crate::primitives::semantic::SemanticPlugin;

// Animation.
pub use crate::animation::animation::UiAnimationPlugin;
pub use crate::animation::blur::BackdropBlurPlugin;
pub use crate::animation::loading::{AppState, LoadingConfig, LoadingPlugin};
pub use crate::animation::motion::UiMotionPlugin;
pub use crate::animation::skeleton::{Skeleton, SkeletonPlugin};

// Components.
pub use crate::components::alert::{Alert, AlertPlugin, AlertVariant};
pub use crate::components::avatar::Avatar;
pub use crate::components::badge::Badge;
pub use crate::components::button::{BeverlyButton, ButtonColor, ButtonPlugin};
pub use crate::components::button_group::{ButtonGroup, ButtonGroupPlugin};
pub use crate::components::card::{Card, CardPlugin};
pub use crate::components::checkbox::{Checkbox, CheckboxPlugin};
pub use crate::components::container::Container;
pub use crate::components::divider::{Divider, DividerPlugin};
pub use crate::components::dropdown::{Dropdown, DropdownPlugin};
pub use crate::components::file_input::{FileInput, FileInputPlugin};
pub use crate::components::footer::{Footer, FooterPlugin};
pub use crate::components::input::{TextInput, TextInputPlugin};
pub use crate::components::link::{Link, LinkPlugin};
pub use crate::components::list_item::{ListItem, ListItemPlugin};
pub use crate::components::modal::{Modal, ModalPlugin};
pub use crate::components::nav_button::NavButton;
pub use crate::components::navbar::{Navbar, NavbarPlugin};
pub use crate::components::pagination::{Pagination, PaginationPlugin};
pub use crate::components::photo::{Photo, PhotoPlugin};
pub use crate::components::progress_bar::{ProgressBar, ProgressBarPlugin};
pub use crate::components::radio::{RadioButton, RadioGroup};
pub use crate::components::search::{Search, SearchPlugin};
pub use crate::components::select::{Select, SelectPlugin};
pub use crate::components::sidebar::{Sidebar, SidebarPlugin};
pub use crate::components::slider::{Slider, SliderPlugin};
pub use crate::components::spinner::{Spinner, SpinnerPlugin};
pub use crate::components::table::{Table, TablePlugin};
pub use crate::components::tabs::{Tab, Tabs, TabsPlugin};
pub use crate::components::text::{ThemedText, ThemedTextPlugin};
pub use crate::components::textarea::{Textarea, TextareaPlugin};
pub use crate::components::title::{ThemedTitle, TitleLevel, TitlePlugin};
pub use crate::components::toast::{Toast, ToastKind};
pub use crate::components::toggle::{Toggle, TogglePlugin};
pub use crate::components::tooltip::{Tooltip, TooltipPlugin};
