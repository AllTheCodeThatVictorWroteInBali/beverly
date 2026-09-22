use bevy::prelude::*;
use std::collections::HashMap;

use super::PointerId;

#[derive(Component, Clone, Copy, Debug)]
pub struct CapturedPointer {
    pub pointer_id: PointerId,
}

#[derive(Resource, Default, Clone, Debug)]
pub struct PointerCaptureMap {
    by_pointer: HashMap<PointerId, Entity>,
}

impl PointerCaptureMap {
    pub fn capture(&mut self, pointer_id: PointerId, entity: Entity) {
        self.by_pointer.insert(pointer_id, entity);
    }

    pub fn release(&mut self, pointer_id: PointerId) {
        self.by_pointer.remove(&pointer_id);
    }

    pub fn captured_entity(&self, pointer_id: PointerId) -> Option<Entity> {
        self.by_pointer.get(&pointer_id).copied()
    }

    pub fn release_for_entity(&mut self, entity: Entity) {
        self.by_pointer.retain(|_, value| *value != entity);
    }
}

#[derive(Message, Clone, Copy, Debug)]
pub struct PointerCaptureRequest {
    pub pointer_id: PointerId,
    pub entity: Entity,
}

#[derive(Message, Clone, Copy, Debug)]
pub struct PointerReleaseRequest {
    pub pointer_id: PointerId,
    pub entity: Option<Entity>,
}

pub fn apply_capture_requests(
    mut capture_map: ResMut<PointerCaptureMap>,
    mut capture_requests: MessageReader<PointerCaptureRequest>,
    mut release_requests: MessageReader<PointerReleaseRequest>,
) {
    for request in capture_requests.read() {
        capture_map.capture(request.pointer_id, request.entity);
    }

    for request in release_requests.read() {
        if let Some(entity) = request.entity {
            if capture_map.captured_entity(request.pointer_id) == Some(entity) {
                capture_map.release(request.pointer_id);
            }
        } else {
            capture_map.release(request.pointer_id);
        }
    }
}

pub fn cleanup_capture_for_removed_entities(
    mut removed: RemovedComponents<super::UiHitNode>,
    mut capture_map: ResMut<PointerCaptureMap>,
) {
    for entity in removed.read() {
        capture_map.release_for_entity(entity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_map_tracks_and_releases_pointer() {
        let mut map = PointerCaptureMap::default();
        let pointer = PointerId(7);
        let a = Entity::from_raw_u32(10).expect("valid entity id");

        map.capture(pointer, a);
        assert_eq!(map.captured_entity(pointer), Some(a));

        map.release(pointer);
        assert_eq!(map.captured_entity(pointer), None);
    }

    #[test]
    fn release_for_entity_clears_all_matching_captures() {
        let mut map = PointerCaptureMap::default();
        let target = Entity::from_raw_u32(12).expect("valid entity id");
        map.capture(PointerId(1), target);
        map.capture(PointerId(2), target);
        map.capture(PointerId(3), Entity::from_raw_u32(99).expect("valid entity id"));

        map.release_for_entity(target);
        assert_eq!(map.captured_entity(PointerId(1)), None);
        assert_eq!(map.captured_entity(PointerId(2)), None);
        assert!(map.captured_entity(PointerId(3)).is_some());
    }
}
