use bevy::prelude::*;
use std::collections::HashMap;

use super::PointerId;

#[derive(Clone, Copy, Debug, Default)]
pub struct PointerVelocity {
    pub px_per_sec: Vec2,
}

#[derive(Resource, Default)]
pub struct PointerVelocityTracker {
    history: HashMap<PointerId, [(f64, Vec2); 4]>,
    size: HashMap<PointerId, usize>,
}

impl PointerVelocityTracker {
    pub fn push_sample(&mut self, pointer_id: PointerId, timestamp_secs: f64, position: Vec2) {
        let entry = self
            .history
            .entry(pointer_id)
            .or_insert([(0.0, Vec2::ZERO); 4]);
        let size = self.size.entry(pointer_id).or_insert(0);

        if *size < 4 {
            entry[*size] = (timestamp_secs, position);
            *size += 1;
        } else {
            entry.rotate_left(1);
            entry[3] = (timestamp_secs, position);
        }
    }

    pub fn velocity(&self, pointer_id: PointerId) -> PointerVelocity {
        let Some(history) = self.history.get(&pointer_id) else {
            return PointerVelocity::default();
        };
        let size = self.size.get(&pointer_id).copied().unwrap_or(0);
        if size < 2 {
            return PointerVelocity::default();
        }

        let first = history[0];
        let last = history[size - 1];
        let dt = (last.0 - first.0).max(1e-6) as f32;
        PointerVelocity {
            px_per_sec: (last.1 - first.1) / dt,
        }
    }

    pub fn clear_pointer(&mut self, pointer_id: PointerId) {
        self.history.remove(&pointer_id);
        self.size.remove(&pointer_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn velocity_is_estimated_from_bounded_history() {
        let mut tracker = PointerVelocityTracker::default();
        let pointer = PointerId(1);
        tracker.push_sample(pointer, 0.0, Vec2::new(0.0, 0.0));
        tracker.push_sample(pointer, 0.1, Vec2::new(10.0, 0.0));
        tracker.push_sample(pointer, 0.2, Vec2::new(30.0, 0.0));

        let v = tracker.velocity(pointer).px_per_sec;
        assert!(v.x > 100.0);
        assert!(v.y.abs() < 1e-6);
    }

    #[test]
    fn clear_pointer_resets_velocity() {
        let mut tracker = PointerVelocityTracker::default();
        let pointer = PointerId(2);
        tracker.push_sample(pointer, 0.0, Vec2::ZERO);
        tracker.push_sample(pointer, 0.1, Vec2::new(1.0, 1.0));
        tracker.clear_pointer(pointer);

        assert_eq!(tracker.velocity(pointer).px_per_sec, Vec2::ZERO);
    }
}
