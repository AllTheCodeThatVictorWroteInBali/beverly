use super::dropzone::{DropZonePlugin, ZoneCancel, spawn_zone_content, zone_node};
use crate::icons::{Icon, IconCommands, IconNode};
use crate::primitives::a11y;
use crate::primitives::semantic::{SemanticNode, SemanticRole};
use crate::rendering::prelude::Border;
use crate::rendering::{Paint, Surface};
use crate::theme::{ThemeMode, ThemeResource};
use bevy::prelude::*;
use bevy::ui::{CalculatedClip, ComputedStackIndex};
#[cfg(feature = "file_dialog")]
use rfd::FileDialog;
#[cfg(feature = "file_dialog")]
use std::thread;
use std::{
    path::PathBuf,
    sync::{
        Mutex,
        mpsc::{self, Receiver, Sender},
    },
};

// ============================================================================
// File Types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Image,
    Audio,
    Video,
}

impl FileType {
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            FileType::Image => &["png", "jpg", "jpeg", "gif", "webp", "bmp", "svg", "avif"],
            FileType::Audio => &["mp3", "wav", "ogg", "flac", "aac", "m4a", "opus"],
            FileType::Video => &["mp4", "mov", "mkv", "webm", "avi", "m4v", "mpeg"],
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            FileType::Image => "Images",
            FileType::Audio => "Audio",
            FileType::Video => "Video",
        }
    }

    pub fn accepts_extension(&self, extension: &str) -> bool {
        self.extensions()
            .iter()
            .any(|ext| ext.eq_ignore_ascii_case(extension))
    }
}

// ============================================================================
// FileInput Component
// ============================================================================

#[derive(Component, Debug, Clone)]
pub struct FileInput {
    pub variant: FileInputVariant,

    pub accepted_types: Vec<FileType>,

    /// Allow multiple files.
    pub multiple: bool,

    /// Text shown by the component.
    pub label: String,

    /// Optional explicit extension list.
    ///
    /// If empty, extensions are inferred from `accepted_types`.
    pub extensions: Vec<String>,

    /// Enable OS drag-and-drop.
    pub drag_and_drop: bool,
}

/// How a file input looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileInputVariant {
    /// A dashed drop area with a preview and progress bar for the chosen file.
    #[default]
    DropZone,
    /// A compact button.
    Button,
}

impl Default for FileInput {
    fn default() -> Self {
        Self {
            variant: FileInputVariant::DropZone,
            accepted_types: vec![FileType::Image, FileType::Audio, FileType::Video],
            multiple: false,
            label: "Drag and drop a file here".to_string(),
            extensions: Vec::new(),
            drag_and_drop: true,
        }
    }
}

impl FileInput {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            ..default()
        }
    }

    /// Shows a compact button instead of the default drop zone.
    pub fn button(mut self) -> Self {
        self.variant = FileInputVariant::Button;
        self
    }

    /// Shows the dashed drop zone (the default).
    pub fn drop_zone(mut self) -> Self {
        self.variant = FileInputVariant::DropZone;
        self
    }

    pub fn images(mut self) -> Self {
        self.accepted_types = vec![FileType::Image];
        self
    }

    pub fn audio(mut self) -> Self {
        self.accepted_types = vec![FileType::Audio];
        self
    }

    pub fn video(mut self) -> Self {
        self.accepted_types = vec![FileType::Video];
        self
    }

    pub fn media(mut self) -> Self {
        self.accepted_types = vec![FileType::Image, FileType::Audio, FileType::Video];
        self
    }

    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    pub fn extensions(mut self, extensions: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.extensions = extensions.into_iter().map(Into::into).collect();

        self
    }

    pub fn drag_and_drop(mut self, enabled: bool) -> Self {
        self.drag_and_drop = enabled;
        self
    }

    pub fn accepts(&self, path: &PathBuf) -> bool {
        let Some(extension) = path.extension().and_then(|e| e.to_str()) else {
            return false;
        };

        // Explicit extensions take precedence.
        if !self.extensions.is_empty() {
            return self
                .extensions
                .iter()
                .any(|ext| ext.trim_start_matches('.').eq_ignore_ascii_case(extension));
        }

        self.accepted_types
            .iter()
            .any(|file_type| file_type.accepts_extension(extension))
    }
}

// ============================================================================
// Drag State
// ============================================================================

#[derive(Component, Debug, Default)]
pub struct FileInputDragState {
    /// The OS currently has one or more files over this window.
    pub dragging: bool,

    /// Number of files currently being dragged over the window.
    pub file_count: usize,

    /// Whether the currently dragged files are accepted.
    pub accepted: bool,

    /// Window currently carrying this file drag.
    pub window: Option<Entity>,
}

// ============================================================================
// Events
// ============================================================================

#[derive(Message, Debug, Clone)]
pub struct FileInputOpened {
    pub entity: Entity,
}

#[derive(Message, Debug, Clone)]
pub struct FilesSelected {
    pub entity: Entity,
    pub files: Vec<SelectedFile>,
}

#[derive(Message, Debug, Clone)]
pub struct FileInputCancelled {
    pub entity: Entity,
}

#[derive(Message, Debug, Clone)]
pub struct FilesDragEntered {
    pub entity: Entity,
    pub file_count: usize,
}

#[derive(Message, Debug, Clone)]
pub struct FilesDragExited {
    pub entity: Entity,
}

/// The chosen file was removed with the X button.
#[derive(Message, Debug, Clone)]
pub struct FileInputCleared {
    pub entity: Entity,
}

#[derive(Message, Debug, Clone)]
pub struct FilesDropped {
    pub entity: Entity,
    pub files: Vec<SelectedFile>,
}

// ============================================================================
// Selected File
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedFile {
    pub path: PathBuf,
    pub file_type: Option<FileType>,
}

#[derive(Component, Debug, Clone, Default, PartialEq, Eq)]
pub struct FileInputSelectionState {
    pub files: Vec<SelectedFile>,
}

impl SelectedFile {
    pub fn filename(&self) -> Option<&str> {
        self.path.file_name()?.to_str()
    }

    pub fn extension(&self) -> Option<&str> {
        self.path.extension()?.to_str()
    }
}

// ============================================================================
// Picker Communication
// ============================================================================

struct PickerRequest {
    entity: Entity,
    #[cfg(feature = "file_dialog")]
    input: FileInput,
}

struct PickerResult {
    entity: Entity,
    files: Option<Vec<PathBuf>>,
}

#[derive(Resource)]
pub struct FilePickerChannels {
    requests: Mutex<Receiver<PickerRequest>>,
    request_sender: Sender<PickerRequest>,

    results: Mutex<Receiver<PickerResult>>,
    result_sender: Sender<PickerResult>,
}

// ============================================================================
// Plugin
// ============================================================================

pub struct FileInputPlugin;

impl Plugin for FileInputPlugin {
    fn build(&self, app: &mut App) {
        let (request_sender, requests) = mpsc::channel();
        let (result_sender, results) = mpsc::channel();

        app.insert_resource(FilePickerChannels {
            requests: Mutex::new(requests),
            request_sender,
            results: Mutex::new(results),
            result_sender,
        })
        .add_message::<FileInputOpened>()
        .add_message::<FilesSelected>()
        .add_message::<FileInputCancelled>()
        .add_message::<FilesDragEntered>()
        .add_message::<FilesDragExited>()
        .add_message::<FilesDropped>()
        .add_message::<FileInputCleared>()
        .add_plugins(DropZonePlugin)
        .add_systems(
            Update,
            (
                process_file_picker_requests,
                process_file_picker_results,
                process_os_drag_and_drop,
                activate_file_input,
                file_input_visual_system,
            ),
        );
    }
}

fn activate_file_input(
    inputs: Query<(Entity, &Interaction, &FileInput), Changed<Interaction>>,
    cancels: Query<(&ZoneCancel, &Interaction)>,
    channels: Res<FilePickerChannels>,
    mut opened: MessageWriter<FileInputOpened>,
) {
    for (entity, interaction, input) in &inputs {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // Pressing the X button must not also open the picker.
        if cancels
            .iter()
            .any(|(cancel, state)| cancel.owner == entity && *state != Interaction::None)
        {
            continue;
        }
        open_file_input(entity, input, &channels);
        opened.write(FileInputOpened { entity });
    }
}

// ============================================================================
// Spawn Helper
// ============================================================================

pub fn spawn_file_input(commands: &mut Commands, input: FileInput) -> Entity {
    let content_input = input.clone();
    let mut entity = commands.spawn(file_input_bundle(input));
    let id = entity.id();
    entity.with_children(|parent| spawn_file_input_content(parent, id, &content_input));
    id
}

/// Spawns a file input as a child of `parent`.
pub fn spawn_file_input_in(parent: &mut ChildSpawnerCommands, input: FileInput) -> Entity {
    let content_input = input.clone();
    let mut entity = parent.spawn(file_input_bundle(input));
    let id = entity.id();
    entity.with_children(|content| spawn_file_input_content(content, id, &content_input));
    id
}

fn file_input_bundle(input: FileInput) -> impl Bundle {
    let label = input.label.clone();
    let node = match input.variant {
        FileInputVariant::Button => Node {
            min_width: px(180),
            min_height: px(42),
            padding: UiRect::axes(px(14), px(10)),
            column_gap: px(8),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(px(8.0)),
            ..default()
        },
        FileInputVariant::DropZone => zone_node(),
    };
    (
        input,
        FileInputDragState::default(),
        FileInputSelectionState::default(),
        Button,
        a11y::TabIndex(0),
        SemanticNode::new(SemanticRole::Button).label(label),
        node,
        BackgroundColor(Color::NONE),
        Surface::rounded_rect_fill(8.0, Paint::solid(Color::NONE))
            .uniform_border(1.0, Paint::solid(Color::NONE)),
    )
}

fn spawn_file_input_content(parent: &mut ChildSpawnerCommands, owner: Entity, input: &FileInput) {
    if input.variant == FileInputVariant::DropZone {
        spawn_zone_content(parent, owner, input);
        return;
    }

    let label = input.label.clone();
    // Colors are applied by `file_input_visual_system` every frame.
    let icon = parent.spawn_icon_colored(Icon::feather("upload"), 16.0, Color::BLACK);
    parent
        .commands()
        .entity(icon)
        .insert(FileInputIcon { owner });
    parent.spawn((
        FileInputLabel { owner },
        Text::new(label),
        TextFont {
            font_size: FontSize::Px(14.0),
            ..default()
        },
        TextColor(Color::BLACK),
    ));
}

#[derive(Component, Clone, Copy)]
struct FileInputLabel {
    owner: Entity,
}

#[derive(Component, Clone, Copy)]
struct FileInputIcon {
    owner: Entity,
}

/// Same neutral palette as the default alert, button and avatar.
fn file_input_visual_system(
    theme: Option<Res<ThemeResource>>,
    inputs: Query<(
        &Interaction,
        &FileInputDragState,
        &FileInputSelectionState,
        &FileInput,
    )>,
    mut surfaces: Query<(Entity, &mut Surface), With<FileInput>>,
    mut labels: Query<(&FileInputLabel, &mut Text, &mut TextColor)>,
    mut icons: Query<(&FileInputIcon, &mut IconNode)>,
) {
    let light = theme.is_none_or(|theme| theme.current.mode == ThemeMode::Light);
    let gray = |v: u8| Color::srgb_u8(v, v, v);
    let (paper, ink, muted, border, hover, pressed, error) = if light {
        (
            gray(255),
            gray(0),
            gray(115),
            gray(212),
            gray(245),
            gray(229),
            Color::srgb_u8(220, 38, 38),
        )
    } else {
        (
            gray(23),
            gray(255),
            gray(163),
            gray(38),
            gray(38),
            gray(51),
            Color::srgb_u8(248, 113, 113),
        )
    };

    for (entity, mut surface) in &mut surfaces {
        let Ok((interaction, drag, _, input)) = inputs.get(entity) else {
            continue;
        };
        if input.variant != FileInputVariant::Button {
            continue;
        }

        let fill = if drag.dragging {
            hover
        } else {
            match interaction {
                Interaction::Pressed => pressed,
                Interaction::Hovered => hover,
                Interaction::None => paper,
            }
        };
        let outline = match (drag.dragging, drag.accepted) {
            (true, true) => ink,
            (true, false) => error,
            _ => border,
        };

        surface.fill = Paint::solid(fill);
        surface.border = Some(Border::new(1.0, Paint::solid(outline)));
    }

    for (label, mut text, mut color) in &mut labels {
        let Ok((_, _, selection, input)) = inputs.get(label.owner) else {
            continue;
        };

        let shown = match selection.files.as_slice() {
            [] => input.label.clone(),
            [file] => file.filename().unwrap_or(&input.label).to_string(),
            files => format!("{} files", files.len()),
        };
        if text.0 != shown {
            text.0 = shown;
        }

        let target = if selection.files.is_empty() {
            muted
        } else {
            ink
        };
        if color.0 != target {
            color.0 = target;
        }
    }

    for (icon, mut node) in &mut icons {
        if inputs.get(icon.owner).is_ok() && node.color != muted {
            node.color = muted;
        }
    }
}

// ============================================================================
// Open Native File Picker
// ============================================================================

pub fn open_file_input(entity: Entity, _input: &FileInput, channels: &FilePickerChannels) {
    let _ = channels.request_sender.send(PickerRequest {
        entity,
        #[cfg(feature = "file_dialog")]
        input: _input.clone(),
    });
}

// ============================================================================
// Process Native Picker Requests
// ============================================================================

fn process_file_picker_requests(channels: Res<FilePickerChannels>) {
    let Ok(requests) = channels.requests.lock() else {
        return;
    };

    while let Ok(request) = requests.try_recv() {
        #[cfg(feature = "file_dialog")]
        {
            let sender = channels.result_sender.clone();

            thread::spawn(move || {
                let mut dialog = FileDialog::new();

                let mut extensions = request.input.extensions.clone();

                if extensions.is_empty() {
                    for file_type in &request.input.accepted_types {
                        extensions.extend(file_type.extensions().iter().map(|ext| ext.to_string()));
                    }
                }

                extensions.sort();
                extensions.dedup();

                if !extensions.is_empty() {
                    let labels = request
                        .input
                        .accepted_types
                        .iter()
                        .map(|t| t.label())
                        .collect::<Vec<_>>()
                        .join(", ");

                    let extension_refs: Vec<&str> = extensions.iter().map(String::as_str).collect();

                    dialog = dialog.add_filter(&labels, &extension_refs);
                }

                let files = if request.input.multiple {
                    dialog.pick_files()
                } else {
                    dialog.pick_file().map(|file| vec![file])
                };

                let _ = sender.send(PickerResult {
                    entity: request.entity,
                    files,
                });
            });
        }

        #[cfg(not(feature = "file_dialog"))]
        {
            let _ = channels.result_sender.send(PickerResult {
                entity: request.entity,
                files: None,
            });
        }
    }
}

// ============================================================================
// Process Native Picker Results
// ============================================================================

fn process_file_picker_results(
    channels: Res<FilePickerChannels>,
    inputs: Query<&FileInput>,
    mut selections: Query<&mut FileInputSelectionState>,
    mut selected_events: MessageWriter<FilesSelected>,
    mut cancelled_events: MessageWriter<FileInputCancelled>,
) {
    let Ok(results) = channels.results.lock() else {
        return;
    };

    while let Ok(result) = results.try_recv() {
        let Ok(input) = inputs.get(result.entity) else {
            continue;
        };

        match result.files {
            Some(paths) if !paths.is_empty() => {
                let files = paths
                    .into_iter()
                    .filter(|path| input.accepts(path))
                    .map(|path| SelectedFile {
                        file_type: detect_file_type(&path),
                        path,
                    })
                    .collect::<Vec<_>>();

                if !files.is_empty() {
                    if let Ok(mut selection) = selections.get_mut(result.entity) {
                        selection.files = files.clone();
                    }
                    selected_events.write(FilesSelected {
                        entity: result.entity,
                        files,
                    });
                }
            }

            _ => {
                cancelled_events.write(FileInputCancelled {
                    entity: result.entity,
                });
            }
        }
    }
}

// ============================================================================
// OS Drag & Drop
// ============================================================================

fn process_os_drag_and_drop(
    mut drop_events: MessageReader<FileDragAndDrop>,
    inputs: Query<(Entity, &FileInput)>,
    windows: Query<&Window>,
    drop_targets: Query<(
        Entity,
        &FileInput,
        &ComputedNode,
        &UiGlobalTransform,
        &ComputedStackIndex,
        &InheritedVisibility,
        Option<&CalculatedClip>,
    )>,
    mut drag_states: Query<&mut FileInputDragState>,
    mut selections: Query<&mut FileInputSelectionState>,
    mut selected_events: MessageWriter<FilesSelected>,
    mut dropped_events: MessageWriter<FilesDropped>,
    mut entered_events: MessageWriter<FilesDragEntered>,
    mut exited_events: MessageWriter<FilesDragExited>,
) {
    for event in drop_events.read() {
        match event {
            // ------------------------------------------------------------
            // Files entered the Bevy window
            // ------------------------------------------------------------
            FileDragAndDrop::DroppedFile { window, path_buf } => {
                let target =
                    file_input_drop_target_for_window(*window, path_buf, &windows, &drop_targets);
                handle_dropped_file(
                    path_buf.clone(),
                    *window,
                    target,
                    &inputs,
                    &mut drag_states,
                    &mut selections,
                    &mut selected_events,
                    &mut dropped_events,
                    &mut exited_events,
                );
            }

            // ------------------------------------------------------------
            // File drag ended
            // ------------------------------------------------------------
            FileDragAndDrop::HoveredFileCanceled { window } => {
                for (entity, _) in &inputs {
                    if let Ok(mut state) = drag_states.get_mut(entity) {
                        if state.dragging && state.window == Some(*window) {
                            state.dragging = false;
                            state.file_count = 0;
                            state.accepted = false;
                            state.window = None;

                            exited_events.write(FilesDragExited { entity });
                        }
                    }
                }
            }

            // ------------------------------------------------------------
            // File is hovering over the window
            // ------------------------------------------------------------
            FileDragAndDrop::HoveredFile { window, path_buf } => {
                let target =
                    file_input_drop_target_for_window(*window, path_buf, &windows, &drop_targets);
                for (entity, input) in &inputs {
                    if let Ok(mut state) = drag_states.get_mut(entity) {
                        let is_target = target == Some(entity)
                            && input.drag_and_drop
                            && input.accepts(path_buf);
                        if is_target {
                            let was_target = state.dragging && state.window == Some(*window);
                            state.dragging = true;
                            state.file_count = 1;
                            state.accepted = true;
                            state.window = Some(*window);
                            if !was_target {
                                entered_events.write(FilesDragEntered {
                                    entity,
                                    file_count: 1,
                                });
                            }
                        } else if state.dragging && state.window == Some(*window) {
                            state.dragging = false;
                            state.file_count = 0;
                            state.accepted = false;
                            state.window = None;
                            exited_events.write(FilesDragExited { entity });
                        }
                    }
                }
            }
        }
    }
}

fn file_input_drop_target_for_window(
    window_entity: Entity,
    path: &PathBuf,
    windows: &Query<&Window>,
    targets: &Query<(
        Entity,
        &FileInput,
        &ComputedNode,
        &UiGlobalTransform,
        &ComputedStackIndex,
        &InheritedVisibility,
        Option<&CalculatedClip>,
    )>,
) -> Option<Entity> {
    let window = windows.get(window_entity).ok()?;
    let pointer = window
        .cursor_position()
        .map(|position| position * window.scale_factor())
        .filter(|pointer| pointer.is_finite());

    let eligible = targets
        .iter()
        .filter(|(_, input, node, _, _, visibility, _)| {
            input.drag_and_drop && input.accepts(path) && visibility.get() && !node.is_empty()
        })
        .collect::<Vec<_>>();

    let under_pointer = pointer.and_then(|pointer| {
        let hits = eligible
            .iter()
            .filter(|(_, _, node, transform, _, _, clip)| {
                clip.is_none_or(|clip| !clip.clip.is_empty() && clip.clip.contains(pointer))
                    && node.contains_point(**transform, pointer)
            })
            .map(|(entity, _, _, _, stack, _, _)| (*entity, stack.0))
            .collect::<Vec<_>>();
        choose_topmost_file_input(hits)
    });
    if under_pointer.is_some() {
        return under_pointer;
    }

    // The OS doesn't report the cursor while files are dragged in from another app
    // (it is stale or missing), so with a single candidate accept the drop anywhere.
    match eligible.as_slice() {
        [(entity, ..)] => Some(*entity),
        _ => None,
    }
}

fn choose_topmost_file_input(
    candidates: impl IntoIterator<Item = (Entity, u32)>,
) -> Option<Entity> {
    candidates
        .into_iter()
        .max_by(|(left_entity, left_stack), (right_entity, right_stack)| {
            left_stack
                .cmp(right_stack)
                .then_with(|| left_entity.to_bits().cmp(&right_entity.to_bits()))
        })
        .map(|(entity, _)| entity)
}

// ============================================================================
// Handle Dropped File
// ============================================================================

fn handle_dropped_file(
    path: PathBuf,
    window: Entity,
    target: Option<Entity>,
    inputs: &Query<(Entity, &FileInput)>,
    drag_states: &mut Query<&mut FileInputDragState>,
    selections: &mut Query<&mut FileInputSelectionState>,
    selected_events: &mut MessageWriter<FilesSelected>,
    dropped_events: &mut MessageWriter<FilesDropped>,
    exited_events: &mut MessageWriter<FilesDragExited>,
) {
    for (entity, _) in inputs.iter() {
        if let Ok(mut state) = drag_states.get_mut(entity) {
            if state.window == Some(window) {
                if state.dragging {
                    exited_events.write(FilesDragExited { entity });
                }
                state.dragging = false;
                state.file_count = 0;
                state.accepted = false;
                state.window = None;
            }
        }
    }

    let Some(entity) = target else {
        return;
    };
    let Ok((_, input)) = inputs.get(entity) else {
        return;
    };
    if !input.drag_and_drop || !input.accepts(&path) {
        return;
    }

    let file = SelectedFile {
        file_type: detect_file_type(&path),
        path,
    };

    let files = vec![file];

    if let Ok(mut selection) = selections.get_mut(entity) {
        if input.multiple {
            selection.files.push(files[0].clone());
        } else {
            selection.files = files.clone();
        }
    }

    selected_events.write(FilesSelected {
        entity,
        files: files.clone(),
    });

    dropped_events.write(FilesDropped { entity, files });
}

// ============================================================================
// File Type Detection
// ============================================================================

fn detect_file_type(path: &PathBuf) -> Option<FileType> {
    let extension = path.extension()?.to_str()?.to_lowercase();

    for file_type in [FileType::Image, FileType::Audio, FileType::Video] {
        if file_type.accepts_extension(&extension) {
            return Some(file_type);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_extensions_override_media_defaults() {
        let input = FileInput::new("Upload").images().extensions([".txt"]);
        assert!(input.accepts(&PathBuf::from("notes.TXT")));
        assert!(!input.accepts(&PathBuf::from("photo.png")));
    }

    #[test]
    fn media_defaults_accept_case_insensitive_extensions() {
        let input = FileInput::new("Upload").images();
        assert!(input.accepts(&PathBuf::from("photo.JpEg")));
        assert!(!input.accepts(&PathBuf::from("song.mp3")));
    }

    #[test]
    fn file_type_detection_rejects_unknown_and_missing_extensions() {
        assert_eq!(
            detect_file_type(&PathBuf::from("photo.PNG")),
            Some(FileType::Image)
        );
        assert_eq!(detect_file_type(&PathBuf::from("archive.zip")), None);
        assert_eq!(detect_file_type(&PathBuf::from("README")), None);
    }

    #[test]
    fn overlapping_file_inputs_choose_only_the_topmost_accepting_target() {
        let lower = Entity::from_bits(10);
        let upper = Entity::from_bits(20);

        assert_eq!(
            choose_topmost_file_input([(lower, 1), (upper, 2)]),
            Some(upper)
        );
        assert_eq!(choose_topmost_file_input([(lower, 1)]), Some(lower));
        assert_eq!(choose_topmost_file_input([]), None);
    }

    #[test]
    fn equal_stack_candidates_have_stable_entity_tiebreaking() {
        let low_id = Entity::from_bits(10);
        let high_id = Entity::from_bits(20);

        assert_eq!(
            choose_topmost_file_input([(high_id, 4), (low_id, 4)]),
            Some(high_id),
        );
    }
}
