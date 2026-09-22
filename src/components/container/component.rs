use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Container;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContainerScroll {
    pub x: ScrollMode,
    pub y: ScrollMode,
    pub z: ScrollMode,
}

impl ContainerScroll {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn x() -> Self {
        Self::default().with_axis(ScrollAxis::X, ScrollMode::Scroll)
    }

    pub fn y() -> Self {
        Self::default().with_axis(ScrollAxis::Y, ScrollMode::Scroll)
    }

    pub fn both() -> Self {
        Self::default()
            .with_axis(ScrollAxis::X, ScrollMode::Scroll)
            .with_axis(ScrollAxis::Y, ScrollMode::Scroll)
    }

    pub fn with_axis(mut self, axis: ScrollAxis, mode: ScrollMode) -> Self {
        match axis {
            ScrollAxis::X => self.x = mode,
            ScrollAxis::Y => self.y = mode,
            ScrollAxis::Z => self.z = mode,
        }
        self
    }

    pub fn with_x(self, mode: ScrollMode) -> Self {
        self.with_axis(ScrollAxis::X, mode)
    }

    pub fn with_y(self, mode: ScrollMode) -> Self {
        self.with_axis(ScrollAxis::Y, mode)
    }

    pub fn with_z(self, mode: ScrollMode) -> Self {
        self.with_axis(ScrollAxis::Z, mode)
    }

    pub fn to_overflow(self) -> Overflow {
        Overflow {
            x: self.x.to_overflow_axis(),
            y: self.y.to_overflow_axis(),
        }
    }

    pub fn is_scrollable_xy(self) -> bool {
        matches!(self.x, ScrollMode::Scroll) || matches!(self.y, ScrollMode::Scroll)
    }
}

impl Default for ContainerScroll {
    fn default() -> Self {
        Self {
            x: ScrollMode::Clip,
            y: ScrollMode::Clip,
            z: ScrollMode::Clip,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollAxis {
    X,
    Y,
    Z,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollMode {
    Clip,
    Scroll,
}

impl ScrollMode {
    fn to_overflow_axis(self) -> OverflowAxis {
        match self {
            ScrollMode::Clip => OverflowAxis::Clip,
            ScrollMode::Scroll => OverflowAxis::Scroll,
        }
    }
}

/// Spawn a reusable UI container with configurable scroll behavior.
/// UI layout owns `UiTransform`; do not add a parallel spatial transform hierarchy.
pub fn spawn_container<B: Bundle>(
    parent: &mut ChildSpawnerCommands,
    mut node: Node,
    background: BackgroundColor,
    scroll: ContainerScroll,
    extra: B,
    content: impl FnOnce(&mut ChildSpawnerCommands),
) -> Entity {
    node.overflow = scroll.to_overflow();

    let mut entity = parent.spawn((
        Name::new("ui-container"),
        Container,
        scroll,
        node,
        background,
        Visibility::Visible,
        InheritedVisibility::default(),
        ViewVisibility::default(),
        Children::default(),
        extra,
    ));

    if scroll.is_scrollable_xy() {
        entity.insert(ScrollPosition::default());
    }

    entity.with_children(content).id()
}

#[cfg(test)]
mod hierarchy_tests {
    use super::*;

    #[test]
    fn nested_ui_containers_do_not_require_spatial_ancestors() {
        let mut world = World::new();
        let root = world.spawn(Node::default()).id();
        world.commands().entity(root).with_children(|parent| {
            spawn_container(parent, Node::default(), BackgroundColor(Color::NONE),
                ContainerScroll::none(), (), |container| {
                    spawn_container(container, Node::default(), BackgroundColor(Color::NONE),
                        ContainerScroll::y(), (), |_| {});
                });
        });
        world.flush();
        let mut nodes = world.query::<(Entity, &UiTransform, &UiGlobalTransform, &InheritedVisibility)>();
        let entities: Vec<_> = nodes.iter(&world).map(|(entity, ..)| entity).collect();
        assert_eq!(entities.len(), 3);
        for entity in entities {
            assert!(world.get::<Transform>(entity).is_none());
            assert!(world.get::<GlobalTransform>(entity).is_none());
        }
    }
}
