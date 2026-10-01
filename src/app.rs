use bevy::prelude::*;

use crate::theme::{Theme, ThemeResource, light_theme};
use crate::primitives::composition::{UiBuildContext, UiElement};
use crate::primitives::routing::{Page, RouteRegistry};

/// Fluent configuration for a Beverly application.
pub struct BeverlyApp {
    window: Window,
    theme: Theme,
    font_path: Option<String>,
    children: Vec<UiElement>,
}

/// Starts a Beverly application configuration.
pub fn app() -> BeverlyApp {
    BeverlyApp {
        window: Window::default(),
        theme: light_theme(),
        font_path: None,
        children: Vec::new(),
    }
}

impl BeverlyApp {
    /// Sets the initial logical window size.
    pub fn window_size(mut self, width: u32, height: u32) -> Self {
        self.window.resolution = (width, height).into();
        self
    }

    /// Sets the window title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.window.title = title.into();
        self
    }

    /// Sets the initial theme.
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Loads the application's shared font from its asset path.
    pub fn font(mut self, path: impl Into<String>) -> Self {
        self.font_path = Some(path.into());
        self
    }

    /// Adds composable children to the application's root UI surface.
    pub fn children<I>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = UiElement>,
    {
        self.children.extend(children);
        self
    }

    /// Builds the Bevy app and starts its event loop.
    pub fn run(self) {
        let window = self.window;
        let theme = self.theme;
        let font_path = self.font_path;
        let children = self.children;

        let mut runtime = App::new();
        runtime
            .add_plugins(DefaultPlugins.set(WindowPlugin {
                primary_window: Some(window),
                ..default()
            }))
            .add_plugins(crate::BeverlyPlugin)
            .insert_resource(ThemeResource { current: theme })
            .add_systems(Startup, move |world: &mut World| {
                let root = world.spawn((
                    Node {
                        width: percent(100),
                        height: percent(100),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    crate::primitives::root::AppRootSurface,
                )).id();

                for child in &children {
                    child.clone().spawn_in_context(&mut UiBuildContext { world, parent: root });
                }

                if let Some(path) = &font_path {
                    let font = world.resource::<AssetServer>().load(path.clone());
                    world.insert_resource(crate::primitives::root::UiFonts {
                        text: font,
                    });
                }
            })
            .run();
    }
}

/// A composable root UI tree - the Beverly equivalent of a window's content.
///
/// Built with [`ui()`] and attached to a Bevy [`App`] through [`BeverlyAppExt::ui`].
pub struct Ui {
    theme: Theme,
    width: Val,
    height: Val,
    justify_content: JustifyContent,
    align_items: AlignItems,
    flex_direction: FlexDirection,
    row_gap: Val,
    column_gap: Val,
    children: Vec<UiElement>,
    pages: Vec<Page>,
}

/// Starts a new root UI composition.
pub fn ui() -> Ui {
    Ui {
        theme: light_theme(),
        width: Val::Auto,
        height: Val::Auto,
        justify_content: JustifyContent::FlexStart,
        align_items: AlignItems::Stretch,
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(0.0),
        column_gap: Val::Px(0.0),
        children: Vec::new(),
        pages: Vec::new(),
    }
}

impl Ui {
    /// Sets the UI tree's theme.
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the root node's width.
    pub fn width(mut self, width: Val) -> Self {
        self.width = width;
        self
    }

    /// Sets the root node's height.
    pub fn height(mut self, height: Val) -> Self {
        self.height = height;
        self
    }

    /// Centers the root's children both horizontally and vertically.
    pub fn center(mut self) -> Self {
        self.justify_content = JustifyContent::Center;
        self.align_items = AlignItems::Center;
        self
    }

    /// Lays out the root's children in a column (the default).
    pub fn column(mut self) -> Self {
        self.flex_direction = FlexDirection::Column;
        self
    }

    /// Lays out the root's children in a row.
    pub fn row(mut self) -> Self {
        self.flex_direction = FlexDirection::Row;
        self
    }

    /// Sets the spacing between the root's children.
    pub fn gap(mut self, gap: Val) -> Self {
        self.row_gap = gap;
        self.column_gap = gap;
        self
    }

    /// Adds composable children to the root UI surface.
    pub fn children<I>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = UiElement>,
    {
        self.children.extend(children);
        self
    }

    /// Registers the application's routable pages.
    pub fn pages<I>(mut self, pages: I) -> Self
    where
        I: IntoIterator<Item = Page>,
    {
        self.pages.extend(pages);
        self
    }
}

/// Extends Bevy's [`App`] with Beverly's primary entry point.
///
/// This is the recommended way to start a Beverly application - it installs
/// `DefaultPlugins` and [`crate::BeverlyPlugin`], applies the theme, and
/// spawns the root [`Ui`] tree, so callers never need to think in terms of
/// entities, components, or startup systems:
///
/// ```no_run
/// use beverly::prelude::*;
///
/// fn main() {
///     App::new()
///         .ui(my_ui())
///         .run();
/// }
///
/// fn my_ui() -> Ui {
///     ui()
///         .theme(light_theme())
///         .center()
///         .children([text("Hello, World!")])
/// }
/// ```
pub trait BeverlyAppExt {
    /// Installs Beverly and spawns `ui` as the application's root UI tree.
    fn ui(&mut self, ui: Ui) -> &mut Self;
}

impl BeverlyAppExt for App {
    fn ui(&mut self, ui: Ui) -> &mut Self {
        let Ui {
            theme,
            width,
            height,
            justify_content,
            align_items,
            flex_direction,
            row_gap,
            column_gap,
            children,
            pages,
        } = ui;

        self.add_plugins(DefaultPlugins)
            .add_plugins(crate::BeverlyPlugin)
            .insert_resource(ThemeResource { current: theme })
            .insert_resource(RouteRegistry(pages.clone()))
            .add_systems(Startup, move |world: &mut World| {
                let root = world
                    .spawn((
                        Node {
                            width,
                            height,
                            justify_content,
                            align_items,
                            flex_direction,
                            row_gap,
                            column_gap,
                            ..default()
                        },
                        crate::primitives::root::AppRootSurface,
                    ))
                    .id();

                if pages.is_empty() {
                    for child in &children {
                        child
                            .clone()
                            .spawn_in_context(&mut UiBuildContext { world, parent: root });
                    }
                }
            });

        self
    }
}