# Navigation

Beverly's navigation components provide layout and interactive controls. They do not impose a router or own the application's current page. Your app handles a navigation button's `Interaction::Pressed`, changes its own page state, and builds or updates the corresponding view.

The navigation pieces are:

- `Navbar` and `NavbarSections` for a three-section top bar.
- `Sidebar` and `SidebarState` for a collapsible side navigation surface.
- `NavButton` and `PageId` for page buttons used by the sidebar.

> **The component reports the user's intent through interaction state. The application owns routing and page state.**

## Navigation Flow

`nav_button(...)` creates a Bevy `Button` with a `NavButton { page }` component. When the user clicks or activates it, Bevy changes the button's `Interaction`. A system can observe that change and route to the selected page:

```text
Pointer or keyboard activation
    ↓
Interaction becomes Pressed
    ↓
Query matches Changed<Interaction> + NavButton
    ↓
Application selects NavButton.page
```

This is an interaction-triggered system, not a Beverly navigation message. The `NavButton` does not publish a custom Bevy `Message`, change a Bevy `State`, or spawn page content for you.

```rust
use bevy::prelude::*;
use beverly::components::nav_button::{NavButton, PageId};

#[derive(States, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
enum AppPage {
    #[default]
    Music,
    Settings,
}

fn route_from_nav_button(
    buttons: Query<(&Interaction, &NavButton), Changed<Interaction>>,
    mut next_page: ResMut<NextState<AppPage>>,
) {
    for (interaction, button) in &buttons {
        if *interaction == Interaction::Pressed {
            let page = match button.page {
                PageId::Settings => AppPage::Settings,
                _ => AppPage::Music,
            };
            next_page.set(page);
        }
    }
}
```

Register the app state and handler in your own plugin. `BeverlyPlugin` installs the visual and sidebar systems; the application still decides what each `PageId` means.

## Sidebar

`spawn_sidebar` builds a sidebar with a drawer toggle and buttons for `PageId::all()`. It takes a parent child-spawner, the `UiFonts` resource, and a top offset in pixels:

```rust
use bevy::prelude::*;
use beverly::components::sidebar::spawn_sidebar;
use beverly::primitives::root::UiFonts;

fn build_sidebar(parent: &mut ChildSpawnerCommands, ui_fonts: &UiFonts) {
    spawn_sidebar(parent, ui_fonts, 64.0);
}
```

The offset is useful when the sidebar sits below a fixed top bar. `SidebarPlugin`, included by `BeverlyPlugin`, initializes `SidebarState`, handles the drawer toggle, animates the sidebar width, and updates its theme colors.

`SidebarState` is a resource with a public `open: bool` field. It defaults to open. The built-in drawer button toggles this value; the sidebar system animates between expanded and collapsed widths and hides text labels in the collapsed layout.

## Page Buttons

`nav_button(parent, page, ui_fonts)` creates one sidebar button. `drawer_toggle(parent, ui_fonts)` creates the Menu control that toggles `SidebarState`:

```rust
use bevy::prelude::*;
use beverly::components::nav_button::{PageId, drawer_toggle, nav_button};
use beverly::primitives::root::UiFonts;

fn add_custom_sidebar_items(parent: &mut ChildSpawnerCommands, ui_fonts: &UiFonts) {
    drawer_toggle(parent, ui_fonts);
    nav_button(parent, PageId::Settings, ui_fonts);
}
```

`PageId` is the page metadata carried by a navigation button. Its methods are:

- `PageId::all()` returns the seven pages used in the default sidebar. `PlaylistDetail` is intentionally not in that list, but can still be used to create a button directly.
- `page.label()` returns the display label.
- `page.body()` returns sample descriptive text for that page.
- `page.icon()` returns the associated Feather icon.
- `page.accent()` returns the page's example accent color.

The default button list is only a starting point. To render a custom set or order, call `nav_button` for the `PageId` values your app wants to show.

## Navbar

`spawn_navbar(commands, fixed)` creates a full-width, three-column navbar and returns its entity. It also inserts `NavbarSections` on that entity, containing the child entity IDs for the left, center, and right sections.

```rust
use bevy::prelude::*;
use beverly::components::navbar::{
    NavbarSections, add_to_center, add_to_left, add_to_right, spawn_navbar,
};

fn build_top_bar(mut commands: Commands) {
    spawn_navbar(&mut commands, true);
}

fn fill_top_bar(
    mut commands: Commands,
    new_navbars: Query<&NavbarSections, Added<NavbarSections>>,
) {
    for sections in &new_navbars {
        add_to_left(&mut commands, sections, text("Workspace"));
        add_to_center(&mut commands, sections, text("Library"));
        add_to_right(&mut commands, sections, text("Account"));
    }
}
```

Run `build_top_bar` during `Startup` and `fill_top_bar` during `Update`. The section query runs after the navbar and its children have been created, and `Added<NavbarSections>` prevents adding the sample content repeatedly.

Use `true` for a fixed navbar positioned at the top of its parent, or `false` for a relative navbar in normal layout flow. `NavbarPlugin`, included by `BeverlyPlugin`, applies that positioning when the `Navbar` component changes.

The `Navbar` builder methods are:

- `Navbar::new()` creates a non-fixed navbar.
- `.fixed(bool)` sets whether the navbar uses fixed positioning.

`NavbarBundle::new(fixed)` constructs the navbar bundle directly. Most applications should use `spawn_navbar`, which also creates the three section entities and records their IDs.

The section helpers all take `&mut Commands`, `&NavbarSections`, and any Bevy `Bundle`; each returns the spawned child entity:

- `add_to_left(...)` places a bundle in the left section.
- `add_to_center(...)` places a bundle in the center section.
- `add_to_right(...)` places a bundle in the right section.

## Enter and Exit Transitions

Page transitions belong to your page lifecycle, not to `NavButton`. Beverly's `UiAnimationPlugin` (installed by `BeverlyPlugin`) can animate a page root with `TransformTransitionTarget`. For a Bevy UI `Node`, the target's X/Y translation is measured in logical pixels.

Start an entering page slightly offset, then target its resting transform:

```rust
use std::time::Duration;
use bevy::prelude::*;
use beverly::animation::animation::{Easing, TransformTransitionTarget, Transition};

fn spawn_page(commands: &mut Commands) -> Entity {
    let transition = Transition::new(Duration::from_millis(220), Easing::EaseOut);

    commands
        .spawn((
            Node::default(),
            UiTransform::from_translation(Val2::px(24.0, 0.0)),
            TransformTransitionTarget::new(Transform::IDENTITY, transition),
        ))
        .id()
}
```

For an exit, set the current page root's target to a small offset and let the animation run:

```rust
fn begin_page_exit(commands: &mut Commands, page_root: Entity) {
    let transition = Transition::new(Duration::from_millis(180), Easing::EaseIn);

    commands.entity(page_root).insert(TransformTransitionTarget::new(
        Transform::from_translation(Vec3::new(-24.0, 0.0, 0.0)),
        transition,
    ));
}
```

Do not despawn the outgoing root immediately when starting its exit animation. Keep it mounted until the transition duration has elapsed, then remove it and finish the route change. A common lifecycle is to mark the old page as exiting, start its target animation, and let a timer or page-transition coordinator despawn it after the transition. Spawn the new page with its own entering target. When using Bevy `OnExit` systems, avoid automatic immediate despawn of an entity that still needs to animate.

Transitions are decorative by default, so Beverly's reduced-motion policy can make them immediate. Keep page identity and focus behavior correct without relying on the motion to communicate the route change.

## Accessibility

Use clear page labels, preserve a logical keyboard order, and make the current route visually apparent in your app's page-button styling. Navigation controls are actions for changing context, so the application should also move focus to an appropriate location when a new page is shown. Keep the route change understandable when motion is disabled.
