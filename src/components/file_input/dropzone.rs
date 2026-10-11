//! Drop-zone variant of `FileInput`: a dashed outline, an image preview and a
//! loading progress bar beneath it.

use std::{
    fs::File,
    io::Read,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
    thread,
};

use bevy::asset::RenderAssetUsages;
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
use bevy::prelude::*;

use super::component::{
    FileInput, FileInputCleared, FileInputDragState, FileInputSelectionState, FileInputVariant,
    FileType, SelectedFile,
};
use crate::components::progress_bar::{ProgressBar, spawn_progress_bar_into};
use crate::icons::{Icon, IconCommands, IconNode};
use crate::primitives::a11y;
use crate::primitives::interaction::{
    InteractionAction, InteractionActionEvent, PointerCursorOnHover,
};
use crate::primitives::semantic::{SemanticNode, SemanticRole};
use crate::theme::{ThemeMode, ThemeResource};

/// Preview is fitted inside this box (logical px).
const PREVIEW_BOX: Vec2 = Vec2::new(200.0, 96.0);
/// Files larger than this are not read into memory for a preview.
const MAX_PREVIEW_BYTES: u64 = 64 * 1024 * 1024;
/// Fastest the bar is allowed to fill, so even instant loads read as progress.
const MAX_FILL_PER_SECOND: f32 = 1.4;

const DASH_LENGTH: f32 = 8.0;
const DASH_GAP: f32 = 6.0;
const DASH_THICKNESS: f32 = 2.0;
const DASH_RADIUS: f32 = 12.0;

/// Wobble played when the pointer enters the zone.
const WOBBLE_SECONDS: f32 = 0.55;
const WOBBLE_DEGREES: f32 = 1.6;
const WOBBLE_HERTZ: f32 = 6.0;
const WOBBLE_DECAY: f32 = 6.0;

// ============================================================================
// Palette
// ============================================================================

/// Same neutral palette as the default alert, button and avatar.
struct Palette {
    paper: Color,
    ink: Color,
    muted: Color,
    dash: Color,
    hover: Color,
    pressed: Color,
    error: Color,
}

fn palette(theme: Option<&ThemeResource>) -> Palette {
    let gray = |v: u8| Color::srgb_u8(v, v, v);
    if theme.is_none_or(|theme| theme.current.mode == ThemeMode::Light) {
        Palette {
            paper: gray(255),
            ink: gray(0),
            muted: gray(115),
            dash: gray(176),
            hover: gray(245),
            pressed: gray(229),
            error: Color::srgb_u8(220, 38, 38),
        }
    } else {
        Palette {
            paper: gray(23),
            ink: gray(255),
            muted: gray(163),
            dash: gray(82),
            hover: gray(38),
            pressed: gray(51),
            error: Color::srgb_u8(248, 113, 113),
        }
    }
}

// ============================================================================
// Parts
// ============================================================================

#[derive(Clone, Copy, PartialEq, Eq)]
enum ZonePartKind {
    Prompt,
    Preview,
    Image,
    Placeholder,
    Cancel,
}

/// The X button that removes the chosen file.
#[derive(Component, Clone, Copy)]
pub(super) struct ZoneCancel {
    pub(super) owner: Entity,
}

#[derive(Component)]
struct ZoneCancelIcon;

#[derive(Component, Clone, Copy)]
struct ZonePart {
    owner: Entity,
    kind: ZonePartKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ZoneTextKind {
    Title,
    Hint,
    Name,
    Status,
}

#[derive(Component, Clone, Copy)]
struct ZoneText {
    owner: Entity,
    kind: ZoneTextKind,
}

#[derive(Component)]
struct ZoneIcon;

#[derive(Component, Clone, Copy)]
struct ZoneBar {
    owner: Entity,
}

#[derive(Component, Clone, Copy)]
struct ZoneDashes {
    owner: Entity,
}

#[derive(Component, Default)]
struct DashLayout {
    size: Vec2,
}

#[derive(Component, Clone, Copy)]
struct ZoneDash {
    owner: Entity,
}

// ============================================================================
// Spawn
// ============================================================================

pub(super) fn zone_node() -> Node {
    Node {
        width: px(360),
        min_height: px(220),
        padding: UiRect::all(px(24)),
        row_gap: px(8),
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border_radius: BorderRadius::all(px(DASH_RADIUS)),
        ..default()
    }
}

pub(super) fn spawn_zone_content(
    parent: &mut ChildSpawnerCommands,
    owner: Entity,
    input: &FileInput,
) {
    // Colors are applied by the zone systems every frame.
    parent
        .commands()
        .entity(owner)
        .insert((PointerCursorOnHover, ZoneWobble::default()));
    let text_font = |size: f32| TextFont {
        font_size: FontSize::Px(size),
        ..default()
    };

    parent
        .spawn((
            ZonePart {
                owner,
                kind: ZonePartKind::Prompt,
            },
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(6),
                ..default()
            },
        ))
        .with_children(|prompt| {
            let icon = prompt.spawn_icon_colored(Icon::feather("upload"), 28.0, Color::BLACK);
            prompt.commands().entity(icon).insert(ZoneIcon);
            prompt.spawn((
                ZoneText {
                    owner,
                    kind: ZoneTextKind::Title,
                },
                Text::new(input.label.clone()),
                text_font(14.0),
                TextColor(Color::BLACK),
            ));
            prompt.spawn((
                ZoneText {
                    owner,
                    kind: ZoneTextKind::Hint,
                },
                Text::new("or click to browse"),
                text_font(13.0),
                TextColor(Color::BLACK),
            ));
        });

    parent
        .spawn((
            ZonePart {
                owner,
                kind: ZonePartKind::Preview,
            },
            Node {
                display: Display::None,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(10),
                ..default()
            },
        ))
        .with_children(|preview| {
            preview.spawn((
                ZonePart {
                    owner,
                    kind: ZonePartKind::Image,
                },
                Node {
                    display: Display::None,
                    border_radius: BorderRadius::all(px(8)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                ImageNode::default(),
            ));

            // Shown instead of the image when the file can't be previewed.
            preview
                .spawn((
                    ZonePart {
                        owner,
                        kind: ZonePartKind::Placeholder,
                    },
                    Node {
                        display: Display::None,
                        ..default()
                    },
                ))
                .with_children(|placeholder| {
                    let icon =
                        placeholder.spawn_icon_colored(Icon::feather("file"), 28.0, Color::BLACK);
                    placeholder.commands().entity(icon).insert(ZoneIcon);
                });

            preview.spawn((
                ZoneText {
                    owner,
                    kind: ZoneTextKind::Name,
                },
                Text::new(""),
                text_font(13.0),
                TextColor(Color::BLACK),
            ));

            let bar = spawn_progress_bar_into(
                preview,
                ProgressBar::new().size(220.0, 4.0).animation_speed(20.0),
            );
            preview.commands().entity(bar).insert(ZoneBar { owner });

            preview.spawn((
                ZoneText {
                    owner,
                    kind: ZoneTextKind::Status,
                },
                Text::new(""),
                text_font(12.0),
                TextColor(Color::BLACK),
            ));
        });

    // Last so the dashes draw over the zone's own content edges.
    parent.spawn((
        ZoneDashes { owner },
        DashLayout::default(),
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: percent(100),
            height: percent(100),
            ..default()
        },
    ));

    // After the dashes so it sits on top of them.
    parent
        .spawn((
            ZoneCancel { owner },
            PointerCursorOnHover,
            ZonePart {
                owner,
                kind: ZonePartKind::Cancel,
            },
            Button,
            a11y::TabIndex(0),
            SemanticNode::new(SemanticRole::Button).label("Remove file"),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                top: px(10),
                right: px(10),
                width: px(28),
                height: px(28),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(px(14)),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|cancel| {
            let icon = cancel.spawn_icon_colored(Icon::feather("x"), 16.0, Color::BLACK);
            cancel.commands().entity(icon).insert(ZoneCancelIcon);
        });
}

// ============================================================================
// Loading
// ============================================================================

#[derive(Default)]
struct LoadShared {
    /// Bytes read so far, in thousandths of the file.
    progress: AtomicU32,
    failed: AtomicBool,
    data: Mutex<Option<Vec<u8>>>,
}

struct Decoded {
    /// `None` when the file can't be shown as an image.
    image: Option<Handle<Image>>,
    size: Vec2,
}

#[derive(Component)]
struct FileInputLoad {
    shared: Arc<LoadShared>,
    file: SelectedFile,
    bytes: u64,
    /// What the bar shows: follows real progress, rate-limited.
    displayed: f32,
    decoded: Option<Decoded>,
}

impl FileInputLoad {
    fn start(file: SelectedFile) -> Self {
        let shared = Arc::new(LoadShared::default());
        let bytes = std::fs::metadata(&file.path).map(|m| m.len()).unwrap_or(0);

        let previewable = file.file_type == Some(FileType::Image) && bytes <= MAX_PREVIEW_BYTES;
        if previewable {
            let shared = shared.clone();
            let path = file.path.clone();
            thread::spawn(move || {
                let Ok(mut source) = File::open(&path) else {
                    shared.failed.store(true, Ordering::Release);
                    return;
                };
                let mut data = Vec::with_capacity(bytes as usize);
                let mut chunk = vec![0u8; 256 * 1024];
                loop {
                    match source.read(&mut chunk) {
                        Ok(0) => break,
                        Ok(read) => {
                            data.extend_from_slice(&chunk[..read]);
                            let permille = (data.len() as u64 * 1000) / bytes.max(1);
                            shared
                                .progress
                                .store(permille.min(1000) as u32, Ordering::Release);
                        }
                        Err(_) => {
                            shared.failed.store(true, Ordering::Release);
                            return;
                        }
                    }
                }
                shared.progress.store(1000, Ordering::Release);
                if let Ok(mut slot) = shared.data.lock() {
                    *slot = Some(data);
                }
            });
        } else {
            // Nothing to read for a preview; report the file as loaded.
            shared.progress.store(1000, Ordering::Release);
        }

        Self {
            shared,
            file,
            bytes,
            displayed: 0.0,
            decoded: None,
        }
    }

    fn done(&self) -> bool {
        self.displayed >= 0.999 && self.decoded.is_some()
    }
}

fn zone_start_load_system(
    mut commands: Commands,
    inputs: Query<(Entity, &FileInput, &FileInputSelectionState), Changed<FileInputSelectionState>>,
) {
    for (entity, input, selection) in &inputs {
        if input.variant != FileInputVariant::DropZone {
            continue;
        }

        match selection.files.last() {
            Some(file) => {
                commands
                    .entity(entity)
                    .insert(FileInputLoad::start(file.clone()));
            }
            None => {
                commands.entity(entity).remove::<FileInputLoad>();
            }
        }
    }
}

fn zone_load_progress_system(
    time: Res<Time>,
    images: Option<ResMut<Assets<Image>>>,
    mut loads: Query<&mut FileInputLoad>,
) {
    let Some(mut images) = images else {
        return;
    };

    for mut load in &mut loads {
        let real = load.shared.progress.load(Ordering::Acquire) as f32 / 1000.0;
        let step = time.delta_secs() * MAX_FILL_PER_SECOND;
        load.displayed = (load.displayed + step).min(real);

        if load.decoded.is_some() {
            continue;
        }

        let data = load
            .shared
            .data
            .lock()
            .ok()
            .and_then(|mut slot| slot.take());

        if let Some(data) = data {
            let extension = load.file.extension().unwrap_or("png").to_string();
            let decoded = Image::from_buffer(
                &data,
                ImageType::Extension(&extension),
                CompressedImageFormats::NONE,
                true,
                ImageSampler::Default,
                RenderAssetUsages::default(),
            );
            load.decoded = Some(match decoded {
                Ok(image) => Decoded {
                    size: Vec2::new(image.width() as f32, image.height() as f32),
                    image: Some(images.add(image)),
                },
                Err(_) => Decoded {
                    image: None,
                    size: Vec2::ZERO,
                },
            });
        } else if load.shared.failed.load(Ordering::Acquire)
            || load.file.file_type != Some(FileType::Image)
            || load.bytes > MAX_PREVIEW_BYTES
        {
            load.decoded = Some(Decoded {
                image: None,
                size: Vec2::ZERO,
            });
        }
    }
}

// ============================================================================
// Visuals
// ============================================================================

fn zone_surface_system(
    theme: Option<Res<ThemeResource>>,
    inputs: Query<(&FileInput, &Interaction, &FileInputDragState)>,
    mut surfaces: Query<(Entity, &mut crate::rendering::Surface)>,
) {
    let palette = palette(theme.as_deref());

    for (entity, mut surface) in &mut surfaces {
        let Ok((input, interaction, drag)) = inputs.get(entity) else {
            continue;
        };
        if input.variant != FileInputVariant::DropZone {
            continue;
        }

        let fill = if drag.dragging || *interaction != Interaction::None {
            palette.hover
        } else {
            palette.paper
        };
        surface.fill = crate::rendering::Paint::solid(fill);
        // The outline is drawn as dashes, not as a surface border.
        if surface.border.is_some() {
            surface.border = None;
        }
    }
}

fn zone_layout_system(
    inputs: Query<(&FileInputSelectionState, Option<&FileInputLoad>)>,
    mut parts: Query<(&ZonePart, &mut Node, Option<&mut ImageNode>)>,
) {
    for (part, mut node, image_node) in &mut parts {
        let Ok((selection, load)) = inputs.get(part.owner) else {
            continue;
        };
        let has_selection = !selection.files.is_empty();
        let decoded = load.and_then(|load| load.decoded.as_ref());

        match part.kind {
            ZonePartKind::Prompt => {
                node.display = if has_selection {
                    Display::None
                } else {
                    Display::Flex
                };
            }
            ZonePartKind::Preview => {
                node.display = if has_selection {
                    Display::Flex
                } else {
                    Display::None
                };
            }
            ZonePartKind::Image => {
                let Some(Decoded {
                    image: Some(handle),
                    size,
                }) = decoded
                else {
                    node.display = Display::None;
                    continue;
                };
                if let Some(mut image_node) = image_node {
                    if image_node.image != *handle {
                        image_node.image = handle.clone();
                    }
                }
                let scale = (PREVIEW_BOX.x / size.x.max(1.0))
                    .min(PREVIEW_BOX.y / size.y.max(1.0))
                    .min(1.5);
                node.display = Display::Flex;
                node.width = px(size.x * scale);
                node.height = px(size.y * scale);
            }
            ZonePartKind::Placeholder => {
                let unavailable = decoded.is_some_and(|decoded| decoded.image.is_none());
                node.display = if has_selection && unavailable {
                    Display::Flex
                } else {
                    Display::None
                };
            }
            ZonePartKind::Cancel => {
                node.display = if has_selection {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
    }
}

fn zone_text_system(
    theme: Option<Res<ThemeResource>>,
    inputs: Query<(
        &FileInput,
        &FileInputDragState,
        &FileInputSelectionState,
        Option<&FileInputLoad>,
    )>,
    mut texts: Query<(&ZoneText, &mut Text, &mut TextColor)>,
) {
    let palette = palette(theme.as_deref());

    for (zone_text, mut text, mut color) in &mut texts {
        let Ok((input, drag, selection, load)) = inputs.get(zone_text.owner) else {
            continue;
        };

        let (shown, target) = match zone_text.kind {
            ZoneTextKind::Title => (
                if drag.dragging {
                    "Release to upload".to_string()
                } else {
                    input.label.clone()
                },
                palette.ink,
            ),
            ZoneTextKind::Hint => ("or click to browse".to_string(), palette.muted),
            ZoneTextKind::Name => (
                selection
                    .files
                    .last()
                    .and_then(|file| file.filename())
                    .unwrap_or_default()
                    .to_string(),
                palette.ink,
            ),
            ZoneTextKind::Status => (
                match load {
                    Some(load) if load.done() => format!("Ready · {}", human_size(load.bytes)),
                    Some(load) => format!("Loading… {}%", (load.displayed * 100.0) as u32),
                    None => String::new(),
                },
                palette.muted,
            ),
        };

        if text.0 != shown {
            text.0 = shown;
        }
        if color.0 != target {
            color.0 = target;
        }
    }
}

fn zone_icon_system(
    theme: Option<Res<ThemeResource>>,
    mut icons: Query<&mut IconNode, With<ZoneIcon>>,
) {
    let muted = palette(theme.as_deref()).muted;
    for mut icon in &mut icons {
        if icon.color != muted {
            icon.color = muted;
        }
    }
}

fn zone_bar_system(
    inputs: Query<Option<&FileInputLoad>>,
    mut bars: Query<(&ZoneBar, &mut ProgressBar)>,
) {
    for (bar, mut progress_bar) in &mut bars {
        let target = inputs
            .get(bar.owner)
            .ok()
            .flatten()
            .map_or(0.0, |load| load.displayed);
        if progress_bar.progress != target {
            progress_bar.progress = target;
        }
    }
}

// ============================================================================
// Dashed outline
// ============================================================================

/// Rebuilds the dash segments whenever the zone's size changes.
fn zone_dash_build_system(
    mut commands: Commands,
    mut containers: Query<(
        Entity,
        &ZoneDashes,
        &ComputedNode,
        &mut DashLayout,
        Option<&Children>,
    )>,
) {
    for (entity, dashes, computed, mut layout, children) in &mut containers {
        let size = computed.size() * computed.inverse_scale_factor();
        if size.x < 1.0 || size.y < 1.0 || (size - layout.size).abs().max_element() < 0.5 {
            continue;
        }
        layout.size = size;

        for child in children.into_iter().flatten() {
            commands.entity(*child).despawn();
        }

        let owner = dashes.owner;
        commands.entity(entity).with_children(|parent| {
            for (x, y, width, height) in dash_rects(size) {
                parent.spawn((
                    ZoneDash { owner },
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(x),
                        top: px(y),
                        width: px(width),
                        height: px(height),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ));
            }
        });
    }
}

/// `(x, y, width, height)` of every dash around a rounded rectangle.
fn dash_rects(size: Vec2) -> Vec<(f32, f32, f32, f32)> {
    let (t, r) = (DASH_THICKNESS, DASH_RADIUS);
    let mut rects = Vec::new();

    // Dashes evenly spaced along a straight run, ends flush with the corners.
    let run = |length: f32| -> Vec<f32> {
        let count = ((length + DASH_GAP) / (DASH_LENGTH + DASH_GAP))
            .round()
            .max(1.0) as usize;
        if count == 1 {
            return vec![(length - DASH_LENGTH) / 2.0];
        }
        let gap = (length - count as f32 * DASH_LENGTH) / (count - 1) as f32;
        (0..count).map(|i| i as f32 * (DASH_LENGTH + gap)).collect()
    };

    for offset in run(size.x - 2.0 * r) {
        rects.push((r + offset, 0.0, DASH_LENGTH, t));
        rects.push((r + offset, size.y - t, DASH_LENGTH, t));
    }
    for offset in run(size.y - 2.0 * r) {
        rects.push((0.0, r + offset, t, DASH_LENGTH));
        rects.push((size.x - t, r + offset, t, DASH_LENGTH));
    }

    // Corners are approximated by dots along each quarter arc.
    let arc = r - t / 2.0;
    for (center, sign) in [
        (Vec2::new(r, r), Vec2::new(-1.0, -1.0)),
        (Vec2::new(size.x - r, r), Vec2::new(1.0, -1.0)),
        (Vec2::new(r, size.y - r), Vec2::new(-1.0, 1.0)),
        (Vec2::new(size.x - r, size.y - r), Vec2::new(1.0, 1.0)),
    ] {
        for degrees in [15.0_f32, 45.0, 75.0] {
            let angle = degrees.to_radians();
            let point = center + sign * Vec2::new(angle.cos(), angle.sin()) * arc;
            rects.push((point.x - t / 2.0, point.y - t / 2.0, t, t));
        }
    }

    rects
}

fn zone_dash_color_system(
    theme: Option<Res<ThemeResource>>,
    inputs: Query<&FileInputDragState>,
    mut dashes: Query<(&ZoneDash, &mut BackgroundColor)>,
) {
    let palette = palette(theme.as_deref());

    for (dash, mut color) in &mut dashes {
        let Ok(drag) = inputs.get(dash.owner) else {
            continue;
        };
        let target = match (drag.dragging, drag.accepted) {
            (true, true) => palette.ink,
            (true, false) => palette.error,
            _ => palette.dash,
        };
        if color.0 != target {
            color.0 = target;
        }
    }
}

fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

pub(super) struct DropZonePlugin;

#[derive(Component, Default)]
struct ZoneWobble {
    elapsed: f32,
    active: bool,
    was_hovered: bool,
}

/// A small damped rotation each time the pointer enters the zone.
fn zone_wobble_system(
    time: Res<Time>,
    mut zones: Query<(&Interaction, &mut ZoneWobble, &mut UiTransform)>,
) {
    for (interaction, mut wobble, mut transform) in &mut zones {
        let hovered = *interaction != Interaction::None;
        if hovered && !wobble.was_hovered && !wobble.active {
            wobble.active = true;
            wobble.elapsed = 0.0;
        }
        wobble.was_hovered = hovered;

        if !wobble.active {
            continue;
        }

        wobble.elapsed += time.delta_secs();
        let t = wobble.elapsed;
        if t >= WOBBLE_SECONDS {
            wobble.active = false;
            transform.rotation = Rot2::IDENTITY;
            continue;
        }

        let angle = WOBBLE_DEGREES.to_radians()
            * (std::f32::consts::TAU * WOBBLE_HERTZ * t).sin()
            * (-WOBBLE_DECAY * t).exp();
        transform.rotation = Rot2::radians(angle);
    }
}

/// Clears the chosen file when the X button is pressed or activated.
fn zone_cancel_system(
    cancels: Query<(&ZoneCancel, &Interaction), Changed<Interaction>>,
    cancel_lookup: Query<&ZoneCancel>,
    mut actions: MessageReader<InteractionActionEvent>,
    mut selections: Query<&mut FileInputSelectionState>,
    mut cleared: MessageWriter<FileInputCleared>,
) {
    let pressed = cancels
        .iter()
        .filter(|(_, interaction)| **interaction == Interaction::Pressed)
        .map(|(cancel, _)| cancel.owner);
    let activated = actions
        .read()
        .filter(|action| action.action == InteractionAction::Activate)
        .filter_map(|action| cancel_lookup.get(action.target).ok())
        .map(|cancel| cancel.owner)
        .collect::<Vec<_>>();

    for owner in pressed.chain(activated) {
        let Ok(mut selection) = selections.get_mut(owner) else {
            continue;
        };
        if selection.files.is_empty() {
            continue;
        }
        selection.files.clear();
        cleared.write(FileInputCleared { entity: owner });
    }
}

fn zone_cancel_visual_system(
    theme: Option<Res<ThemeResource>>,
    mut cancels: Query<(&Interaction, &mut BackgroundColor, &Children), With<ZoneCancel>>,
    mut icons: Query<&mut IconNode, With<ZoneCancelIcon>>,
) {
    let palette = palette(theme.as_deref());

    for (interaction, mut background, children) in &mut cancels {
        let hovered = *interaction != Interaction::None;
        let fill = if hovered {
            palette.pressed
        } else {
            Color::NONE
        };
        if background.0 != fill {
            background.0 = fill;
        }

        let target = if hovered { palette.ink } else { palette.muted };
        for child in children.iter() {
            if let Ok(mut icon) = icons.get_mut(child) {
                if icon.color != target {
                    icon.color = target;
                }
            }
        }
    }
}

impl Plugin for DropZonePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                zone_start_load_system,
                zone_cancel_system,
                zone_wobble_system,
                zone_cancel_visual_system,
                zone_load_progress_system,
                zone_surface_system,
                zone_layout_system,
                zone_text_system,
                zone_icon_system,
                zone_bar_system,
                zone_dash_build_system,
                zone_dash_color_system,
            )
                .chain(),
        );
    }
}
