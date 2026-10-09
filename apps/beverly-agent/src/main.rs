mod llm;

use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use beverly::components::text::{TextRole, ThemedText};
use beverly::components::textarea::{
    Textarea, TextareaConfig, spawn_textarea, textarea_keyboard_system,
};
use beverly::prelude::*;
use bevy::prelude::*;
use bevy::ui_widgets::ScrollArea;

use llm::LocalLlama;

/// Messages sent from the background LLM thread back to the UI.
enum ChatEvent {
    Status(String),
    Token(String),
    Done,
    Error(String),
}

/// Channel pair wiring the UI to the background LLM worker thread.
#[derive(Resource)]
struct ChatChannel {
    prompt_tx: Sender<String>,
    events: Mutex<Receiver<ChatEvent>>,
}

#[derive(Resource, Default)]
struct ChatState {
    generating: bool,
    /// The text entity of the assistant bubble currently being streamed into.
    streaming_entity: Option<Entity>,
}

#[derive(Component)]
struct ChatTextarea;

/// Marks the scrollable column that message bubbles are appended to.
#[derive(Component)]
struct MessagesContainer;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: light_theme(),
        })
        .init_resource::<ChatState>()
        .add_systems(Startup, (setup, spawn_llm_worker))
        .add_systems(
            Update,
            (
                chat_submit_system.before(textarea_keyboard_system),
                chat_poll_system.after(textarea_keyboard_system),
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands
        .spawn(Node {
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(Node {
                    width: percent(100),
                    max_width: px(760.0),
                    height: percent(100),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(px(24.0)),
                    row_gap: px(16.0),
                    ..default()
                })
                .with_children(|panel| {
                    panel.spawn((
                        MessagesContainer,
                        // Trackpad/mouse-wheel scrolling: `ScrollArea` is bevy_ui_widgets'
                        // hover-aware scroll handler (already active via DefaultPlugins'
                        // UiWidgetsPlugins) and requires `ScrollPosition` itself.
                        ScrollArea,
                        Node {
                            width: percent(100),
                            flex_grow: 1.0,
                            flex_direction: FlexDirection::Column,
                            overflow: Overflow::scroll_y(),
                            row_gap: px(10.0),
                            padding: UiRect::bottom(px(8.0)),
                            ..default()
                        },
                    ));

                    let config =
                        TextareaConfig::new("Ask the local LLM something, then press Enter...")
                            .label("Chat with local LLM")
                            .height(96.0);

                    let textarea = spawn_textarea(panel, config);
                    panel.commands().entity(textarea).insert((
                        ChatTextarea,
                        Node {
                            width: percent(100),
                            min_height: px(96.0),
                            flex_shrink: 0.0,
                            ..default()
                        },
                    ));
                });
        });
}

/// Spawns a right-aligned (user) or left-aligned (assistant) chat bubble as a
/// child of the messages container, returning the entity of its `Text`.
fn spawn_message_bubble(
    commands: &mut Commands,
    container: Entity,
    text: String,
    is_user: bool,
) -> Entity {
    let bubble_fill = if is_user {
        Color::srgb(0.85, 0.90, 1.0)
    } else {
        Color::srgb(0.93, 0.93, 0.92)
    };

    let mut text_entity = Entity::PLACEHOLDER;

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
                row.spawn((
                    Node {
                        max_width: percent(80),
                        padding: UiRect::all(px(12.0)),
                        ..default()
                    },
                    Surface::rounded_rect_fill(12.0, Paint::solid(bubble_fill)),
                ))
                .with_children(|bubble| {
                    text_entity = bubble
                        .spawn((
                            ThemedText::new(TextRole::Body),
                            Text::new(text),
                            TextFont {
                                font_size: FontSize::Px(16.0),
                                ..default()
                            },
                            TextColor(Color::srgb(0.12, 0.12, 0.14)),
                        ))
                        .id();
                });
            });
    });

    text_entity
}

/// Spawns the LLM worker on a background OS thread and wires up the channels
/// used to send prompts in and stream generated tokens back out.
fn spawn_llm_worker(mut commands: Commands) {
    let (prompt_tx, prompt_rx) = mpsc::channel::<String>();
    let (event_tx, event_rx) = mpsc::channel::<ChatEvent>();

    thread::spawn(move || {
        let _ = event_tx.send(ChatEvent::Status(
            "Loading model (this may download several GB on first run)...".to_string(),
        ));

        let mut llama = match LocalLlama::init(true) {
            Ok(llama) => llama,
            Err(err) => {
                let _ = event_tx.send(ChatEvent::Error(format!("Failed to load model: {err}")));
                return;
            }
        };

        let _ = event_tx.send(ChatEvent::Status(
            "Model ready. Type a message and press Enter.".to_string(),
        ));

        while let Ok(prompt) = prompt_rx.recv() {
            let tx = event_tx.clone();
            let result = llama.generate_streaming(
                &prompt,
                256,
                0.7,
                |token| {
                    let _ = tx.send(ChatEvent::Token(token));
                },
                || {},
            );

            match result {
                Ok(()) => {
                    let _ = event_tx.send(ChatEvent::Done);
                }
                Err(err) => {
                    let _ = event_tx.send(ChatEvent::Error(format!("Generation failed: {err}")));
                }
            }
        }
    });

    commands.insert_resource(ChatChannel {
        prompt_tx,
        events: Mutex::new(event_rx),
    });
}

/// Intercepts Enter (without Shift) on the chat textarea to submit the current
/// prompt to the LLM worker instead of letting it insert a newline.
fn chat_submit_system(
    mut commands: Commands,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    channel: Option<Res<ChatChannel>>,
    mut chat_state: ResMut<ChatState>,
    mut textareas: Query<&mut Textarea, With<ChatTextarea>>,
    containers: Query<Entity, With<MessagesContainer>>,
) {
    let Some(channel) = channel else { return };

    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    if shift || !keyboard.just_pressed(KeyCode::Enter) {
        return;
    }

    let Ok(mut textarea) = textareas.single_mut() else {
        return;
    };

    if !textarea.focused || chat_state.generating {
        keyboard.clear_just_pressed(KeyCode::Enter);
        return;
    }

    let prompt = textarea.value().trim().to_string();
    keyboard.clear_just_pressed(KeyCode::Enter);

    if prompt.is_empty() {
        return;
    }

    let Ok(container) = containers.single() else {
        return;
    };

    textarea.clear();
    spawn_message_bubble(&mut commands, container, prompt.clone(), true);
    let assistant_entity = spawn_message_bubble(&mut commands, container, String::new(), false);
    chat_state.streaming_entity = Some(assistant_entity);
    chat_state.generating = true;

    if channel.prompt_tx.send(prompt).is_err() {
        chat_state.generating = false;
    }
}

/// Drains buffered events from the LLM worker, appending streamed tokens
/// directly onto the currently streaming assistant bubble.
fn chat_poll_system(
    channel: Option<Res<ChatChannel>>,
    mut chat_state: ResMut<ChatState>,
    mut commands: Commands,
    containers: Query<Entity, With<MessagesContainer>>,
    mut texts: Query<&mut Text>,
    mut scroll: Query<&mut ScrollPosition, With<MessagesContainer>>,
) {
    let Some(channel) = channel else { return };

    let Ok(events) = channel.events.lock() else {
        return;
    };

    let mut received_any = false;

    for event in events.try_iter() {
        received_any = true;

        match event {
            ChatEvent::Status(status) => {
                if let Ok(container) = containers.single() {
                    spawn_message_bubble(&mut commands, container, status, false);
                }
            }
            ChatEvent::Token(token) => {
                if let Some(entity) = chat_state.streaming_entity {
                    if let Ok(mut text) = texts.get_mut(entity) {
                        text.0.push_str(&token);
                    }
                }
            }
            ChatEvent::Done => {
                chat_state.streaming_entity = None;
                chat_state.generating = false;
            }
            ChatEvent::Error(err) => {
                if let Some(entity) = chat_state.streaming_entity.take() {
                    if let Ok(mut text) = texts.get_mut(entity) {
                        text.0 = format!("[error] {err}");
                    }
                }
                chat_state.generating = false;
            }
        }
    }

    if received_any {
        if let Ok(mut scroll_position) = scroll.single_mut() {
            scroll_position.y = f32::MAX;
        }
    }
}
