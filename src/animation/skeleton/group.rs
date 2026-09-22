use bevy::prelude::*;

use super::{Skeleton, component::finite_nonnegative};

/// One accessible busy, polite loading region for any number of decorative leaves.
/// Caller-supplied Node fields win over the default column layout.
#[derive(Component, Clone, Debug, PartialEq)]
#[require(Node = default_group_node(), Pickable::IGNORE)]
pub struct SkeletonGroup {
    pub label: String,
    pub busy: bool,
}

impl SkeletonGroup {
    pub fn new(label: impl Into<String>) -> Self {
        Self { label: label.into(), busy: true }
    }

    /// Clear busy when replacing a region's children with real content.
    pub fn busy(mut self, busy: bool) -> Self { self.busy = busy; self }
}

fn default_group_node() -> Node {
    Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(8.0), ..default() }
}

/// Spawn configuration, not another render component or animation owner.
#[derive(Clone, Debug)]
pub struct SkeletonTextLines {
    count: usize,
    group: SkeletonGroup,
    node: Node,
    line: Skeleton,
    final_width: Option<Val>,
}

impl SkeletonTextLines {
    pub fn new(count: usize) -> Self {
        Self {
            count,
            group: SkeletonGroup::new("Loading content"),
            node: default_group_node(),
            line: Skeleton::text().width(Val::Percent(100.0)),
            final_width: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self { self.group.label = label.into(); self }
    /// Replace the entire group layout explicitly; it is not modified at sync time.
    pub fn node(mut self, node: Node) -> Self { self.node = node; self }
    pub fn width(mut self, width: Val) -> Self { self.node.width = width; self }
    pub fn gap(mut self, gap: f32) -> Self { self.node.row_gap = Val::Px(finite_nonnegative(gap, 8.0)); self }
    pub fn final_width(mut self, width: Val) -> Self { self.final_width = Some(width); self }
    /// Shared leaf styling (e.g. shimmer direction, phase, or explicit height).
    pub fn line(mut self, line: Skeleton) -> Self { self.line = line; self }

    /// Spawn a group, optionally attaching the returned Entity to an existing parent.
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let mut group = commands.spawn((self.node, self.group));
        group.with_children(|parent| {
            for index in 0..self.count {
                let mut line = self.line.clone();
                if index + 1 == self.count {
                    if let Some(width) = self.final_width { line = line.width(width); }
                }
                parent.spawn(line);
            }
        });
        group.id()
    }
}