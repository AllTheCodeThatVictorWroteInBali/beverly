#[cfg(not(target_arch = "wasm32"))]
#[path = "../llm.rs"]
mod llm;

#[cfg(target_arch = "wasm32")]
#[path = "../conversation.rs"]
mod conversation;
#[cfg(target_arch = "wasm32")]
#[path = "../web_platform.rs"]
mod web_platform;

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
#[cfg(not(target_arch = "wasm32"))]
use std::process::Command;
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::mpsc::{self, Receiver, Sender};
#[cfg(not(target_arch = "wasm32"))]
use std::thread;
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

use beverly::components::progress_bar::{ProgressBar, spawn_progress_bar_into};
use beverly::components::text::{FontWeight, TextRole, ThemedText, Typography};
use beverly::components::textarea::{
    Textarea, TextareaConfig, spawn_textarea, textarea_keyboard_system, textarea_visual_system,
};
use beverly::icons::IconCommands;
use beverly::prelude::*;
use bevy::prelude::*;
use bevy::ui_widgets::ScrollArea;
#[cfg(target_arch = "wasm32")]
use conversation::ChatModel;
#[cfg(not(target_arch = "wasm32"))]
use llm::{ChatModel, LocalLlama};
#[cfg(not(target_arch = "wasm32"))]
use sysinfo::System;

/// Messages sent from the background LLM thread back to the UI.
enum ChatEvent {
    LoadProgress(String, f32),
    Ready,
    FirstToken(u128),
    Token(String),
    Done,
    Error(String),
    #[cfg(target_arch = "wasm32")]
    ResponseText(String),
    #[cfg(target_arch = "wasm32")]
    GeneratedTokens(usize),
    #[cfg(target_arch = "wasm32")]
    WorkerMetrics(f64, u64),
    #[cfg(target_arch = "wasm32")]
    BackendFailure(String),
}

enum WorkerCommand {
    Load(ChatModel),
    Prompt(String),
}

#[derive(Resource)]
struct ChatChannel {
    #[cfg(not(target_arch = "wasm32"))]
    worker_tx: Sender<WorkerCommand>,
    #[cfg(target_arch = "wasm32")]
    worker_tx: web_platform::WorkerSender,
    #[cfg(not(target_arch = "wasm32"))]
    events: Mutex<Receiver<ChatEvent>>,
    #[cfg(target_arch = "wasm32")]
    events: web_platform::WorkerEvents,
    #[cfg(not(target_arch = "wasm32"))]
    stop: Arc<AtomicBool>,
    #[cfg(target_arch = "wasm32")]
    stop: web_platform::WorkerStop,
}

#[derive(Resource, Default)]
struct ChatState {
    model_ready: bool,
    generating: bool,
    streaming_entity: Option<Entity>,
    streaming_message: Option<Entity>,
    ttft_entity: Option<Entity>,
}

#[derive(Resource)]
struct ResourceMonitor {
    #[cfg(not(target_arch = "wasm32"))]
    system: System,
    timer: Timer,
    token_timer: Timer,
    generated_tokens: Arc<AtomicUsize>,
    generation_active: Arc<AtomicBool>,
    last_token_sample: Instant,
    last_token_total: usize,
    tokens_per_second: f64,
    used_gib: f64,
    total_gib: f64,
    cpu: f32,
    gpu: String,
    fps_window_frames: u64,
    fps_window_seconds: f64,
    frames_per_second: f64,
}

#[derive(Resource, Default)]
struct SelectedModel(Option<ChatModel>);

#[derive(Clone, Copy, PartialEq, Eq)]
enum DownloadPhase {
    Checking,
    Downloading,
    Downloaded,
    Present,
}

#[derive(Resource)]
struct LoadingProgress {
    download: DownloadPhase,
    download_fraction: f32,
    fraction: f32,
    status: String,
    initializing: bool,
    ready: bool,
    failed: bool,
}

#[derive(Component)]
struct ChatTextarea;

#[derive(Component)]
struct SubmitPromptButton;

#[derive(Component)]
struct SubmitIcon;

const CHAT_IDLE_BORDER: Color = Color::srgba(0.29, 0.34, 0.46, 0.48);

#[derive(Component)]
struct MessagesContainer;

#[derive(Component)]
struct ThinkingIndicator;

#[derive(Component)]
struct MetricsText;

#[derive(Component)]
struct MetricsModelText;

#[derive(Component)]
struct LoadingProgressBar;

#[derive(Component)]
struct DownloadProgressBar;

#[derive(Component)]
struct DownloadStatusText;

#[derive(Component)]
struct InitGroup;

#[derive(Component)]
struct LoadingStatusText;

#[derive(Component)]
struct LoadingGroup;

#[derive(Component)]
struct ModelPickerScreen;

#[derive(Component)]
struct ModelPickerButton(ChatModel);

#[derive(Component)]
struct AgentFooter;
#[derive(Component)]
struct AgentFooterContact;
#[derive(Component)]
struct AgentFooterMetrics;
#[derive(Component)]
struct AgentMetricsRow;
#[derive(Component)]
struct ChatScreenRoot;

pub fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                file_path: app_asset_root(),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Beverly Agent".into(),
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugins(BeverlyPlugin)
    .insert_resource(ThemeResource {
        current: light_theme(),
    })
    .init_resource::<ChatState>()
    .init_resource::<SelectedModel>()
    .insert_resource(LoadingProgress {
        download: DownloadPhase::Checking,
        download_fraction: 0.0,
        fraction: 0.0,
        status: "Initializing model".to_string(),
        initializing: false,
        ready: false,
        failed: false,
    })
    .insert_resource(ResourceMonitor {
        #[cfg(not(target_arch = "wasm32"))]
        system: System::new_all(),
        timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
        token_timer: Timer::new(Duration::from_millis(250), TimerMode::Repeating),
        generated_tokens: Arc::new(AtomicUsize::new(0)),
        generation_active: Arc::new(AtomicBool::new(false)),
        last_token_sample: Instant::now(),
        last_token_total: 0,
        tokens_per_second: 0.0,
        used_gib: 0.0,
        total_gib: 0.0,
        cpu: 0.0,
        gpu: "n/a".to_string(),
        fps_window_frames: 0,
        fps_window_seconds: 0.0,
        frames_per_second: 0.0,
    })
    .add_systems(Startup, (setup, spawn_llm_worker))
    .add_systems(
        Update,
        (
            handle_model_selection,
            reserve_textarea_button_space,
            update_submit_icon,
            chat_submit_system.before(textarea_keyboard_system),
            chat_poll_system.after(textarea_keyboard_system),
            sync_textarea_busy
                .after(chat_submit_system)
                .after(textarea_visual_system),
            animate_loading_progress,
            update_metrics,
        ),
    )
    .add_systems(
        PostUpdate,
        (size_model_picker_buttons, style_submit_prompt_button)
            .before(bevy::ui::UiSystems::Prepare),
    );
    #[cfg(not(target_arch = "wasm32"))]
    app.run();
    #[cfg(target_arch = "wasm32")]
    {
        use crate::browser::DemoAppExt;
        app.add_systems(Update, web_platform::fit_browser_ui);
        app.add_systems(
            Update,
            web_platform::read_mobile_input.before(chat_submit_system),
        );
        app.add_systems(
            PostUpdate,
            web_platform::position_mobile_input.after(bevy::ui::UiSystems::PostLayout),
        );
        app.run_demo();
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn app_asset_root() -> String {
    let bundle_assets = std::env::current_exe()
        .ok()
        .and_then(|executable| {
            executable
                .parent()
                .map(|directory| directory.join("../Resources/assets"))
        })
        .filter(|path| path.is_dir());

    bundle_assets
        .unwrap_or_else(|| PathBuf::from("assets"))
        .to_string_lossy()
        .into_owned()
}

#[cfg(target_arch = "wasm32")]
fn app_asset_root() -> String {
    "assets".into()
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2d, IsDefaultUiCamera));

    commands
        .spawn((
            ModelPickerScreen,
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(18.0),
                padding: UiRect::all(px(24.0)),
                ..default()
            },
            soft_gradient_background(),
        ))
        .with_children(|parent| {
            spawn_app_logo(parent, &asset_server);
            #[cfg(not(target_arch = "wasm32"))]
            spawn_themed_text(parent, "Choose a local model", TextRole::Heading, 28.0);
            #[cfg(target_arch = "wasm32")]
            spawn_themed_text(parent, "Beverly Agent", TextRole::Heading, 28.0);
            spawn_themed_text(
                parent,
                "Weights download on first selection and are cached for later launches.",
                TextRole::Muted,
                15.0,
            );

            parent
                .spawn(Node {
                    width: percent(100),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: px(20.0),
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                })
                .with_children(|row| {
                    spawn_model_button(row, ChatModel::TinyLlama, "cpu");
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                    spawn_model_button(row, ChatModel::SmolLm2_360M, "zap");
                    spawn_model_button(row, ChatModel::SmolLm2_1_7B, "cpu");
                    }
                });

            #[cfg(not(target_arch = "wasm32"))]
            spawn_themed_text(
                parent,
                "Supported LLMs: Hugging Face Llama-architecture models using safetensors. GGUF/Q8_0 and non-Llama models are not supported yet.",
                TextRole::Muted,
                12.0,
            );
            #[cfg(target_arch = "wasm32")]
            spawn_themed_text(parent, "TinyLlama Q4_0 · 640 MB download · CPU inference on your device", TextRole::Muted, 12.0);

            parent
                .spawn((
                    LoadingGroup,
                    Visibility::Hidden,
                    Node {
                        width: percent(100),
                        max_width: px(700.0),
                        min_height: px(160.0),
                        flex_shrink: 0.0,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        align_self: AlignSelf::Center,
                        row_gap: px(8.0),
                        margin: UiRect::top(px(12.0)),
                        ..default()
                    },
                ))
                .with_children(|loading| {
                    loading.spawn((
                        DownloadStatusText,
                        Text::new("Waiting for model selection"),
                        TextLayout::justify(Justify::Center),
                        ThemedText::new(TextRole::Muted).size(12.0),
                        Node {
                            width: percent(100),
                            min_height: px(36.0),
                            flex_shrink: 0.0,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                    ));
                    let download_bar = spawn_progress_bar_into(loading, loading_bar());
                    loading
                        .commands()
                        .entity(download_bar)
                        .insert((
                            DownloadProgressBar,
                            Node {
                                width: percent(100),
                                max_width: px(700.0),
                                height: px(10.0),
                                align_self: AlignSelf::Center,
                                border_radius: BorderRadius::all(px(5.0)),
                                overflow: Overflow::clip(),
                                ..default()
                            },
                        ));

                    loading
                        .spawn((
                            InitGroup,
                            Visibility::Hidden,
                            Node {
                                width: percent(100),
                                min_height: px(72.0),
                                flex_shrink: 0.0,
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                align_self: AlignSelf::Center,
                                row_gap: px(8.0),
                                margin: UiRect::top(px(12.0)),
                                ..default()
                            },
                        ))
                        .with_children(|init| {
                            init.spawn((
                                LoadingStatusText,
                                Text::new("Initializing model"),
                                TextLayout::justify(Justify::Center),
                                ThemedText::new(TextRole::Muted).size(12.0),
                                Node {
                                    width: percent(100),
                                    min_height: px(36.0),
                                    flex_shrink: 0.0,
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                            ));
                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                let init_bar = spawn_progress_bar_into(init, loading_bar());
                                init.commands().entity(init_bar).insert((
                                    LoadingProgressBar,
                                    Node {
                                        width: percent(100),
                                        max_width: px(700.0),
                                        height: px(10.0),
                                        align_self: AlignSelf::Center,
                                        border_radius: BorderRadius::all(px(5.0)),
                                        overflow: Overflow::clip(),
                                        ..default()
                                    },
                                ));
                            }
                            #[cfg(target_arch = "wasm32")]
                            init.spawn((
                                Dots::new(),
                                ThemedText::new(TextRole::Muted).size(14.0),
                                Node { width: px(40.0), height: px(24.0), flex_shrink: 0.0, align_self: AlignSelf::Center, ..default() },
                            ));
                        });
                });
        });
}

fn loading_bar() -> ProgressBar {
    ProgressBar::new()
        .size(700.0, 10.0)
        .progress(0.0)
        .background_color(Color::srgb(0.90, 0.91, 0.92))
        .fill_color(Color::srgb(0.62, 0.64, 0.67))
}

fn spawn_themed_text(
    parent: &mut ChildSpawnerCommands,
    value: impl Into<String>,
    role: TextRole,
    size: f32,
) -> Entity {
    parent
        .spawn((Text::new(value), ThemedText::new(role).size(size)))
        .id()
}

fn spawn_app_logo(parent: &mut ChildSpawnerCommands, asset_server: &AssetServer) {
    parent.spawn((
        ImageNode {
            image: asset_server.load("beverly_logo_black.png"),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            left: px(20.0),
            top: px(16.0),
            width: px(80.172),
            height: px(39.168),
            ..default()
        },
    ));
}

fn soft_gradient_background() -> Surface {
    Surface::rounded_rect_fill(
        0.0,
        Paint::linear(beverly::rendering::LinearGradient::new(
            Vec2::new(0.0, 0.36),
            Vec2::new(1.0, 0.64),
            vec![
                GradientStop::new(0.0, Color::srgb(0.985, 0.985, 0.995)),
                GradientStop::new(0.48, Color::srgb(0.995, 0.972, 0.980)),
                GradientStop::new(1.0, Color::srgb(0.925, 0.975, 0.995)),
            ],
        )),
    )
}

fn spawn_model_button(parent: &mut ChildSpawnerCommands, model: ChatModel, icon: &'static str) {
    parent.spawn((
        ModelPickerButton(model),
        BeverlyButton::light(model.name())
            .children([ButtonChild::icon(icon), ButtonChild::text(model.name())]),
        Node {
            width: px(190.0),
            height: px(190.0),
            flex_shrink: 0.0,
            ..default()
        },
    ));
}

fn size_model_picker_buttons(mut buttons: Query<&mut Node, With<ModelPickerButton>>) {
    for mut node in &mut buttons {
        node.width = px(190.0);
        node.height = px(190.0);
        node.flex_shrink = 0.0;
    }
}

fn handle_model_selection(
    mut selected_model: ResMut<SelectedModel>,
    mut loading: ResMut<LoadingProgress>,
    mut loading_visibility: Query<&mut Visibility, With<LoadingGroup>>,
    channel: Res<ChatChannel>,
    buttons: Query<(&Interaction, &ModelPickerButton)>,
) {
    if selected_model.0.is_some() {
        return;
    }

    let Some(model) = buttons.iter().find_map(|(interaction, button)| {
        (*interaction == Interaction::Pressed).then_some(button.0)
    }) else {
        return;
    };

    info!("Selected model: {}", model.name());
    selected_model.0 = Some(model);
    #[cfg(target_arch = "wasm32")]
    {
        loading.failed = false;
        loading.ready = false;
        loading.initializing = false;
        loading.download = DownloadPhase::Checking;
        loading.download_fraction = 0.0;
    }
    if let Ok(mut visibility) = loading_visibility.single_mut() {
        *visibility = Visibility::Inherited;
    }
    if channel.worker_tx.send(WorkerCommand::Load(model)).is_err() {
        warn!("The model worker has stopped running.");
    }
}

fn spawn_chat_screen(commands: &mut Commands, asset_server: &AssetServer, model: ChatModel) {
    commands
        .spawn((
            ChatScreenRoot,
            Node {
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .insert(soft_gradient_background())
        .with_children(|parent| {
            spawn_app_logo(parent, asset_server);
            parent
                .spawn(Node {
                    width: percent(100),
                    max_width: px(760.0),
                    height: percent(100),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(px(24.0)),
                    row_gap: px(14.0),
                    ..default()
                })
                .with_children(|panel| {
                    panel
                        .spawn((
                            MessagesContainer,
                            ScrollArea,
                            Node {
                                width: percent(100),
                                flex_grow: 1.0,
                                flex_direction: FlexDirection::Column,
                                overflow: Overflow::scroll_y(),
                                min_height: px(0.0),
                                row_gap: px(10.0),
                                padding: UiRect::bottom(px(8.0)),
                                ..default()
                            },
                        ))
                        .with_children(|messages| {
                            #[cfg(target_arch = "wasm32")]
                            messages.spawn((
                                web_platform::EmptyConversation,
                                Text::new("Ask TinyLlama a question."),
                                ThemedText::new(TextRole::Muted).size(16.0),
                            ));
                        });

                    // The submit button is a sibling, not a child, of the textarea:
                    // nesting a BeverlyButton inside the textarea's own Button/Surface
                    // entity hid its icon, and overwriting the textarea's Node wiped
                    // out spawn_textarea's own layout fields (padding/border/radius).
                    panel
                        .spawn(Node {
                            width: percent(100),
                            min_height: px(96.0),
                            flex_shrink: 0.0,
                            position_type: PositionType::Relative,
                            ..default()
                        })
                        .with_children(|wrapper| {
                            let config = TextareaConfig::new(
                                "Ask the local LLM something, then press Enter...",
                            )
                            .label("Chat with local LLM")
                            .height(96.0);
                            let textarea = spawn_textarea(wrapper, config);
                            wrapper.commands().entity(textarea).insert(ChatTextarea);

                            wrapper.spawn((
                                SubmitPromptButton,
                                BeverlyButton::dark("Send").children([ButtonChild::custom(
                                    |parent, _| {
                                        // ButtonChild::icon("send") rendered invisibly here; an explicit-color icon works.
                                        let icon = parent.spawn_icon_colored(
                                            beverly::icons::Icon::feather("send"),
                                            16.0,
                                            CHAT_IDLE_BORDER,
                                        );
                                        parent.commands().entity(icon).insert(SubmitIcon);
                                    },
                                )]),
                                Node {
                                    position_type: PositionType::Absolute,
                                    right: px(8.0),
                                    bottom: px(8.0),
                                    width: px(38.0),
                                    height: px(38.0),
                                    flex_shrink: 0.0,
                                    ..default()
                                },
                            ));
                        });

                    panel
                        .spawn((
                            AgentFooter,
                            Node {
                                width: percent(100),
                                row_gap: px(4.0),
                                padding: UiRect {
                                    top: px(8.0),
                                    ..default()
                                },
                                border: UiRect {
                                    top: px(1.0),
                                    ..default()
                                },
                                ..default()
                            },
                        ))
                        .with_children(|footer| {
                            #[cfg(target_arch = "wasm32")]
                            footer
                                .spawn(Node {
                                    flex_shrink: 0.0,
                                    ..default()
                                })
                                .with_children(|slot| {
                                    slot.spawn(
                                        BeverlyButton::standard("New chat")
                                            .sized(ButtonSize::Sm)
                                            .icon("message-square")
                                            .on("click", web_platform::new_chat),
                                    );
                                });
                            footer
                                .spawn((
                                    AgentFooterContact,
                                    Node {
                                        width: percent(50),
                                        flex_direction: FlexDirection::Column,
                                        row_gap: px(2.0),
                                        padding: UiRect::left(px(20.0)),
                                        ..default()
                                    },
                                ))
                                .with_children(|contact| {
                                    contact.spawn((
                                        Text::new("Want to build with Beverly?"),
                                        Typography::default()
                                            .with_size(12.0)
                                            .with_weight(FontWeight::BOLD)
                                            .with_color(Color::srgb(0.12, 0.14, 0.18)),
                                    ));
                                    spawn_themed_text(
                                        contact,
                                        "victor@beverlyui.com",
                                        TextRole::Muted,
                                        12.0,
                                    );
                                });
                            footer
                                .spawn((
                                    AgentFooterMetrics,
                                    Node {
                                        width: percent(50),
                                        flex_direction: FlexDirection::Column,
                                        ..default()
                                    },
                                ))
                                .with_children(|metrics| {
                                    metrics.spawn((
                                        MetricsModelText,
                                        Text::new(format!(
                                            "Model: {} · RAM: measuring · CPU: measuring",
                                            model.name()
                                        )),
                                        ThemedText::new(TextRole::Muted).size(12.0),
                                    ));
                                    metrics
                                        .spawn((
                                            AgentMetricsRow,
                                            Node {
                                                flex_direction: FlexDirection::Row,
                                                align_items: AlignItems::Center,
                                                column_gap: px(4.0),
                                                ..default()
                                            },
                                        ))
                                        .with_children(|telemetry| {
                                            telemetry.spawn((
                                                MetricsText,
                                                Text::new("GPU: measuring · 0 tok/s · 0 FPS ·"),
                                                ThemedText::new(TextRole::Muted).size(12.0),
                                            ));
                                            telemetry.spawn_icon_muted(
                                                beverly::icons::Icon::feather("wifi-off"),
                                                14.0,
                                            );
                                        });
                                });
                        });
                });
        });
}

fn update_submit_icon(
    chat_state: Res<ChatState>,
    mut icons: Query<&mut beverly::icons::IconNode, With<SubmitIcon>>,
) {
    let target = beverly::icons::Icon::feather(if chat_state.generating {
        "pause"
    } else {
        "send"
    });
    for mut icon in &mut icons {
        if icon.icon != target {
            icon.icon = target.clone();
        }
    }
}

fn sync_textarea_busy(
    chat_state: Res<ChatState>,
    mut textareas: Query<(&mut Textarea, &mut Surface), With<ChatTextarea>>,
) {
    for (mut textarea, mut surface) in &mut textareas {
        if textarea.busy != chat_state.generating {
            textarea.busy = chat_state.generating;
        }
        if let Some(border) = surface
            .border
            .as_mut()
            .filter(|_| !textarea.busy && !textarea.focused)
        {
            border.paint = Paint::solid(CHAT_IDLE_BORDER);
        }
    }
}

fn style_submit_prompt_button(
    mut buttons: Query<(&mut Node, &mut Surface), With<SubmitPromptButton>>,
    mut icons: Query<&mut beverly::icons::IconNode, With<SubmitIcon>>,
) {
    for (mut node, mut surface) in &mut buttons {
        node.position_type = PositionType::Absolute;
        node.right = px(8.0);
        node.bottom = px(8.0);
        node.width = px(38.0);
        node.height = px(38.0);
        node.padding = UiRect::all(px(0.0));
        node.border = UiRect::ZERO;
        node.border_radius = BorderRadius::MAX;
        node.flex_shrink = 0.0;
        surface.fill = Paint::solid(Color::NONE);
        surface.border = None;
    }
    for mut icon in &mut icons {
        if icon.color != CHAT_IDLE_BORDER {
            icon.color = CHAT_IDLE_BORDER;
        }
    }
}

/// Reserve space on the right edge of the textarea so typed text doesn't run
/// under the overlaid submit button. Patches only the `padding.right` field
/// instead of replacing the whole `Node`, so spawn_textarea's own layout
/// (border, radius, alignment) is left intact.
fn reserve_textarea_button_space(mut textareas: Query<&mut Node, Added<ChatTextarea>>) {
    for mut node in &mut textareas {
        node.padding.right = px(58.0);
    }
}
fn animate_loading_progress(
    time: Res<Time>,
    selected_model: Res<SelectedModel>,
    mut loading: ResMut<LoadingProgress>,
    mut download_status: Query<&mut Text, (With<DownloadStatusText>, Without<LoadingStatusText>)>,
    mut loading_status: Query<&mut Text, (With<LoadingStatusText>, Without<DownloadStatusText>)>,
    mut download_bars: Query<
        &mut ProgressBar,
        (With<DownloadProgressBar>, Without<LoadingProgressBar>),
    >,
    mut loading_bars: Query<
        &mut ProgressBar,
        (With<LoadingProgressBar>, Without<DownloadProgressBar>),
    >,
    mut init_group: Query<&mut Visibility, (With<InitGroup>, Without<DownloadProgressBar>)>,
    mut download_bar_nodes: Query<
        (&mut Node, &mut Visibility),
        (With<DownloadProgressBar>, Without<InitGroup>),
    >,
) {
    #[cfg(not(target_arch = "wasm32"))]
    if loading.initializing && !loading.ready {
        let remaining = 0.90 - loading.fraction;
        let step_fraction = 1.0 - (-0.33 * time.delta_secs()).exp();
        loading.fraction = (loading.fraction + remaining * step_fraction).min(0.8999);
    }

    let name = selected_model.0.map(ChatModel::name).unwrap_or("model");
    if let Ok(mut status) = download_status.single_mut() {
        let mut value = match loading.download {
            DownloadPhase::Checking => format!("Checking {name} files"),
            DownloadPhase::Downloading => format!(
                "Downloading {name}: {:.0}%",
                loading.download_fraction.clamp(0.0, 1.0) * 100.0
            ),
            DownloadPhase::Downloaded => format!("{name} Downloaded"),
            DownloadPhase::Present => format!("{name} Loaded"),
        };
        #[cfg(target_arch = "wasm32")]
        {
            if loading.failed {
                value = "Model is unavailable".into();
            } else if loading.download == DownloadPhase::Present {
                value = "Cached model assets verified".into();
            }
        }
        if status.0 != value {
            status.0 = value;
        }
    }
    if let Ok(mut bar) = download_bars.single_mut() {
        let progress = loading.download_fraction.clamp(0.0, 1.0);
        if bar.progress != progress {
            bar.progress = progress;
        }
    }

    let download_done = matches!(
        loading.download,
        DownloadPhase::Downloaded | DownloadPhase::Present
    );
    if let Ok((mut node, mut visibility)) = download_bar_nodes.single_mut() {
        let (display, target) = if download_done {
            (Display::None, Visibility::Hidden)
        } else {
            (Display::Flex, Visibility::Inherited)
        };
        if node.display != display {
            node.display = display;
        }
        if *visibility != target {
            *visibility = target;
        }
        if node.align_self != AlignSelf::Center {
            node.align_self = AlignSelf::Center;
        }
        if node.max_width != px(700.0) {
            node.max_width = px(700.0);
        }
    }

    let show_init = loading.failed
        || matches!(
            loading.download,
            DownloadPhase::Downloaded | DownloadPhase::Present
        );
    if let Ok(mut visibility) = init_group.single_mut() {
        let target = if show_init {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != target {
            *visibility = target;
        }
    }

    if let Ok(mut status) = loading_status.single_mut() {
        let value = if loading.failed {
            loading.status.clone()
        } else {
            #[cfg(not(target_arch = "wasm32"))]
            {
                format!(
                    "{}: {:.0}%",
                    loading.status,
                    loading.fraction.clamp(0.0, 1.0) * 100.0
                )
            }
            #[cfg(target_arch = "wasm32")]
            {
                loading.status.clone()
            }
        };
        if status.0 != value {
            status.0 = value;
        }
    }
    if let Ok(mut bar) = loading_bars.single_mut() {
        let progress = loading.fraction.clamp(0.0, 1.0);
        if bar.progress != progress {
            bar.progress = progress;
        }
    }
}

fn spawn_message_bubble(
    commands: &mut Commands,
    container: Entity,
    text: String,
    is_user: bool,
    show_ttft: bool,
) -> (Entity, Option<Entity>, Option<Entity>) {
    let fill = if is_user {
        Color::srgb(0.85, 0.90, 1.0)
    } else {
        Color::WHITE
    };
    let mut text_entity = Entity::PLACEHOLDER;
    let mut ttft_entity = None;
    let mut message_entity = None;

    commands.entity(container).with_children(|parent| {
        parent
            .spawn(Node {
                width: percent(100),
                justify_content: if is_user {
                    JustifyContent::FlexEnd
                } else {
                    JustifyContent::FlexStart
                },
                ..default()
            })
            .with_children(|row| {
                if is_user {
                    let mut bubble = row.spawn((
                        Node {
                            max_width: percent(80),
                            padding: UiRect::all(px(12.0)),
                            ..default()
                        },
                        Surface::rounded_rect_fill(8.0, Paint::solid(fill))
                            .uniform_border(1.0, Paint::solid(Color::srgb(0.87, 0.88, 0.90))),
                    ));
                    message_entity = Some(bubble.id());
                    bubble.with_children(|bubble| {
                        text_entity = bubble
                            .spawn((Text::new(text), ThemedText::new(TextRole::Body).size(16.0)))
                            .id();
                    });
                } else {
                    let mut response = row.spawn(Node {
                        width: percent(80),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(4.0),
                        ..default()
                    });
                    message_entity = Some(response.id());
                    response.with_children(|response| {
                        if show_ttft {
                            response
                                .spawn(Node {
                                    width: percent(100),
                                    justify_content: JustifyContent::FlexEnd,
                                    ..default()
                                })
                                .with_children(|metadata| {
                                    ttft_entity = Some(
                                        metadata
                                            .spawn((
                                                Text::new("— TTFT"),
                                                ThemedText::new(TextRole::Muted).size(12.0),
                                            ))
                                            .id(),
                                    );
                                });
                        }
                        response
                            .spawn((
                                Node {
                                    width: percent(100),
                                    padding: UiRect::all(px(12.0)),
                                    ..default()
                                },
                                Surface::rounded_rect_fill(8.0, Paint::solid(fill)).uniform_border(
                                    1.0,
                                    Paint::solid(Color::srgb(0.87, 0.88, 0.90)),
                                ),
                            ))
                            .with_children(|bubble| {
                                if text.is_empty() {
                                    bubble
                                        .spawn((
                                            ThinkingIndicator,
                                            Node {
                                                flex_direction: FlexDirection::Row,
                                                ..default()
                                            },
                                        ))
                                        .with_children(|row| {
                                            row.spawn((
                                                Text::new("Thinking"),
                                                ThemedText::new(TextRole::Muted).size(16.0),
                                            ));
                                            row.spawn((
                                                Dots::new(),
                                                ThemedText::new(TextRole::Muted).size(16.0),
                                            ));
                                        });
                                }
                                text_entity = bubble
                                    .spawn((
                                        Text::new(text),
                                        ThemedText::new(TextRole::Body).size(16.0),
                                    ))
                                    .id();
                            });
                    });
                }
            });
    });

    (text_entity, ttft_entity, message_entity)
}

fn spawn_system_notice(commands: &mut Commands, container: Entity, text: String) {
    commands.entity(container).with_children(|parent| {
        parent
            .spawn(Node {
                width: percent(80),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexStart,
                ..default()
            })
            .with_children(|row| {
                row.spawn((Text::new(text), ThemedText::new(TextRole::Muted).size(12.0)));
            });
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_llm_worker(mut commands: Commands, throughput: Res<ResourceMonitor>) {
    let (worker_tx, worker_rx) = mpsc::channel::<WorkerCommand>();
    let (event_tx, event_rx) = mpsc::channel::<ChatEvent>();
    let generated_tokens = Arc::clone(&throughput.generated_tokens);
    let generation_active = Arc::clone(&throughput.generation_active);
    let stop = Arc::new(AtomicBool::new(false));
    let worker_stop = Arc::clone(&stop);

    thread::spawn(move || {
        let mut llama: Option<LocalLlama> = None;
        while let Ok(command) = worker_rx.recv() {
            match command {
                WorkerCommand::Load(model) => {
                    let _ = event_tx.send(ChatEvent::LoadProgress(
                        format!("Preparing {}", model.name()),
                        0.0,
                    ));
                    let progress_tx = event_tx.clone();
                    match LocalLlama::init_with_model(false, model, move |stage, fraction| {
                        let _ = progress_tx.send(ChatEvent::LoadProgress(stage, fraction));
                    }) {
                        Ok(loaded) => {
                            llama = Some(loaded);
                            let _ = event_tx.send(ChatEvent::Ready);
                        }
                        Err(err) => {
                            let _ = event_tx
                                .send(ChatEvent::Error(format!("Failed to load model: {err}")));
                        }
                    }
                }
                WorkerCommand::Prompt(prompt) => {
                    let Some(llama) = llama.as_mut() else {
                        let _ = event_tx.send(ChatEvent::Error(
                            "Select a model before sending a prompt.".to_string(),
                        ));
                        continue;
                    };
                    let tx = event_tx.clone();
                    let started = Instant::now();
                    let mut sent_first_token = false;
                    worker_stop.store(false, Ordering::Release);
                    let result = llama.generate_streaming_until(
                        &prompt,
                        256,
                        0.7,
                        |token| {
                            if !sent_first_token {
                                sent_first_token = true;
                                let elapsed_ms = started.elapsed().as_millis();
                                let _ = tx.send(ChatEvent::FirstToken(elapsed_ms));
                            }
                            let _ = tx.send(ChatEvent::Token(token));
                        },
                        || {
                            generated_tokens.fetch_add(1, Ordering::Relaxed);
                        },
                        || worker_stop.load(Ordering::Acquire),
                    );
                    generation_active.store(false, Ordering::Release);
                    match result {
                        Ok(()) => {
                            let _ = event_tx.send(ChatEvent::Done);
                        }
                        Err(err) => {
                            let _ = event_tx
                                .send(ChatEvent::Error(format!("Generation failed: {err}")));
                        }
                    }
                }
            }
        }
    });

    commands.insert_resource(ChatChannel {
        worker_tx,
        events: Mutex::new(event_rx),
        stop,
    });
}

#[cfg(target_arch = "wasm32")]
fn spawn_llm_worker(mut commands: Commands) {
    web_platform::initialize();
    commands.insert_resource(ChatChannel {
        worker_tx: web_platform::WorkerSender,
        events: web_platform::WorkerEvents,
        stop: web_platform::WorkerStop,
    });
}

fn chat_submit_system(
    mut commands: Commands,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    channel: Option<Res<ChatChannel>>,
    mut throughput: ResMut<ResourceMonitor>,
    mut chat_state: ResMut<ChatState>,
    mut textareas: Query<&mut Textarea, With<ChatTextarea>>,
    submit_buttons: Query<&Interaction, (With<SubmitPromptButton>, Changed<Interaction>)>,
    containers: Query<Entity, With<MessagesContainer>>,
) {
    let Some(channel) = channel else { return };
    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let enter_pressed = !shift && keyboard.just_pressed(KeyCode::Enter);
    let button_pressed = submit_buttons
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);
    if !enter_pressed && !button_pressed {
        return;
    }
    if button_pressed && chat_state.generating {
        channel.stop.store(true, Ordering::Release);
        return;
    }

    let Ok(mut textarea) = textareas.single_mut() else {
        return;
    };
    if (enter_pressed && !textarea.focused) || chat_state.generating || !chat_state.model_ready {
        keyboard.clear_just_pressed(KeyCode::Enter);
        return;
    }

    let prompt = textarea.value().trim().to_string();
    if enter_pressed {
        keyboard.clear_just_pressed(KeyCode::Enter);
    }
    if prompt.is_empty() {
        return;
    }
    let Ok(container) = containers.single() else {
        return;
    };

    throughput.generated_tokens.store(0, Ordering::Relaxed);
    throughput.generation_active.store(true, Ordering::Release);
    throughput.last_token_total = 0;
    throughput.tokens_per_second = 0.0;
    throughput.last_token_sample = Instant::now();
    textarea.clear();
    let _ = spawn_message_bubble(&mut commands, container, prompt.clone(), true, false);
    let (response_entity, ttft_entity, message_entity) =
        spawn_message_bubble(&mut commands, container, String::new(), false, true);
    chat_state.streaming_entity = Some(response_entity);
    chat_state.streaming_message = message_entity;
    chat_state.ttft_entity = ttft_entity;
    chat_state.generating = true;
    if channel
        .worker_tx
        .send(WorkerCommand::Prompt(prompt))
        .is_err()
    {
        throughput.generation_active.store(false, Ordering::Release);
        chat_state.generating = false;
    }
}

fn chat_poll_system(
    channel: Option<Res<ChatChannel>>,
    monitor: Res<ResourceMonitor>,
    mut selected_model: ResMut<SelectedModel>,
    mut loading: ResMut<LoadingProgress>,
    mut chat_state: ResMut<ChatState>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut picker_visibility: Query<&mut Visibility, With<ModelPickerScreen>>,
    containers: Query<Entity, With<MessagesContainer>>,
    thinking: Query<Entity, With<ThinkingIndicator>>,
    mut texts: Query<&mut Text, Without<LoadingStatusText>>,
    mut scroll: Query<&mut ScrollPosition, With<MessagesContainer>>,
    screens: Query<Entity, With<ChatScreenRoot>>,
) {
    let Some(channel) = channel else { return };
    let Ok(events) = channel.events.lock() else {
        return;
    };
    let mut received_any = false;

    for event in events.try_iter() {
        received_any = true;
        match event {
            #[cfg(target_arch = "wasm32")]
            ChatEvent::BackendFailure(message) => {
                monitor.generation_active.store(false, Ordering::Release);
                chat_state.model_ready = false;
                chat_state.generating = false;
                chat_state.streaming_entity = None;
                chat_state.streaming_message = None;
                chat_state.ttft_entity = None;
                selected_model.0 = None;
                loading.failed = true;
                loading.initializing = false;
                loading.status = format!("{message} Reload TinyLlama to start a new conversation.");
                for screen in &screens {
                    commands.entity(screen).despawn();
                }
                if let Ok(mut visibility) = picker_visibility.single_mut() {
                    *visibility = Visibility::Inherited;
                }
            }
            ChatEvent::LoadProgress(stage, fraction) => {
                if stage.starts_with("Downloading") {
                    loading.download = DownloadPhase::Downloading;
                    loading.download_fraction = fraction.clamp(0.0, 1.0);
                } else if stage == "Download complete" {
                    loading.download = DownloadPhase::Downloaded;
                    loading.download_fraction = 1.0;
                } else if stage == "Model files present" {
                    loading.download = DownloadPhase::Present;
                    loading.download_fraction = 1.0;
                } else if stage == "Initializing model" {
                    loading.status = stage;
                    loading.fraction = 0.0;
                    loading.initializing = true;
                } else {
                    loading.download = DownloadPhase::Checking;
                }
            }
            ChatEvent::Ready => {
                chat_state.model_ready = true;
                loading.ready = true;
                loading.initializing = false;
                loading.status = "Model ready".to_string();
                loading.fraction = 1.0;
                if let Some(model) = selected_model.0 {
                    if let Ok(mut visibility) = picker_visibility.single_mut() {
                        *visibility = Visibility::Hidden;
                    }
                    spawn_chat_screen(&mut commands, &asset_server, model);
                }
            }
            ChatEvent::Token(token) => {
                if let Some(entity) = chat_state.streaming_entity {
                    if let Ok(mut text) = texts.get_mut(entity) {
                        text.0.push_str(&token);
                    }
                }
            }
            #[cfg(target_arch = "wasm32")]
            ChatEvent::ResponseText(response) => {
                if let Some(entity) = chat_state.streaming_entity {
                    if let Ok(mut text) = texts.get_mut(entity) {
                        text.0 = response;
                    }
                }
            }
            #[cfg(target_arch = "wasm32")]
            ChatEvent::GeneratedTokens(tokens) => {
                monitor.generated_tokens.store(tokens, Ordering::Relaxed);
            }
            #[cfg(target_arch = "wasm32")]
            ChatEvent::WorkerMetrics(rate, memory) => {
                commands.queue(move |world: &mut World| {
                    let mut monitor = world.resource_mut::<ResourceMonitor>();
                    monitor.tokens_per_second = rate;
                    monitor.used_gib = memory as f64 / 1_073_741_824.0;
                });
            }
            ChatEvent::FirstToken(elapsed_ms) => {
                for indicator in &thinking {
                    commands.entity(indicator).despawn();
                }
                if let Some(entity) = chat_state.ttft_entity {
                    if let Ok(mut text) = texts.get_mut(entity) {
                        text.0 = format_ttft(elapsed_ms);
                    }
                }
            }
            ChatEvent::Done => {
                for indicator in &thinking {
                    commands.entity(indicator).despawn();
                }
                monitor.generation_active.store(false, Ordering::Release);
                chat_state.streaming_entity = None;
                chat_state.streaming_message = None;
                chat_state.ttft_entity = None;
                chat_state.generating = false;
            }
            ChatEvent::Error(err) => {
                for indicator in &thinking {
                    commands.entity(indicator).despawn();
                }
                monitor.generation_active.store(false, Ordering::Release);
                if !chat_state.model_ready {
                    loading.status = "Model loading failed".to_string();
                    loading.initializing = false;
                    loading.failed = true;
                    #[cfg(target_arch = "wasm32")]
                    {
                        loading.status = format!("{err} Select TinyLlama to retry.");
                        selected_model.0 = None;
                    }
                }
                if let Some(entity) = chat_state.streaming_entity.take() {
                    if let Some(message) = chat_state.streaming_message.take() {
                        commands.entity(message).despawn();
                    }
                    let _ = entity;
                }
                if let Ok(container) = containers.single() {
                    spawn_system_notice(&mut commands, container, err);
                }
                chat_state.ttft_entity = None;
                chat_state.generating = false;
            }
        }
    }

    if received_any {
        if let Ok(mut position) = scroll.single_mut() {
            position.y = f32::MAX;
        }
    }
}

fn format_ttft(elapsed_ms: u128) -> String {
    if elapsed_ms > 1000 {
        format!("{:.2}s TTFT", elapsed_ms as f64 / 1000.0)
    } else {
        format!("{elapsed_ms}ms TTFT")
    }
}

#[cfg(test)]
mod ttft_tests {
    use super::format_ttft;

    #[test]
    fn milliseconds_at_or_below_one_second() {
        assert_eq!(format_ttft(0), "0ms TTFT");
        assert_eq!(format_ttft(850), "850ms TTFT");
        assert_eq!(format_ttft(1000), "1000ms TTFT");
    }

    #[test]
    fn seconds_above_one_second() {
        assert_eq!(format_ttft(1001), "1.00s TTFT");
        assert_eq!(format_ttft(1234), "1.23s TTFT");
        assert_eq!(format_ttft(62347), "62.35s TTFT");
    }
}

#[cfg(test)]
mod submit_style_tests {
    use super::*;
    use beverly::icons::{Icon, IconNode};

    #[test]
    fn send_and_pause_are_borderless_and_match_idle_input_border() {
        let mut app = App::new();
        app.init_resource::<ChatState>().add_systems(
            Update,
            (update_submit_icon, style_submit_prompt_button).chain(),
        );
        let button = app
            .world_mut()
            .spawn((
                SubmitPromptButton,
                Node {
                    border: UiRect::all(px(1.0)),
                    ..default()
                },
                Surface::rounded_rect_fill(8.0, Color::BLACK).uniform_border(1.0, Color::WHITE),
            ))
            .id();
        let icon = app
            .world_mut()
            .spawn((SubmitIcon, IconNode::new(Icon::feather("send"))))
            .id();

        for (generating, name) in [(false, "send"), (true, "pause")] {
            app.world_mut().resource_mut::<ChatState>().generating = generating;
            app.update();
            let surface = app.world().get::<Surface>(button).unwrap();
            assert_eq!(surface.fill, Paint::solid(Color::NONE));
            assert!(surface.border.is_none());
            let node = app.world().get::<Node>(button).unwrap();
            assert_eq!(node.border, UiRect::ZERO);
            assert_eq!(node.width, px(38.0));
            assert_eq!(node.height, px(38.0));
            let icon = app.world().get::<IconNode>(icon).unwrap();
            assert_eq!(icon.icon, Icon::feather(name));
            assert_eq!(icon.color, CHAT_IDLE_BORDER);
        }
    }
}

#[cfg(test)]
mod loading_layout_tests {
    use super::*;

    #[test]
    fn download_status_and_progress_bar_are_centered_and_screen_bounded() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.init_asset::<Image>();
        app.add_systems(Startup, setup);
        app.update();

        let loading_node = app
            .world_mut()
            .query_filtered::<&Node, With<LoadingGroup>>()
            .single(app.world())
            .unwrap();
        assert_eq!(loading_node.align_items, AlignItems::Center);
        assert_eq!(loading_node.width, percent(100.0));
        assert_eq!(loading_node.max_width, px(700.0));

        let (status_text, status_layout, status_node) = app
            .world_mut()
            .query_filtered::<(&Text, &TextLayout, &Node), With<DownloadStatusText>>()
            .single(app.world())
            .unwrap();
        assert_eq!(status_text.0, "Waiting for model selection");
        assert_eq!(status_layout.justify, Justify::Center);
        assert_eq!(status_node.justify_content, JustifyContent::Center);
        assert_eq!(status_node.align_items, AlignItems::Center);

        let bar_node = app
            .world_mut()
            .query_filtered::<&Node, With<DownloadProgressBar>>()
            .single(app.world())
            .unwrap();
        assert_eq!(bar_node.width, percent(100.0));
        assert_eq!(bar_node.max_width, px(700.0));
    }
}

fn update_metrics(
    time: Res<Time>,
    mut monitor: ResMut<ResourceMonitor>,
    selected_model: Res<SelectedModel>,
    mut metrics_text: Query<&mut Text, (With<MetricsText>, Without<MetricsModelText>)>,
    mut metrics_model_text: Query<&mut Text, (With<MetricsModelText>, Without<MetricsText>)>,
) {
    monitor.fps_window_frames += 1;
    monitor.fps_window_seconds += time.delta_secs_f64();
    monitor.timer.tick(time.delta());
    monitor.token_timer.tick(time.delta());
    let system_metrics_due = monitor.timer.just_finished();
    let token_metrics_due = monitor.token_timer.just_finished();
    if !system_metrics_due && !token_metrics_due {
        return;
    }

    if system_metrics_due {
        if monitor.fps_window_seconds > 0.0 {
            monitor.frames_per_second =
                monitor.fps_window_frames as f64 / monitor.fps_window_seconds;
        }
        monitor.fps_window_frames = 0;
        monitor.fps_window_seconds = 0.0;
        #[cfg(not(target_arch = "wasm32"))]
        {
            monitor.system.refresh_memory();
            monitor.system.refresh_cpu_usage();
            monitor.used_gib = monitor.system.used_memory() as f64 / 1_073_741_824.0;
            monitor.total_gib = monitor.system.total_memory() as f64 / 1_073_741_824.0;
            monitor.cpu = monitor.system.global_cpu_usage();
            monitor.gpu = read_gpu_utilization()
                .map(|usage| format!("{usage}%"))
                .unwrap_or_else(|| "n/a".to_string());
        }
    }

    if token_metrics_due {
        let now = Instant::now();
        let total = monitor.generated_tokens.load(Ordering::Relaxed);
        if monitor.generation_active.load(Ordering::Acquire) {
            let elapsed = now.duration_since(monitor.last_token_sample).as_secs_f64();
            if elapsed > 0.0 {
                monitor.tokens_per_second =
                    total.saturating_sub(monitor.last_token_total) as f64 / elapsed;
            }
        } else {
            #[cfg(not(target_arch = "wasm32"))]
            {
                monitor.tokens_per_second = 0.0;
            }
            monitor.last_token_total = total;
        }
        monitor.last_token_total = total;
        monitor.last_token_sample = now;
    }

    let Some(model_name) = selected_model.0.map(ChatModel::name) else {
        return;
    };

    if let Ok(mut text) = metrics_model_text.single_mut() {
        #[cfg(not(target_arch = "wasm32"))]
        {
            text.0 = format!(
                "Model: {model_name} · RAM: {:.1}/{:.1} GB · CPU: {:.0}%",
                monitor.used_gib, monitor.total_gib, monitor.cpu,
            );
        }
        #[cfg(target_arch = "wasm32")]
        {
            text.0 = format!("{model_name} · Q4_0 · {:.2} GiB Wasm", monitor.used_gib);
        }
    }
    if let Ok(mut text) = metrics_text.single_mut() {
        #[cfg(not(target_arch = "wasm32"))]
        {
            text.0 = format!(
                "GPU: {} · {:.0} tok/s · {:.0} FPS ·",
                monitor.gpu, monitor.tokens_per_second, monitor.frames_per_second,
            );
        }
        #[cfg(target_arch = "wasm32")]
        {
            text.0 = format!(
                "{:.2} tok/s · {:.0} FPS",
                monitor.tokens_per_second, monitor.frames_per_second
            );
        }
    }
}

/// Apple Silicon exposes this unprivileged GPU percentage through IORegistry.
#[cfg(not(target_arch = "wasm32"))]
fn read_gpu_utilization() -> Option<u8> {
    let output = Command::new("ioreg")
        .args(["-r", "-d", "1", "-w", "0", "-c", "AGXAccelerator"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let output = String::from_utf8_lossy(&output.stdout);
    let (_, value) = output.split_once("\"Device Utilization %\"")?;
    let value = value.trim_start().strip_prefix('=')?.trim_start();
    let digits = value
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>();
    digits.parse().ok()
}
