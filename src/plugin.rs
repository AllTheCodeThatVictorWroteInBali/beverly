use bevy::prelude::*;

/// Adds every Beverly component system, animation driver, and the core
/// rendering/theme plumbing to the app.
///
/// Individual components can still be used without this plugin as long as
/// their own plugin (or the systems they depend on) is added manually; see
/// each component's documentation for its specific requirements.
pub struct BeverlyPlugin;

impl Plugin for BeverlyPlugin {
    fn build(&self, app: &mut App) {
        app
            // Rendering / theme foundation.
            .add_plugins(crate::rendering::UiRenderingPlugin)
            .add_plugins(crate::rendering::UiFrameworkAuditPlugin)
            .add_plugins(crate::theme::ThemePlugin)
            // Cross-cutting primitives.
            .add_plugins(crate::primitives::a11y::A11yPlugin)
            .add_plugins(crate::primitives::clipboard::ClipboardPlugin)
            .add_plugins(crate::primitives::focus::FocusPlugin)
            .add_plugins(crate::primitives::interaction::InteractionPlugin)
            .add_plugins(crate::primitives::keyboard::KeyboardPlugin)
            .add_plugins(crate::primitives::semantic::SemanticPlugin)
            // Animation.
            .add_plugins(crate::animation::animation::UiAnimationPlugin)
            .add_plugins(crate::animation::motion::UiMotionPlugin)
            .add_plugins(crate::animation::blur::BackdropBlurPlugin)
            .add_plugins(crate::animation::loading::LoadingPlugin)
            .add_plugins(crate::animation::skeleton::SkeletonPlugin)
            // Icons.
            .add_plugins(crate::icons::FeatherIconsPlugin)
            // Components.
            .add_plugins(crate::components::alert::AlertPlugin)
            .add_plugins(crate::components::button_group::ButtonGroupPlugin)
            .add_plugins(crate::components::card::CardPlugin)
            .add_plugins(crate::components::checkbox::CheckboxPlugin)
            .add_plugins(crate::components::divider::DividerPlugin)
            .add_plugins(crate::components::dropdown::DropdownPlugin)
            .add_plugins(crate::components::file_input::FileInputPlugin)
            .add_plugins(crate::components::footer::FooterPlugin)
            .add_plugins(crate::components::input::TextInputPlugin)
            .add_plugins(crate::components::link::LinkPlugin)
            .add_plugins(crate::components::list_item::ListItemPlugin)
            .add_plugins(crate::components::modal::ModalPlugin)
            .add_plugins(crate::components::navbar::NavbarPlugin)
            .add_plugins(crate::components::pagination::PaginationPlugin)
            .add_plugins(crate::components::photo::PhotoPlugin)
            .add_plugins(crate::components::play_button::PlayButtonPlugin)
            .add_plugins(crate::components::progress_bar::ProgressBarPlugin)
            .add_plugins(crate::components::radio::RadioPlugin)
            .add_plugins(crate::components::search::SearchPlugin)
            .add_plugins(crate::components::select::SelectPlugin)
            .add_plugins(crate::components::sidebar::SidebarPlugin)
            .add_plugins(crate::components::slider::SliderPlugin)
            .add_plugins(crate::components::spinner::SpinnerPlugin)
            .add_plugins(crate::components::table::TablePlugin)
            .add_plugins(crate::components::tabs::TabsPlugin)
            .add_plugins(crate::components::text::ThemedTextPlugin)
            .add_plugins(crate::components::textarea::TextareaPlugin)
            .add_plugins(crate::components::title::TitlePlugin)
            .add_plugins(crate::components::toggle::TogglePlugin)
            .add_plugins(crate::components::tooltip::TooltipPlugin);
    }
}
