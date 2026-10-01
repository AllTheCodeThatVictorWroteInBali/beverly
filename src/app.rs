use bevy::prelude::*;

use crate::theme::{Theme, ThemeResource, light_theme};
use crate::primitives::composition::{UiBuildContext, UiElement};

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
                world.spawn(Camera2d);

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