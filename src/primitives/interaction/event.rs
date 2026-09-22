use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PointerId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PointerType {
    Mouse,
    Touch,
    Pen,
    Unknown,
}

impl Default for PointerType {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct PointerButtons {
    pub primary: bool,
    pub secondary: bool,
    pub middle: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerButtonState {
    Pressed,
    Released,
    None,
}

impl Default for PointerButtonState {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct PointerEventModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub command: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerEvent {
    pub pointer_id: PointerId,
    pub pointer_type: PointerType,
    pub window_position: Vec2,
    pub screen_position: Vec2,
    pub delta: Vec2,
    pub buttons: PointerButtons,
    pub button_state: PointerButtonState,
    pub pressure: f32,
    pub tilt: Vec2,
    pub timestamp_secs: f64,
    pub modifiers: PointerEventModifiers,
}

impl Default for PointerEvent {
    fn default() -> Self {
        Self {
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
            window_position: Vec2::ZERO,
            screen_position: Vec2::ZERO,
            delta: Vec2::ZERO,
            buttons: PointerButtons::default(),
            button_state: PointerButtonState::None,
            pressure: 0.5,
            tilt: Vec2::ZERO,
            timestamp_secs: 0.0,
            modifiers: PointerEventModifiers::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractionEventType {
    PointerEnter,
    PointerLeave,
    PointerMove,
    PointerDown,
    PointerUp,
    PointerCancel,
    PointerOver,
    PointerOut,
    Click,
    DoubleClick,
    DragStart,
    DragMove,
    DragEnd,
    DragCancel,
    LongPress,
    Scroll,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractionEventPhase {
    Capture,
    Target,
    Bubble,
}

#[derive(Clone, Copy, Debug)]
pub struct InteractionEventContext {
    pub target: Entity,
    pub current_target: Entity,
    pub phase: InteractionEventPhase,
    pub local_position: Vec2,
}

#[derive(Message, Clone, Copy, Debug)]
pub struct UiPointerEvent {
    pub event_type: InteractionEventType,
    pub pointer: PointerEvent,
    pub context: InteractionEventContext,
    pub propagation_stopped: bool,
    pub default_prevented: bool,
}

#[derive(Message, Clone, Copy, Debug)]
pub struct PointerEventRequest {
    pub event_type: InteractionEventType,
    pub pointer: PointerEvent,
}

#[derive(Bundle, Clone, Copy, Debug, Default)]
pub struct PointerEventBundle {
    pub marker: super::UiHitNode,
}
