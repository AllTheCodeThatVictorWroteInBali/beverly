use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct EventPropagationControl {
    pub stop_propagation: bool,
    pub prevent_default: bool,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct InteractionEventCapture;

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct InteractionEventTarget;
