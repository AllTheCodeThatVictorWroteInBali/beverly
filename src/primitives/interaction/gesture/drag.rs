use bevy::prelude::*;

use super::super::{PointerId, PointerType};

#[derive(Message, Clone, Copy, Debug)]
pub struct GestureDragEvent {
    pub pointer_id: PointerId,
    pub pointer_type: PointerType,
    pub target: Entity,
    pub start_position: Vec2,
    pub current_position: Vec2,
    pub delta: Vec2,
    pub total_delta: Vec2,
    pub velocity: Vec2,
    pub ended: bool,
    pub cancelled: bool,
}
