use bevy::prelude::*;

use super::{PointerId, gesture::GestureTapEvent};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractionAction {
    Activate,
    Dismiss,
    Toggle,
    Select,
    Open,
    Close,
    Increment,
    Decrement,
    SetMinimum,
    SetMaximum,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractionActionSource {
    Pointer,
    Keyboard,
    Accessibility,
    Automation,
    Programmatic,
    Unknown,
}

impl Default for InteractionActionSource {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Message, Clone, Copy, Debug)]
pub struct InteractionActionEvent {
    pub action: InteractionAction,
    pub target: Entity,
    pub pointer_id: Option<PointerId>,
    pub source: InteractionActionSource,
    pub consumed: bool,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct ActionBinding {
    pub action: InteractionAction,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct DisabledInteraction;

pub fn derive_activation_actions(
    mut writer: MessageWriter<InteractionActionEvent>,
    mut taps: MessageReader<GestureTapEvent>,
    bindings: Query<&ActionBinding>,
    disabled: Query<(), With<DisabledInteraction>>,
) {
    for tap in taps.read() {
        let target = tap.target;
        if disabled.get(target).is_ok() {
            continue;
        }

        let action = bindings
            .get(target)
            .map(|binding| binding.action)
            .unwrap_or(InteractionAction::Activate);

        writer.write(InteractionActionEvent {
            action,
            target,
            pointer_id: Some(tap.pointer_id),
            source: InteractionActionSource::Pointer,
            consumed: false,
        });
    }
}
