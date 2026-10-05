use bevy::prelude::*;

/// Animated trailing dots that cycle `""`, `"."`, `".."`, `"..."` and repeat.
///
/// The component owns the entity's `Text`, so pair it with `ThemedText` for
/// styling but do not set the text yourself.
#[derive(Component, Clone, Copy, Debug)]
#[require(Text)]
pub struct Dots {
    pub max_dots: usize,
    pub interval: f32,
    elapsed: f32,
}

impl Default for Dots {
    fn default() -> Self {
        Self {
            max_dots: 3,
            interval: 0.4,
            elapsed: 0.0,
        }
    }
}

impl Dots {
    pub fn new() -> Self {
        Self::default()
    }

    /// Seconds each step (including the empty step) stays on screen.
    pub fn interval(mut self, seconds: f32) -> Self {
        self.interval = seconds.max(0.01);
        self
    }

    /// Largest number of dots shown before the cycle returns to zero.
    pub fn max_dots(mut self, count: usize) -> Self {
        self.max_dots = count;
        self
    }

    fn visible_dots(&self) -> usize {
        ((self.elapsed / self.interval) as usize).min(self.max_dots)
    }

    fn cycle_seconds(&self) -> f32 {
        self.interval * (self.max_dots + 1) as f32
    }
}

fn animate_dots(time: Res<Time>, mut dots: Query<(&mut Dots, &mut Text)>) {
    for (mut dots, mut text) in &mut dots {
        dots.elapsed = (dots.elapsed + time.delta_secs()) % dots.cycle_seconds();

        let count = dots.visible_dots();
        if text.0.len() != count {
            text.0 = ".".repeat(count);
        }
    }
}

pub struct DotsPlugin;

impl Plugin for DotsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animate_dots);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at_step(dots: Dots, step: f32) -> usize {
        Dots {
            elapsed: (step + 0.5) * dots.interval,
            ..dots
        }
        .visible_dots()
    }

    #[test]
    fn steps_through_zero_to_max_dots() {
        let dots = Dots::new();
        let shown: Vec<_> = (0..4).map(|step| at_step(dots, step as f32)).collect();
        assert_eq!(shown, [0, 1, 2, 3]);
    }

    #[test]
    fn max_dots_and_interval_are_configurable() {
        let dots = Dots::new().max_dots(5).interval(0.1);
        assert!((dots.cycle_seconds() - 0.6).abs() < 1e-6);
        assert_eq!(at_step(dots, 5.0), 5);
    }

    #[test]
    fn system_wraps_back_to_zero_and_writes_text() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, DotsPlugin));
        // A huge interval makes the frame delta irrelevant to the asserted step.
        let dots = Dots::new().interval(1000.0);
        let entity = app
            .world_mut()
            .spawn(Dots {
                elapsed: 2.5 * dots.interval,
                ..dots
            })
            .id();

        app.update();
        assert_eq!(app.world().get::<Text>(entity).unwrap().0, "..");

        app.world_mut().get_mut::<Dots>(entity).unwrap().elapsed = 3.9 * dots.interval;
        app.update();
        assert_eq!(app.world().get::<Text>(entity).unwrap().0, "...");

        app.world_mut().get_mut::<Dots>(entity).unwrap().elapsed = 4.5 * dots.interval;
        app.update();
        assert_eq!(app.world().get::<Text>(entity).unwrap().0, "");
    }
}
