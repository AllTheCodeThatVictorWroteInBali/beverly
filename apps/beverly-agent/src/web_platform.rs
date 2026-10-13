use super::ChatTextarea;
use super::{
    AgentFooter, AgentFooterContact, AgentFooterMetrics, AgentMetricsRow, ChatEvent, ChatState,
    LoadingGroup, MessagesContainer, MetricsModelText, MetricsText, ModelPickerScreen,
    WorkerCommand,
};
use beverly::components::textarea::Textarea;
use bevy::prelude::*;
use std::sync::atomic::Ordering;
use wasm_bindgen::prelude::*;

#[derive(Component)]
pub struct EmptyConversation;

#[wasm_bindgen(module = "/bridge.js")]
extern "C" {
    #[wasm_bindgen(js_name = initializeWorker)]
    pub fn initialize();
    #[wasm_bindgen(js_name = sendCommand)]
    fn send_command(message: &str);
    #[wasm_bindgen(js_name = drainEvents)]
    fn drain_events() -> String;
    #[wasm_bindgen(js_name = syncMobileInput)]
    fn sync_mobile_input(value: &str, busy: bool, left: f32, top: f32, width: f32, height: f32);
    #[wasm_bindgen(js_name = drainMobileInput)]
    fn drain_mobile_input() -> String;
}

pub struct WorkerSender;
pub struct WorkerEvents;
pub struct WorkerStop;
pub struct EventGuard(Vec<ChatEvent>);

impl WorkerSender {
    pub fn send(&self, command: WorkerCommand) -> Result<(), String> {
        let message = match command {
            WorkerCommand::Load(_) => serde_json::json!({"command":"load"}),
            WorkerCommand::Prompt(prompt) => {
                serde_json::json!({"command":"prompt","prompt":prompt})
            }
        };
        send_command(&message.to_string());
        Ok(())
    }
}

impl WorkerStop {
    pub fn store(&self, stop: bool, _ordering: Ordering) {
        if stop {
            send_command("{\"command\":\"cancel\"}");
        }
    }
}

impl WorkerEvents {
    pub fn lock(&self) -> Result<EventGuard, ()> {
        let events: Vec<serde_json::Value> =
            serde_json::from_str(&drain_events()).map_err(|_| ())?;
        let mut result = Vec::new();
        for event in events {
            match event["type"].as_str().unwrap_or("") {
                "progress" => result.push(ChatEvent::LoadProgress(
                    event["stage"]
                        .as_str()
                        .unwrap_or("Downloading model weights")
                        .into(),
                    event["fraction"].as_f64().unwrap_or(0.0) as f32,
                )),
                "status" => result.push(ChatEvent::LoadProgress(
                    event["stage"]
                        .as_str()
                        .unwrap_or("Initializing model")
                        .into(),
                    0.0,
                )),
                "ready" => {
                    result.push(ChatEvent::Ready);
                    result.push(ChatEvent::WorkerMetrics(
                        0.0,
                        event["wasmMemoryBytes"].as_u64().unwrap_or(0),
                    ));
                }
                "first-token" => result.push(ChatEvent::FirstToken(
                    event["elapsedMs"].as_f64().unwrap_or(0.0).round() as u128,
                )),
                "token" => {
                    result.push(ChatEvent::ResponseText(
                        event["text"].as_str().unwrap_or("").into(),
                    ));
                    result.push(ChatEvent::GeneratedTokens(
                        event["tokens"].as_u64().unwrap_or(0) as usize,
                    ));
                }
                "done" => {
                    result.push(ChatEvent::WorkerMetrics(
                        event["tokensPerSecond"].as_f64().unwrap_or(0.0),
                        event["wasmMemoryBytes"].as_u64().unwrap_or(0),
                    ));
                    result.push(ChatEvent::Done);
                }
                "error" => {
                    let message = event["message"]
                        .as_str()
                        .unwrap_or("Browser inference failed")
                        .to_owned();
                    result.push(if event["fatal"].as_bool().unwrap_or(false) {
                        ChatEvent::BackendFailure(message)
                    } else {
                        ChatEvent::Error(message)
                    });
                }
                _ => (),
            }
        }
        Ok(EventGuard(result))
    }
}

impl EventGuard {
    pub fn try_iter(self) -> std::vec::IntoIter<ChatEvent> {
        self.0.into_iter()
    }
}

pub fn fit_browser_ui(
    mut commands: Commands,
    chat: Res<ChatState>,
    mut nodes: Query<(
        &mut Node,
        Option<&ModelPickerScreen>,
        Option<&MessagesContainer>,
        Option<&LoadingGroup>,
        Option<&MetricsText>,
        Option<&MetricsModelText>,
        Option<&AgentFooter>,
        Option<&AgentFooterContact>,
        Option<&AgentFooterMetrics>,
        Option<&AgentMetricsRow>,
    )>,
    windows: Query<&Window>,
    mut bars: Query<&mut beverly::components::progress_bar::ProgressBar>,
    empty: Query<Entity, With<EmptyConversation>>,
) {
    if chat.generating {
        for entity in &empty {
            commands.entity(entity).despawn();
        }
    }
    for (
        mut node,
        picker,
        messages,
        loading,
        metrics,
        model_metrics,
        footer,
        contact,
        footer_metrics,
        metrics_row,
    ) in &mut nodes
    {
        if picker.is_some() {
            node.padding = UiRect {
                top: px(72),
                bottom: px(24),
                left: px(16),
                right: px(16),
            };
        }
        if messages.is_some() {
            node.margin.top = px(52);
            node.min_height = px(0);
        }
        if loading.is_some() || metrics.is_some() || model_metrics.is_some() {
            node.max_width = percent(100);
            node.min_width = px(0);
        }
        if footer.is_some() {
            node.column_gap = px(12);
            node.align_items = AlignItems::Center;
            node.flex_shrink = 0.0;
            node.min_width = px(0);
        }
        if contact.is_some() {
            node.display = Display::None;
        }
        if footer_metrics.is_some() {
            node.width = Val::Auto;
            node.flex_grow = 1.0;
            node.min_width = px(0);
        }
        if metrics_row.is_some() {
            node.width = percent(100);
            node.min_width = px(0);
        }
        if metrics.is_some() {
            node.flex_grow = 1.0;
        }
    }
    if let Ok(window) = windows.single() {
        let width = (window.width() - 32.0).clamp(1.0, 700.0);
        for mut bar in &mut bars {
            if bar.width != width {
                bar.width = width;
            }
        }
    }
}

pub fn new_chat(commands: &mut Commands, _button: Entity) {
    commands.queue(|world: &mut World| {
        if world.resource::<ChatState>().generating {
            return;
        }
        send_command("{\"command\":\"reset\"}");
        let containers: Vec<_> = world
            .query_filtered::<Entity, With<MessagesContainer>>()
            .iter(world)
            .collect();
        for container in containers {
            let children: Vec<_> = world
                .get::<Children>(container)
                .map(|children| children.iter().collect())
                .unwrap_or_default();
            for child in children {
                world.despawn(child);
            }
            world.spawn((
                EmptyConversation,
                ChildOf(container),
                Text::new("Ask TinyLlama a question."),
                beverly::prelude::ThemedText::new(beverly::prelude::TextRole::Muted).size(16.0),
            ));
        }
    });
}

pub fn read_mobile_input(
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    mut synthetic_enter: Local<bool>,
    mut textareas: Query<&mut Textarea, With<ChatTextarea>>,
) {
    if *synthetic_enter {
        keyboard.release(KeyCode::Enter);
        *synthetic_enter = false;
    }
    let Ok(events) = serde_json::from_str::<Vec<serde_json::Value>>(&drain_mobile_input()) else {
        return;
    };
    let Ok(mut textarea) = textareas.single_mut() else {
        return;
    };
    for event in events {
        if event["type"] == "focus" {
            textarea.focused = true;
        }
        if event["type"] == "blur" {
            textarea.focused = false;
        }
        if event["type"] == "input" && !textarea.busy {
            let value = event["value"].as_str().unwrap_or("");
            textarea.set_value(value);
            textarea.focused = true;
            let selection = event["selection"]
                .as_u64()
                .unwrap_or(value.encode_utf16().count() as u64)
                as usize;
            let mut units = 0;
            textarea.cursor = value
                .chars()
                .take_while(|character| {
                    units += character.len_utf16();
                    units <= selection
                })
                .count();
        }
        if event["type"] == "submit" && !textarea.busy {
            textarea.focused = true;
            keyboard.press(KeyCode::Enter);
            *synthetic_enter = true;
        }
    }
}

pub fn position_mobile_input(
    textareas: Query<(&Textarea, &ComputedNode, &UiGlobalTransform), With<ChatTextarea>>,
) {
    if let Ok((textarea, computed, transform)) = textareas.single() {
        let scale = computed.inverse_scale_factor();
        let size = computed.size() * scale;
        let position = bevy::math::Affine2::from(transform).translation * scale - size * 0.5;
        sync_mobile_input(
            textarea.value(),
            textarea.busy,
            position.x,
            position.y,
            (size.x - 58.0).max(0.0),
            size.y,
        );
    } else {
        sync_mobile_input("", true, 0.0, 0.0, 0.0, 0.0);
    }
}
