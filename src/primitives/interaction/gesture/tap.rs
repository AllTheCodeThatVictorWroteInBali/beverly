use bevy::prelude::*;

use super::super::PointerId;

#[derive(Message, Clone, Copy, Debug)]
pub struct GestureTapEvent {
    pub pointer_id: PointerId,
    pub target: Entity,
    pub position: Vec2,
    pub count: u8,
}
