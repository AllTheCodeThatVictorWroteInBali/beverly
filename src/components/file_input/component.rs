use crate::primitives::a11y;
use crate::primitives::semantic::{SemanticNode, SemanticRole};
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

impl Default for FileInput {
    fn default() -> Self {
        Self {
            accepted_types: vec![FileType::Image, FileType::Audio, FileType::Video],
            multiple: false,
            label: "Choose file".to_string(),
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
        .add_systems(
            Update,
            (
                process_file_picker_requests,
                process_file_picker_results,
                process_os_drag_and_drop,
                activate_file_input,
            ),
        );
    }
}

fn activate_file_input(
    inputs: Query<(Entity, &Interaction, &FileInput), Changed<Interaction>>,
    channels: Res<FilePickerChannels>,
    mut opened: MessageWriter<FileInputOpened>,
) {
    for (entity, interaction, input) in &inputs {
        if *interaction != Interaction::Pressed {
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
    let label = input.label.clone();
    commands
        .spawn((
            input,
            FileInputDragState::default(),
            FileInputSelectionState::default(),
            Button,
            a11y::TabIndex(0),
            SemanticNode::new(SemanticRole::Button).label(label.clone()),
            Node {
                min_width: px(180),
                min_height: px(42),
                padding: UiRect::axes(px(14), px(10)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
            crate::rendering::Surface::rounded_rect_fill(
                8.0,
                crate::rendering::Paint::solid(Color::NONE),
            ),
        ))
        .with_children(|parent| {
            parent.spawn(Text::new(label));
        })
        .id()
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
    let pointer = window.cursor_position()? * window.scale_factor();
    if !pointer.is_finite() {
        return None;
    }

    let candidates = targets
        .iter()
        .filter(|(_, input, node, transform, _, visibility, clip)| {
            input.drag_and_drop
                && input.accepts(path)
                && visibility.get()
                && !node.is_empty()
                && clip.is_none_or(|clip| !clip.clip.is_empty() && clip.clip.contains(pointer))
                && node.contains_point(**transform, pointer)
        })
        .map(|(entity, _, _, _, stack, _, _)| (entity, stack.0))
        .collect::<Vec<_>>();
    choose_topmost_file_input(candidates)
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
