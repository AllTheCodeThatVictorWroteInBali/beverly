//! Shared scalar spring used by toggle motion and play-button glass deformation.

/// Advances a damped scalar spring and snaps it to rest within a small tolerance.
/// This is the toggle's existing integrator, not a separate animation engine.
pub fn spring_step(
    progress: &mut f32,
    velocity: &mut f32,
    target: f32,
    dt: f32,
    stiffness: f32,
    damping: f32,
) {
    if dt <= 0.0 || !dt.is_finite() {
        return;
    }
    *velocity += (target - *progress) * stiffness * dt;
    *velocity *= (1.0 - damping * dt).clamp(0.0, 1.0);
    *progress += *velocity * dt;
    if (target - *progress).abs() < 0.001 && velocity.abs() < 0.001 {
        *progress = target;
        *velocity = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_spring_settles_on_press_and_release() {
        let (mut value, mut velocity) = (0.0, 0.0);
        for target in [1.0, 0.0] {
            for _ in 0..240 {
                spring_step(&mut value, &mut velocity, target, 1.0 / 60.0, 260.0, 26.0);
            }
            assert_eq!((value, velocity), (target, 0.0));
        }
    }

    #[test]
    fn shared_spring_ignores_invalid_time() {
        for dt in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            let (mut value, mut velocity) = (0.2, 0.3);
            spring_step(&mut value, &mut velocity, 1.0, dt, 260.0, 26.0);
            assert_eq!((value, velocity), (0.2, 0.3));
        }
    }
}