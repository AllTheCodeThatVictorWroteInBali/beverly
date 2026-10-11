use bevy::prelude::*;

use crate::primitives::semantic::{SemanticNode, SemanticRole};
use crate::rendering::{Paint, Surface};
use crate::theme::ThemeResource;

/// Event emitted when a [`Link`] is clicked.
#[derive(Message, Debug, Clone)]
pub struct LinkClicked {
    pub entity: Entity,
}

/// Marker component for a reusable link.
#[derive(Component, Debug, Clone)]
pub struct Link {
    pub text: String,
    pub icon: Option<String>,
    pub disabled: bool,
    pub aria_description: Option<String>,
    pub target_path: Option<String>,
    pub external: bool,
    pub params: Vec<(String, String)>,
}

impl Link {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            icon: None,
            disabled: false,
            aria_description: None,
            target_path: None,
            external: false,
            params: Vec::new(),
        }
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn aria(mut self, description: impl Into<String>) -> Self {
        self.aria_description = Some(description.into());
        self
    }

    pub fn to(mut self, path: impl Into<String>) -> Self {
        self.target_path = Some(path.into());
        self
    }

    /// Opens the target in the system web browser instead of routing in-app.
    pub fn external(mut self) -> Self {
        self.external = true;
        self
    }

    pub fn params<I, K, V>(mut self, params: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: ToString,
    {
        self.params = params
            .into_iter()
            .map(|(key, value)| (key.into(), value.to_string()))
            .collect();
        self
    }

    pub(crate) fn resolved_target(&self) -> String {
        let Some(path) = &self.target_path else {
            return String::new();
        };
        let (path_part, query_part) = path
            .split_once('?')
            .map_or((path.as_str(), None), |(path, query)| (path, Some(query)));
        let mut segments: Vec<String> = path_part.split('/').map(str::to_string).collect();
        for segment in &mut segments {
            if let Some(name) = segment.strip_prefix(':') {
                if let Some((_, value)) = self.params.iter().find(|(key, _)| key == name) {
                    *segment = value.clone();
                }
            }
        }
        let resolved_path = segments.join("/");
        match query_part {
            Some(query) if !query.is_empty() => format!("{resolved_path}?{query}"),
            _ => resolved_path,
        }
    }
}

/// Creates a composable link element with the given visible text.
pub fn link(text: impl Into<String>) -> crate::primitives::composition::UiElement {
    crate::primitives::composition::UiElement::link(Link::new(text))
}

/// Optional child marker for the text portion.
#[derive(Component)]
pub struct LinkText;

/// Optional child marker for the icon portion.
#[derive(Component)]
pub struct LinkIcon;

/// Plugin providing link behavior.
pub struct LinkPlugin;

impl Plugin for LinkPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<crate::primitives::semantic::SemanticAccessibilityPlugin>() {
            app.add_plugins(crate::primitives::semantic::SemanticAccessibilityPlugin);
        }
        app.add_message::<LinkClicked>().add_systems(
            Update,
            (
                link_semantics_system,
                link_interaction_system,
                open_external_link_system,
                link_visual_system,
            ),
        );
    }
}

fn link_semantics_system(mut commands: Commands, links: Query<(Entity, &Link), Added<Link>>) {
    for (entity, link) in &links {
        commands.entity(entity).insert((
            Button,
            crate::primitives::a11y::TabIndex(if link.disabled { -1 } else { 0 }),
            crate::primitives::interaction::PointerCursorOnHover,
            SemanticNode::new(SemanticRole::Link).label(link.text.clone()),
        ));
        if let Some(description) = &link.aria_description {
            commands
                .entity(entity)
                .insert(crate::primitives::semantic::AriaDescription(
                    description.clone(),
                ));
        }
    }
}

fn link_interaction_system(
    mut interaction_query: Query<(Entity, &Interaction, &Link), (Changed<Interaction>, With<Link>)>,
    mut events: MessageWriter<LinkClicked>,
) {
    for (entity, interaction, link) in &mut interaction_query {
        if link.disabled {
            continue;
        }

        if *interaction == Interaction::Pressed {
            events.write(LinkClicked { entity });
        }
    }
}

fn open_external_link_system(mut clicks: MessageReader<LinkClicked>, links: Query<&Link>) {
    for click in clicks.read() {
        let Ok(link) = links.get(click.entity) else {
            continue;
        };
        if !link.external {
            continue;
        }
        let Some(target) = link.target_path.as_deref() else {
            continue;
        };
        let Ok(url) = url::Url::parse(target) else {
            warn!("External link target must be an absolute HTTP(S) URL: {target:?}");
            continue;
        };
        if !matches!(url.scheme(), "http" | "https") || url.host().is_none() {
            warn!("External link target must be an absolute HTTP(S) URL: {target:?}");
            continue;
        }
        if let Err(error) = webbrowser::open(url.as_str()) {
            warn!("Failed to open external link {target:?}: {error}");
        }
    }
}

fn link_visual_system(
    theme: Res<ThemeResource>,
    links: Query<(Entity, &Link, Option<&Interaction>)>,
    mut backgrounds: Query<&mut Surface, With<Link>>,
    mut text_colors: Query<(&ChildOf, &mut TextColor), With<LinkText>>,
) {
    let colors = theme.current.colors;

    for (entity, link, interaction) in &links {
        if let Ok(mut surface) = backgrounds.get_mut(entity) {
            surface.border = None;
            surface.fill = Paint::solid(Color::NONE);
        }

        for (parent, mut text_color) in &mut text_colors {
            if parent.parent() != entity {
                continue;
            }

            text_color.0 = if link.disabled {
                colors.text_disabled
            } else if matches!(interaction.copied(), Some(Interaction::Hovered)) {
                colors.primary_hover
            } else if matches!(interaction.copied(), Some(Interaction::Pressed)) {
                colors.primary_active
            } else {
                colors.primary
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ThemeResource, light_theme};
    use bevy::a11y::AccessibilityNode;

    #[test]
    fn link_gets_link_semantics_and_accessible_name() {
        let mut app = App::new();
        app.insert_resource(ThemeResource {
            current: light_theme(),
        })
        .add_plugins(LinkPlugin);
        let entity = app.world_mut().spawn(Link::new("Documentation")).id();
        app.update();

        let node = app.world().get::<AccessibilityNode>(entity).unwrap();
        assert_eq!(node.0.label(), Some("Documentation"));
        assert_eq!(node.0.role(), accesskit::Role::Link);
        assert!(app.world().get::<Button>(entity).is_some());
        assert!(
            app.world()
                .get::<crate::primitives::interaction::PointerCursorOnHover>(entity)
                .is_some()
        );
    }

    #[test]
    fn external_target_is_opt_in() {
        assert!(!Link::new("Internal").external);
        let external = Link::new("Beverly")
            .to("https://www.beverlyui.com")
            .external();
        assert!(external.external);
        assert_eq!(
            external.target_path.as_deref(),
            Some("https://www.beverlyui.com")
        );
    }
}
