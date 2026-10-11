use std::collections::HashMap;

use bevy::prelude::*;

use crate::components::link::{Link, LinkClicked};
use crate::primitives::composition::{UiBuildContext, UiElement};
use crate::primitives::root::AppRootSurface;

/// A named Rust function can return a `Page` describing one routable view.
#[derive(Clone, Default)]
pub struct Page {
    route: String,
    layout: Option<Layout>,
    children: Vec<UiElement>,
}

/// Starts a page definition. Pages default to the `/` route.
pub fn page() -> Page {
    Page {
        route: "/".to_string(),
        ..default()
    }
}

impl Page {
    /// Sets this page's route pattern. Segments beginning with `:` capture a value.
    pub fn route(mut self, route: impl Into<String>) -> Self {
        self.route = route.into();
        self
    }

    /// Sets the reusable shell used to render this page.
    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = Some(layout);
        self
    }

    /// Adds this page's content.
    pub fn children<I>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = UiElement>,
    {
        self.children.extend(children);
        self
    }
}

/// A reusable UI shell. A layout normally contains one `page_outlet()`.
#[derive(Clone, Default)]
pub struct Layout {
    children: Vec<UiElement>,
}

/// Starts a reusable layout definition.
pub fn layout() -> Layout {
    Layout::default()
}

impl Layout {
    /// Adds composable elements to the layout shell.
    pub fn children<I>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = UiElement>,
    {
        self.children.extend(children);
        self
    }
}

/// Marks where the active page's content is rendered inside a layout.
pub fn page_outlet() -> UiElement {
    UiElement::custom_in_context(spawn_page_outlet)
}

#[derive(Component)]
struct PageOutlet;

#[derive(Resource, Default)]
pub(crate) struct RouteRegistry(pub(crate) Vec<Page>);

/// The currently matched route and its captured path parameters.
#[derive(Resource, Clone, Debug, Default)]
pub struct RouteState {
    pub path: String,
    pub params: HashMap<String, String>,
    pub query: HashMap<String, String>,
    page_index: Option<usize>,
    dirty: bool,
}

impl RouteState {
    pub fn context(&self) -> RouteContext {
        RouteContext {
            path: self.path.clone(),
            params: self.params.clone(),
            query: self.query.clone(),
        }
    }
}

/// Route data attached to every entity in the active page's UI subtree.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq)]
pub struct RouteContext {
    pub path: String,
    pub params: HashMap<String, String>,
    pub query: HashMap<String, String>,
}

impl RouteContext {
    pub fn param(&self, name: &str) -> Option<&str> {
        self.params.get(name).map(String::as_str)
    }

    pub fn query_value(&self, name: &str) -> Option<&str> {
        self.query.get(name).map(String::as_str)
    }
}

#[derive(Resource, Default, Clone)]
struct RouteRenderState {
    children: Vec<UiElement>,
}

#[derive(Resource)]
struct RouteMount(Entity);

#[derive(Component)]
struct RouteMountRoot;

/// Installs route matching and renders registered pages after the root UI exists.
pub struct RouterPlugin;

impl Plugin for RouterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RouteRegistry>()
            .init_resource::<RouteState>()
            .init_resource::<RouteRenderState>()
            .add_systems(PostStartup, initialize_router)
            .add_systems(Update, (route_link_system, apply_route_change).chain())
            .add_systems(PostUpdate, propagate_route_context);
    }
}

fn initialize_router(world: &mut World) {
    let pages = world.resource::<RouteRegistry>().0.clone();
    if pages.is_empty() {
        return;
    }

    let root = world
        .query_filtered::<Entity, With<AppRootSurface>>()
        .single(world)
        .expect("Ui must create its root before registered pages are rendered");
    let mount = world
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            RouteMountRoot,
            ChildOf(root),
        ))
        .id();
    world.insert_resource(RouteMount(mount));

    let initial = pages
        .iter()
        .enumerate()
        .find_map(|(index, page)| match_route(&page.route, "/").map(|params| (index, params)))
        .or_else(|| pages.first().map(|_| (0, HashMap::new())));
    if let Some((index, params)) = initial {
        let page = &pages[index];
        world.insert_resource(RouteState {
            path: if match_route(&page.route, "/").is_some() {
                "/".to_string()
            } else {
                page.route.clone()
            },
            query: parse_query(&page.route),
            params,
            page_index: Some(index),
            dirty: false,
        });
        world.resource_mut::<RouteRenderState>().children = page.children.clone();
        render_page(world, mount, page);
    }
}

fn route_link_system(
    mut clicks: MessageReader<LinkClicked>,
    links: Query<&Link>,
    registry: Res<RouteRegistry>,
    mut route: ResMut<RouteState>,
) {
    for click in clicks.read() {
        let Ok(link) = links.get(click.entity) else {
            continue;
        };
        if link.target_path.is_none() || link.external {
            continue;
        }
        let path = link.resolved_target();
        let Some((index, params)) = registry.0.iter().enumerate().find_map(|(index, page)| {
            match_route(&page.route, &path).map(|params| (index, params))
        }) else {
            continue;
        };
        route.path = path;
        route.params = params;
        route.query = parse_query(&route.path);
        route.page_index = Some(index);
        route.dirty = true;
    }
}

fn apply_route_change(world: &mut World) {
    let (dirty, page_index) = {
        let mut route = world.resource_mut::<RouteState>();
        let dirty = route.dirty;
        route.dirty = false;
        (dirty, route.page_index)
    };
    if !dirty {
        return;
    }

    let registry = world.resource::<RouteRegistry>().0.clone();
    let Some(page) = page_index.and_then(|index| registry.get(index)).cloned() else {
        return;
    };
    let mount = world.resource::<RouteMount>().0;
    render_page(world, mount, &page);
}

fn render_page(world: &mut World, mount: Entity, page: &Page) {
    world.entity_mut(mount).despawn_related::<Children>();
    let context = world.resource::<RouteState>().context();
    world.resource_mut::<RouteRenderState>().children = page.children.clone();

    if let Some(layout) = &page.layout {
        for child in &layout.children {
            child.clone().spawn_in_context(&mut UiBuildContext {
                world,
                parent: mount,
            });
        }
    } else {
        for child in &page.children {
            child.clone().spawn_in_context(&mut UiBuildContext {
                world,
                parent: mount,
            });
        }
    }
    world.flush();
    world.entity_mut(mount).insert(context);
    propagate_route_context_from_root(world, mount);
}

fn spawn_page_outlet(context: &mut UiBuildContext<'_>) -> Entity {
    let parent = context.parent;
    let outlet = context
        .world
        .spawn((
            PageOutlet,
            Node {
                width: percent(100),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ChildOf(parent),
        ))
        .id();
    let render_state = context.world.resource::<RouteRenderState>().clone();
    for child in render_state.children {
        child.spawn_in_context(&mut UiBuildContext {
            world: context.world,
            parent: outlet,
        });
    }
    outlet
}

fn propagate_route_context(world: &mut World) {
    let roots = world
        .query_filtered::<Entity, With<RouteMountRoot>>()
        .iter(world)
        .collect::<Vec<_>>();
    for root in roots {
        propagate_route_context_from_root(world, root);
    }
}

fn propagate_route_context_from_root(world: &mut World, root: Entity) {
    let Some(context) = world.get::<RouteContext>(root).cloned() else {
        return;
    };
    let mut pending = world
        .get::<Children>(root)
        .map(|children| children.iter().collect::<Vec<_>>())
        .unwrap_or_default();
    while let Some(entity) = pending.pop() {
        let children = world
            .get::<Children>(entity)
            .map(|children| children.iter().collect::<Vec<_>>())
            .unwrap_or_default();
        if world.get::<RouteContext>(entity).is_none() {
            world.entity_mut(entity).insert(context.clone());
        }
        pending.extend(children);
    }
}

fn match_route(pattern: &str, path: &str) -> Option<HashMap<String, String>> {
    let pattern_segments = normalized_segments(pattern);
    let path_segments = normalized_segments(path.split('?').next().unwrap_or(path));
    if pattern_segments.len() != path_segments.len() {
        return None;
    }

    let mut params = HashMap::new();
    for (pattern_segment, path_segment) in pattern_segments.iter().zip(path_segments) {
        if let Some(name) = pattern_segment.strip_prefix(':') {
            if name.is_empty() {
                return None;
            }
            params.insert(name.to_string(), path_segment.to_string());
        } else if *pattern_segment != path_segment {
            return None;
        }
    }
    Some(params)
}

fn parse_query(path: &str) -> HashMap<String, String> {
    let Some((_, query)) = path.split_once('?') else {
        return HashMap::new();
    };
    url::form_urlencoded::parse(query.split('#').next().unwrap_or(query).as_bytes())
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect()
}

fn normalized_segments(path: &str) -> Vec<&str> {
    path.trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::link::Link;
    use crate::primitives::composition::{UiBuildContext, row};

    #[derive(Component)]
    struct RouteProbe;

    fn spawn_route_probe(context: &mut UiBuildContext<'_>) -> Entity {
        let route = context
            .route_context()
            .expect("routed page context should be available while building children");
        context
            .world
            .spawn((RouteProbe, route, ChildOf(context.parent)))
            .id()
    }

    #[test]
    fn route_matching_captures_parameters_and_requires_all_segments() {
        let params = match_route("/users/:user_id/orders/:order_id", "/users/42/orders/7")
            .expect("route should match");
        assert_eq!(params.get("user_id").map(String::as_str), Some("42"));
        assert_eq!(params.get("order_id").map(String::as_str), Some("7"));
        assert!(match_route("/users/:id", "/users/42/orders").is_none());
        assert!(match_route("/users", "/teams").is_none());
    }

    #[test]
    fn fluent_link_builder_keeps_label_description_and_route_separate() {
        let element = crate::components::link::link("View user")
            .aria("Opens this user's profile")
            .to("/users/:id?tab=activity")
            .params([("id", 42)]);
        let UiElement::Link(link) = element else {
            panic!("link() must create a link element");
        };

        assert_eq!(link.text, "View user");
        assert_eq!(
            link.aria_description.as_deref(),
            Some("Opens this user's profile")
        );
        assert_eq!(link.resolved_target(), "/users/42?tab=activity");
    }

    #[test]
    fn routed_link_updates_route_state_and_renders_page_outlet() {
        let profile_page = page()
            .route("/users/:id")
            .layout(layout().children([row().children([page_outlet()])]))
            .children([
                crate::components::text::text("User profile"),
                UiElement::custom_in_context(spawn_route_probe),
            ]);
        let mut app = App::new();
        app.add_message::<LinkClicked>()
            .insert_resource(RouteRegistry(vec![profile_page]))
            .add_plugins(RouterPlugin);
        app.world_mut().spawn((
            Node {
                width: percent(100),
                height: percent(100),
                ..default()
            },
            AppRootSurface,
        ));
        let link = app
            .world_mut()
            .spawn(
                Link::new("Open profile")
                    .to("/users/:id?tab=activity&search=red+fox")
                    .params([("id", 42)]),
            )
            .id();
        app.world_mut().write_message(LinkClicked { entity: link });
        app.update();

        let route = app.world().resource::<RouteState>();
        assert_eq!(route.path, "/users/42?tab=activity&search=red+fox");
        assert_eq!(route.params.get("id").map(String::as_str), Some("42"));
        assert_eq!(route.query.get("tab").map(String::as_str), Some("activity"));
        assert_eq!(
            route.query.get("search").map(String::as_str),
            Some("red fox")
        );
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<RouteProbe>>()
                .iter(app.world())
                .count(),
            1
        );
        let probe_context = {
            let mut query = app
                .world_mut()
                .query_filtered::<&RouteContext, With<RouteProbe>>();
            query
                .single(app.world())
                .expect("child builder receives route context")
                .clone()
        };
        assert_eq!(probe_context.param("id"), Some("42"));
        assert_eq!(probe_context.query_value("search"), Some("red fox"));

        let profile_text = {
            let mut query = app.world_mut().query_filtered::<Entity, With<Text>>();
            query
                .iter(app.world())
                .next()
                .expect("profile text should render")
        };
        let late_child = app
            .world_mut()
            .spawn((Text::new("Late child"), ChildOf(profile_text)))
            .id();
        app.update();
        let late_context = app.world().get::<RouteContext>(late_child).unwrap();
        assert_eq!(late_context.param("id"), Some("42"));
        assert_eq!(late_context.query_value("tab"), Some("activity"));
    }

    #[test]
    fn external_link_does_not_change_in_app_route() {
        let mut app = App::new();
        app.add_message::<LinkClicked>()
            .insert_resource(RouteRegistry(vec![
                page().route("https://www.beverlyui.com"),
            ]))
            .insert_resource(RouteState {
                path: "/".to_string(),
                ..default()
            })
            .add_systems(Update, route_link_system);
        let link = app
            .world_mut()
            .spawn(
                Link::new("Beverly")
                    .to("https://www.beverlyui.com")
                    .external(),
            )
            .id();
        app.world_mut().write_message(LinkClicked { entity: link });
        app.update();

        assert_eq!(app.world().resource::<RouteState>().path, "/");
    }
}
