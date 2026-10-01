# Pages, Links, and Layouts

Beverly keeps route architecture explicit while letting each page and layout remain an ordinary composable UI tree.

- A `Page` names a route and its content.
- A `Layout` describes the reusable shell around a page.
- A `Link` points to a route pattern.

## Register Pages

Return pages from named functions and register them together on the root `Ui`:

```rust
use beverly::prelude::*;

fn application_ui() -> Ui {
    ui().pages([
        home_page(),
        users_page(),
        user_page(),
    ])
}

fn home_page() -> Page {
    page()
        .route("/")
        .layout(app_layout())
        .children([text("Home")])
}

fn users_page() -> Page {
    page()
        .route("/users")
        .layout(app_layout())
        .children([text("User list")])
}

fn user_page() -> Page {
    page()
        .route("/users/:id")
        .layout(app_layout())
        .children([text("User profile")])
}
```

The first registered route matching `/` is shown at startup. If no root route exists, Beverly starts with the first registered page. Route patterns match path segments exactly; a segment beginning with `:` captures one value.

## Compose a Layout

Layouts use the same composable elements as pages. `page_outlet()` marks the location where the active page's content is rendered:

```rust
fn app_layout() -> Layout {
    layout().children([
        text("Application header"),
        row()
            .gap(Val::Px(16.0))
            .children([
                text("Sidebar"),
                page_outlet(),
            ]),
        text("Application footer"),
    ])
}
```

A page without `.layout(...)` renders directly in the route surface. Pages using the same layout share its shell; changing the layout does not change their routes or content.

## Navigate with Links

The visible string passed to `link()` is its label. Accessibility description and destination are separate:

```rust
fn user_link(user_id: u64) -> UiElement {
    link("View user")
        .aria("Opens this user's profile")
        .to("/users/:id")
        .params([("id", user_id)])
}
```

When activated, a link with a target navigates only if the resolved path matches a registered page. A link without `.to(...)` still emits `LinkClicked` and can be handled by application code. Disabled links do not activate.

Query strings can be included in the target and are retained when path parameters are substituted:

```rust
link("View activity")
    .aria("Opens this user's activity")
    .to("/users/:id?tab=activity&search=red+fox")
    .params([("id", user_id)])
```

The current path, path parameters, and parsed query are available globally through `Res<RouteState>`. Beverly also attaches a `RouteContext` component to every entity in the active page subtree, including layout elements, page content, and descendants created later. Component systems can read it with `Query<&RouteContext>`; custom context-aware builders can read the same snapshot through `UiBuildContext::route_context()` while constructing their entity.

```rust
fn inspect_route(route: Res<RouteState>) {
    info!("path: {}, id: {:?}, tab: {:?}", route.path, route.params.get("id"), route.query.get("tab"));
}

fn inspect_routed_widget(contexts: Query<&RouteContext>) {
    for context in &contexts {
        info!("id: {:?}, tab: {:?}", context.param("id"), context.query_value("tab"));
    }
}
```

`RouteContext` exposes `path`, `params`, and `query` maps, with `param(name)` and `query_value(name)` convenience accessors. Query keys and values are form-decoded (`+` becomes a space); repeated query keys retain the last value. The same snapshot can be read during fluent construction:

```rust
fn user_header(context: &mut UiBuildContext<'_>) -> Entity {
    let user_id = context
        .route_context()
        .and_then(|route| route.param("id").map(str::to_owned))
        .unwrap_or_default();
    context.world.spawn((Text::new(format!("User {user_id}")), ChildOf(context.parent))).id()
}
```

Routing is internal to the Bevy application and does not synchronize a browser URL. Applications that need custom state transitions can continue to use `NavButton` and handle its interaction in their own systems.
