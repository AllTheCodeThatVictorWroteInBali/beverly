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
        app.add_systems(PostStartup, ensure_default_ui_camera)
            // Rendering / theme foundation.
            .add_plugins(crate::rendering::UiRenderingPlugin)
            .add_plugins(crate::rendering::UiFrameworkAuditPlugin)
            .add_plugins(crate::theme::ThemePlugin)
            // Cross-cutting primitives.
            .add_plugins(crate::primitives::a11y::A11yPlugin)
            .add_plugins(crate::primitives::binding::BindingPlugin)
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
            .add_plugins(crate::components::avatar::AvatarPlugin)
            .add_plugins(crate::components::button::ButtonPlugin)
            .add_plugins(crate::components::button_group::ButtonGroupPlugin)
            .add_plugins(crate::components::card::CardPlugin)
            .add_plugins(crate::components::checkbox::CheckboxPlugin)
            .add_plugins(crate::components::divider::DividerPlugin)
            .add_plugins(crate::components::dots::DotsPlugin)
            .add_plugins(crate::components::dropdown::DropdownPlugin)
            .add_plugins(crate::components::file_input::FileInputPlugin)
            .add_plugins(crate::components::form::FormPlugin)
            .add_plugins(crate::components::footer::FooterPlugin)
            .add_plugins(crate::components::input::TextInputPlugin)
            .add_plugins(crate::components::link::LinkPlugin)
            .add_plugins(crate::primitives::routing::RouterPlugin)
            .add_plugins(crate::components::list_item::ListItemPlugin)
            .add_plugins(crate::components::modal::ModalPlugin)
            .add_plugins(crate::components::navbar::NavbarPlugin)
            .add_plugins(crate::components::pagination::PaginationPlugin)
            .add_plugins(crate::components::photo::PhotoPlugin)
            .add_plugins(crate::components::play_button::PlayButtonPlugin)
            .add_plugins(crate::components::progress_bar::ProgressBarPlugin)
            .add_plugins(crate::components::radio::RadioPlugin)
            .add_plugins(crate::components::search::SearchPlugin)
            .add_plugins(crate::components::select::SelectPlugin::<String>::default())
            .add_plugins(crate::components::sidebar::SidebarPlugin)
            .add_plugins(crate::components::slider::SliderPlugin)
            .add_plugins(crate::components::spinner::SpinnerPlugin)
            .add_plugins(crate::components::table::TablePlugin)
            .add_plugins(crate::components::tabs::TabsPlugin)
            .add_plugins(crate::components::text::ThemedTextPlugin)
            .add_plugins(crate::components::textarea::TextareaPlugin)
            .add_plugins(crate::components::theme_toggle::ThemeTogglePlugin)
            .add_plugins(crate::components::title::TitlePlugin)
            .add_plugins(crate::components::toast::ToastPlugin)
            .add_plugins(crate::components::toggle::TogglePlugin)
            .add_plugins(crate::components::tooltip::TooltipPlugin);
    }
}

// Icon proxy cameras are Camera2d too, but they must render above the UI, so a
// separate default UI camera is still needed when the app brings none of its own.
fn ensure_default_ui_camera(
    mut commands: Commands,
    cameras: Query<
        (),
        (
            With<Camera2d>,
            Without<crate::icons::component::UiIconProxyCamera>,
            Without<crate::icons::component::UiIconContentClipCamera>,
        ),
    >,
) {
    if cameras.is_empty() {
        commands.spawn((Camera2d, bevy::ui::IsDefaultUiCamera));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_default_ui_camera_when_missing() {
        let mut app = App::new();
        app.add_systems(PostStartup, ensure_default_ui_camera);
        app.update();

        let camera_count = app
            .world_mut()
            .query_filtered::<Entity, With<bevy::ui::IsDefaultUiCamera>>()
            .iter(app.world())
            .count();
        assert_eq!(camera_count, 1);
    }

    #[test]
    fn preserves_an_existing_2d_camera() {
        let mut app = App::new();
        app.world_mut().spawn(Camera2d);
        app.add_systems(PostStartup, ensure_default_ui_camera);
        app.update();

        let camera_count = app
            .world_mut()
            .query_filtered::<Entity, With<Camera2d>>()
            .iter(app.world())
            .count();
        assert_eq!(camera_count, 1);
    }
}
