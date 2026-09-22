use crate::components::spinner::{Spinner, SpinnerAnchor};
use crate::theme::ThemeResource;
use crate::components::{footer::Footer, navbar::Navbar, sidebar::Sidebar};
use crate::primitives::root::ContentRoot;
use bevy::camera::Viewport;
use bevy::camera::visibility::RenderLayers;
use bevy::mesh::Mesh;
use bevy::prelude::*;
use bevy::ui::ComputedStackIndex;
use bevy::window::PrimaryWindow;
use bevy_svg::prelude::*;
use std::collections::HashMap;

/// Icon abstraction used by the UI layer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Icon {
    Feather(String),
}

// Chrome (navbar/sidebar/footer) icons render unclipped on this layer.
const UI_ICON_RENDER_LAYER: usize = 31;
// Main-content icons render on this layer through a camera whose viewport is
// clipped to the live area outside the chrome, so they are genuinely occluded
// (not just hidden outright) as they slide under the navbar/sidebar/footer.
const UI_ICON_CONTENT_RENDER_LAYER: usize = 32;
const UI_ICON_STACK_Z_STEP: f32 = 0.01;
const UI_ICON_LOCAL_Z_BIAS: f32 = 0.001;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum UiIconSyncSet {
    Theme,
    Sync,
}

#[derive(Component)]
struct UiIconProxyCamera;

#[derive(Component)]
struct UiIconContentClipCamera;

#[derive(Component)]
struct UiIconProxy {
    owner: Entity,
    // Which camera renders this proxy, so its world position can be computed
    // relative to that camera's own (possibly clipped) viewport center.
    content: bool,
}

#[derive(Component)]
struct UiIconProxyEntity(Entity);

/// Marker for icons that should automatically follow the active UI theme color.
#[derive(Component)]
pub struct ThemedIcon {
    tone: ThemedIconTone,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ThemedIconTone {
    Text,
    Muted,
}

#[derive(Resource, Default)]
struct IconColorVariants {
    handles: HashMap<(String, [u8; 4]), Handle<Svg>>,
}

impl Icon {
    pub fn feather(name: impl Into<String>) -> Self {
        Self::Feather(name.into())
    }

    pub fn path(&self) -> String {
        match self {
            Self::Feather(name) => format!("embedded://beverly/icons/feather/{name}.svg"),
        }
    }
}

/// Component representing a renderable icon instance in the UI.
#[derive(Component, Clone, Debug)]
pub struct IconNode {
    pub icon: Icon,
    pub size: f32,
    pub color: Color,
}

impl IconNode {
    pub fn new(icon: Icon) -> Self {
        Self {
            icon,
            size: 24.0,
            color: Color::WHITE,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

/// Convenience trait for spawning SVG icons.
pub trait IconCommands {
    fn spawn_icon(&mut self, icon: Icon) -> Entity;

    fn spawn_icon_sized(&mut self, icon: Icon, size: f32) -> Entity;

    fn spawn_icon_muted(&mut self, icon: Icon, size: f32) -> Entity;

    fn spawn_icon_colored(&mut self, icon: Icon, size: f32, color: Color) -> Entity;

    fn spawn_feather(&mut self, name: impl Into<String>) -> Entity;

    fn spawn_feather_sized(&mut self, name: impl Into<String>, size: f32) -> Entity;

    fn spawn_feather_muted(&mut self, name: impl Into<String>, size: f32) -> Entity;

    fn spawn_feather_colored(&mut self, name: impl Into<String>, size: f32, color: Color)
    -> Entity;
}

impl<'w> IconCommands for ChildSpawnerCommands<'w> {
    fn spawn_icon(&mut self, icon: Icon) -> Entity {
        self.spawn_icon_sized(icon, 24.0)
    }

    fn spawn_icon_sized(&mut self, icon: Icon, size: f32) -> Entity {
        self.spawn_themed_icon(icon, size, ThemedIconTone::Text)
    }

    fn spawn_icon_muted(&mut self, icon: Icon, size: f32) -> Entity {
        self.spawn_themed_icon(icon, size, ThemedIconTone::Muted)
    }

    fn spawn_icon_colored(&mut self, icon: Icon, size: f32, color: Color) -> Entity {
        let icon_node = IconNode::new(icon).size(size).color(color);

        self.spawn((
            icon_node,
            Node {
                width: Val::Px(size),
                height: Val::Px(size),
                ..default()
            },
        ))
        .id()
    }

    fn spawn_feather(&mut self, name: impl Into<String>) -> Entity {
        self.spawn_feather_sized(name, 24.0)
    }

    fn spawn_feather_sized(&mut self, name: impl Into<String>, size: f32) -> Entity {
        self.spawn_icon_sized(Icon::feather(name), size)
    }

    fn spawn_feather_muted(&mut self, name: impl Into<String>, size: f32) -> Entity {
        self.spawn_icon_muted(Icon::feather(name), size)
    }

    fn spawn_feather_colored(
        &mut self,
        name: impl Into<String>,
        size: f32,
        color: Color,
    ) -> Entity {
        self.spawn_icon_colored(Icon::feather(name), size, color)
    }
}

trait SpawnThemedIcon {
    fn spawn_themed_icon(&mut self, icon: Icon, size: f32, tone: ThemedIconTone) -> Entity;
}

impl<'w> SpawnThemedIcon for ChildSpawnerCommands<'w> {
    fn spawn_themed_icon(&mut self, icon: Icon, size: f32, tone: ThemedIconTone) -> Entity {
        self.spawn((
            ThemedIcon { tone },
            IconNode::new(icon).size(size),
            Node {
                width: Val::Px(size),
                height: Val::Px(size),
                ..default()
            },
        ))
        .id()
    }
}

/// Plugin responsible for Feather SVG icons.
pub struct FeatherIconsPlugin;

impl Plugin for FeatherIconsPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<bevy_svg::prelude::SvgPlugin>() {
            app.add_plugins(bevy_svg::prelude::SvgPlugin);
        }
        super::embed_feather_icons(app);
        app.init_resource::<IconColorVariants>();
        app.configure_sets(Update, UiIconSyncSet::Theme.before(UiIconSyncSet::Sync));
        app.add_systems(
            Startup,
            (spawn_ui_icon_proxy_camera, spawn_ui_icon_content_proxy_camera),
        );
        app.add_systems(
            Update,
            (
                apply_themed_icon_colors.in_set(UiIconSyncSet::Theme),
                update_ui_icon_content_camera_viewport,
                spawn_ui_svg_icon_proxies,
                sync_ui_svg_icon_proxies.in_set(UiIconSyncSet::Sync),
                despawn_orphan_ui_svg_icon_proxies,
                load_feather_icons,
            ),
        );
    }
}

fn apply_themed_icon_colors(
    theme: Res<ThemeResource>,
    mut icons: Query<(&ThemedIcon, &mut IconNode)>,
) {
    let colors = theme.current.colors;

    for (themed, mut icon) in &mut icons {
        let target = match themed.tone {
            ThemedIconTone::Text => colors.text,
            ThemedIconTone::Muted => colors.text_muted,
        };

        if icon.color != target {
            icon.color = target;
        }
    }
}

fn color_key(color: Color) -> [u8; 4] {
    let c = color.to_srgba();
    [
        (c.red * 255.0).round() as u8,
        (c.green * 255.0).round() as u8,
        (c.blue * 255.0).round() as u8,
        (c.alpha * 255.0).round() as u8,
    ]
}

fn ensure_colored_svg_handle(
    icon: &IconNode,
    variants: &mut IconColorVariants,
    asset_server: &AssetServer,
    svgs: &mut Assets<Svg>,
    meshes: &mut Assets<Mesh>,
) -> Handle<Svg> {
    let path = icon.icon.path();
    let key = (path.clone(), color_key(icon.color));

    if let Some(handle) = variants.handles.get(&key) {
        return handle.clone();
    }

    let base_handle: Handle<Svg> = asset_server.load(path);

    if let Some(base_svg) = svgs.get(&base_handle) {
        let mut colored_svg = base_svg.clone();
        for svg_path in &mut colored_svg.paths {
            svg_path.color = icon.color;
        }

        // Re-tessellate so vertex colors in the mesh match the requested icon tint.
        colored_svg.mesh = meshes.add(colored_svg.tessellate());

        let handle = svgs.add(colored_svg);
        variants.handles.insert(key, handle.clone());
        handle
    } else {
        base_handle
    }
}

fn spawn_ui_icon_proxy_camera(
    mut commands: Commands,
    existing: Query<Entity, With<UiIconProxyCamera>>,
) {
    if !existing.is_empty() {
        return;
    }

    commands.spawn((
        UiIconProxyCamera,
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        RenderLayers::layer(UI_ICON_RENDER_LAYER),
    ));
}

// Main-content icons are rendered through a second camera whose viewport is
// clipped every frame (see `update_ui_icon_content_camera_viewport`) to the
// area outside the navbar/sidebar/footer. That viewport is a real GPU scissor,
// so an icon sliding under chrome is genuinely occluded pixel-by-pixel—partly
// visible while partly covered, exactly like CSS z-index—rather than being
// hidden outright the moment it touches the chrome bounds.
fn spawn_ui_icon_content_proxy_camera(
    mut commands: Commands,
    existing: Query<Entity, With<UiIconContentClipCamera>>,
) {
    if !existing.is_empty() {
        return;
    }

    commands.spawn((
        UiIconContentClipCamera,
        Camera2d,
        Camera {
            // Distinct from the chrome icon camera's order to avoid an
            // ambiguous same-order-same-target warning; the two never draw
            // overlapping content so relative order doesn't affect output.
            order: 2,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        RenderLayers::layer(UI_ICON_CONTENT_RENDER_LAYER),
    ));
}

fn update_ui_icon_content_camera_viewport(
    windows: Query<&Window, With<PrimaryWindow>>,
    navbars: Query<(&ComputedNode, &UiGlobalTransform), With<Navbar>>,
    sidebars: Query<(&ComputedNode, &UiGlobalTransform), With<Sidebar>>,
    footers: Query<(&ComputedNode, &UiGlobalTransform), With<Footer>>,
    mut content_cameras: Query<&mut Camera, With<UiIconContentClipCamera>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    let window_width = window.width();
    let window_height = window.height();

    let top = navbars
        .iter()
        .map(|(computed, transform)| ui_logical_rect(computed, transform).bottom)
        .fold(0.0_f32, f32::max);
    let left = sidebars
        .iter()
        .map(|(computed, transform)| ui_logical_rect(computed, transform).right)
        .fold(0.0_f32, f32::max);
    let bottom = footers
        .iter()
        .map(|(computed, transform)| ui_logical_rect(computed, transform).top)
        .fold(window_height, f32::min);

    let left = left.clamp(0.0, window_width);
    let top = top.clamp(0.0, window_height);
    let right = window_width.max(left);
    let bottom = bottom.clamp(top, window_height);

    let scale_factor = window.scale_factor();
    let physical_pos_x = (left * scale_factor).round().max(0.0) as u32;
    let physical_pos_y = (top * scale_factor).round().max(0.0) as u32;
    let max_width = window.physical_width().saturating_sub(physical_pos_x).max(1);
    let max_height = window.physical_height().saturating_sub(physical_pos_y).max(1);
    let physical_size_x = (((right - left) * scale_factor).round().max(1.0) as u32).min(max_width);
    let physical_size_y = (((bottom - top) * scale_factor).round().max(1.0) as u32).min(max_height);

    for mut camera in &mut content_cameras {
        camera.viewport = Some(Viewport {
            physical_position: UVec2::new(physical_pos_x, physical_pos_y),
            physical_size: UVec2::new(physical_size_x, physical_size_y),
            depth: 0.0..1.0,
        });
    }
}

fn spawn_ui_svg_icon_proxies(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut svgs: ResMut<Assets<Svg>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut variants: ResMut<IconColorVariants>,
    parent_query: Query<&ChildOf>,
    content_roots: Query<Entity, With<ContentRoot>>,
    ui_icons: Query<(Entity, &IconNode), (Added<IconNode>, With<Node>)>,
) {
    let content_root = content_roots.iter().next();

    for (owner, icon_node) in &ui_icons {
        let handle = ensure_colored_svg_handle(
            icon_node,
            &mut variants,
            &asset_server,
            &mut svgs,
            &mut meshes,
        );

        let is_content =
            content_root.is_some_and(|root| is_descendant_of(owner, root, &parent_query));
        let render_layer = if is_content {
            UI_ICON_CONTENT_RENDER_LAYER
        } else {
            UI_ICON_RENDER_LAYER
        };

        let proxy_entity = commands
            .spawn((
                UiIconProxy { owner, content: is_content },
                Svg2d(handle),
                Transform::default(),
                GlobalTransform::default(),
                Visibility::Visible,
                RenderLayers::layer(render_layer),
            ))
            .id();

        if matches!(&icon_node.icon, Icon::Feather(name) if name == "loader") {
            commands
                .entity(proxy_entity)
                .insert((Spinner::speed(5.5), SpinnerAnchor::default()));
        }

        // Owner UI nodes can be despawned during page transitions before
        // deferred commands are applied; guard the link insert to avoid
        // command-queue panics on stale entities.
        commands.queue(move |world: &mut World| {
            if let Ok(mut owner_entity) = world.get_entity_mut(owner) {
                owner_entity.insert(UiIconProxyEntity(proxy_entity));
            } else {
                let _ = world.despawn(proxy_entity);
            }
        });
    }
}

fn sync_ui_svg_icon_proxies(
    ui_icons: Query<(
        &IconNode,
        &ComputedNode,
        &UiGlobalTransform,
        Option<&ComputedStackIndex>,
        &UiIconProxyEntity,
    )>,
    windows: Query<&Window, With<PrimaryWindow>>,
    content_cameras: Query<&Camera, With<UiIconContentClipCamera>>,
    mut proxies: Query<(
        Entity,
        &UiIconProxy,
        &mut Svg2d,
        &mut Transform,
        &mut Visibility,
        Option<&mut SpinnerAnchor>,
    )>,
    mut svgs: ResMut<Assets<Svg>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    mut variants: ResMut<IconColorVariants>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    // A camera's own viewport (when set) re-centers its 2D projection on
    // itself rather than on the full window, so each camera needs its own
    // reference center for the logical-to-world conversion below.
    let window_center = Vec2::new(window.width(), window.height()) * 0.5;
    let content_reference_center = content_cameras
        .iter()
        .find_map(|camera| camera.viewport.as_ref())
        .map(|viewport| {
            (viewport.physical_position.as_vec2() + viewport.physical_size.as_vec2() * 0.5)
                / window.scale_factor()
        })
        .unwrap_or(window_center);

    for (proxy_entity, proxy, mut svg2d, mut transform, mut visibility, anchor) in &mut proxies {
        let Ok((icon_node, computed, ui_transform, stack_index, linked_proxy)) =
            ui_icons.get(proxy.owner)
        else {
            *visibility = Visibility::Hidden;
            continue;
        };

        if linked_proxy.0 != proxy_entity {
            *visibility = Visibility::Hidden;
            continue;
        }

        let target_handle = ensure_colored_svg_handle(
            icon_node,
            &mut variants,
            &asset_server,
            &mut svgs,
            &mut meshes,
        );
        if svg2d.0 != target_handle {
            svg2d.0 = target_handle;
        }

        let viewport_center_physical = ui_transform.transform_point2(Vec2::ZERO);
        let viewport_center_logical = viewport_center_physical * computed.inverse_scale_factor();
        let half_slot = computed.size() * computed.inverse_scale_factor() * 0.5;
        let reference_center = if proxy.content {
            content_reference_center
        } else {
            window_center
        };

        // UI logical coordinates are top-left based; world coordinates are center based.
        let world_x = viewport_center_logical.x - reference_center.x;
        let world_y = reference_center.y - viewport_center_logical.y;
        let stack_z = stack_index
            .map(|stack| stack.0 as f32 * UI_ICON_STACK_Z_STEP + UI_ICON_LOCAL_Z_BIAS)
            .unwrap_or(UI_ICON_LOCAL_Z_BIAS);

        let center = Vec3::new(world_x, world_y, stack_z);

        if let Some(svg) = svgs.get(&svg2d.0) {
            let source_extent = svg.size.max_element().max(1.0);
            let target_extent = icon_node.size.max(1.0);
            let scale = target_extent / source_extent;
            transform.scale = Vec3::splat(scale);

            if let Some(mut anchor) = anchor {
                anchor.center = center;
                anchor.offset = Vec3::new(half_slot.x, -half_slot.y, 0.0);
            } else {
                // Svg2d origin is top-left, while UI slot positioning is center-based.
                transform.translation =
                    Vec3::new(world_x - half_slot.x, world_y + half_slot.y, stack_z);
            }
        }

        *visibility = if !computed.is_empty() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
struct UiLogicalRect {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

fn ui_logical_rect(computed: &ComputedNode, ui_transform: &UiGlobalTransform) -> UiLogicalRect {
    let center = ui_transform.transform_point2(Vec2::ZERO) * computed.inverse_scale_factor();
    let half = computed.size() * computed.inverse_scale_factor() * 0.5;

    UiLogicalRect {
        left: center.x - half.x,
        right: center.x + half.x,
        top: center.y - half.y,
        bottom: center.y + half.y,
    }
}

fn is_descendant_of(entity: Entity, ancestor: Entity, parents: &Query<&ChildOf>) -> bool {
    let mut current = entity;

    loop {
        if current == ancestor {
            return true;
        }

        let Ok(parent) = parents.get(current) else {
            return false;
        };

        current = parent.parent();
    }
}

fn despawn_orphan_ui_svg_icon_proxies(
    mut commands: Commands,
    ui_icons: Query<Entity, With<IconNode>>,
    proxies: Query<(Entity, &UiIconProxy)>,
) {
    for (entity, proxy) in &proxies {
        if ui_icons.get(proxy.owner).is_err() {
            commands
                .entity(entity)
                .despawn_related::<Children>()
                .despawn();
        }
    }
}

/// Loads the SVG associated with every FeatherIcon.
///
/// We keep the component separate from the actual SVG handle so
/// the rest of the application only needs to know the icon name.
fn load_feather_icons(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    icons: Query<(Entity, &IconNode), (Without<Svg2d>, Without<Node>)>,
) {
    for (entity, icon_node) in &icons {
        let svg: Handle<Svg> = asset_server.load(icon_node.icon.path());

        commands.entity(entity).insert(Svg2d(svg));
    }
}
