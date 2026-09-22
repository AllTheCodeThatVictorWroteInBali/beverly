use bevy::prelude::*;
use rfd::FileDialog;
use std::{
    path::PathBuf,
    sync::{
        Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread,
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

#[derive(Debug, Clone)]
pub struct SelectedFile {
    pub path: PathBuf,
    pub file_type: Option<FileType>,
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
            ),
        );
    }
}

// ============================================================================
// Spawn Helper
// ============================================================================

pub fn spawn_file_input(commands: &mut Commands, input: FileInput) -> Entity {
    commands.spawn((input, FileInputDragState::default())).id()
}

// ============================================================================
// Open Native File Picker
// ============================================================================

pub fn open_file_input(entity: Entity, input: &FileInput, channels: &FilePickerChannels) {
    let _ = channels.request_sender.send(PickerRequest {
        entity,
        input: input.clone(),
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
}

// ============================================================================
// Process Native Picker Results
// ============================================================================

fn process_file_picker_results(
    channels: Res<FilePickerChannels>,
    inputs: Query<&FileInput>,
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
    mut commands: Commands,
    mut drop_events: MessageReader<FileDragAndDrop>,
    inputs: Query<(Entity, &FileInput)>,
    mut drag_states: Query<&mut FileInputDragState>,
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
            FileDragAndDrop::DroppedFile {
                window: _,
                path_buf,
            } => {
                handle_dropped_file(
                    &mut commands,
                    path_buf.clone(),
                    &inputs,
                    &mut drag_states,
                    &mut selected_events,
                    &mut dropped_events,
                );
            }

            // ------------------------------------------------------------
            // File drag ended
            // ------------------------------------------------------------
            FileDragAndDrop::HoveredFileCanceled { window: _ } => {
                for (entity, _) in &inputs {
                    if let Ok(mut state) = drag_states.get_mut(entity) {
                        if state.dragging {
                            state.dragging = false;
                            state.file_count = 0;
                            state.accepted = false;

                            exited_events.write(FilesDragExited { entity });
                        }
                    }
                }
            }

            // ------------------------------------------------------------
            // File is hovering over the window
            // ------------------------------------------------------------
            FileDragAndDrop::HoveredFile {
                window: _,
                path_buf,
            } => {
                for (entity, input) in &inputs {
                    if !input.drag_and_drop {
                        continue;
                    }

                    let accepted = input.accepts(path_buf);

                    if let Ok(mut state) = drag_states.get_mut(entity) {
                        let was_dragging = state.dragging;

                        state.dragging = true;
                        state.file_count = 1;
                        state.accepted = accepted;

                        if !was_dragging {
                            entered_events.write(FilesDragEntered {
                                entity,
                                file_count: 1,
                            });
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Handle Dropped File
// ============================================================================

fn handle_dropped_file(
    _commands: &mut Commands,
    path: PathBuf,
    inputs: &Query<(Entity, &FileInput)>,
    drag_states: &mut Query<&mut FileInputDragState>,
    selected_events: &mut MessageWriter<FilesSelected>,
    dropped_events: &mut MessageWriter<FilesDropped>,
) {
    for (entity, input) in inputs.iter() {
        if !input.drag_and_drop {
            continue;
        }

        let accepted = input.accepts(&path);

        if let Ok(mut state) = drag_states.get_mut(entity) {
            state.dragging = false;
            state.file_count = 0;
            state.accepted = false;
        }

        if !accepted {
            continue;
        }

        let file = SelectedFile {
            file_type: detect_file_type(&path),
            path: path.clone(),
        };

        let files = vec![file];

        selected_events.write(FilesSelected {
            entity,
            files: files.clone(),
        });

        dropped_events.write(FilesDropped { entity, files });
    }
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
