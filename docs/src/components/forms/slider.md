# Slider

A slider selects a numeric value within a range. `Slider` stores the value;
`SliderStyle` controls the track, fill, thumb, and optional value display.
`BeverlyPlugin` installs the slider systems.

## Live Demo

<iframe src="../../assets/demos/index.html?demo=slider" title="Interactive Beverly slider demo" loading="lazy" allow="fullscreen" style="display: block; width: 100%; height: min(820px, 90vh); min-height: 580px; border: 0;"></iframe>

<a href="../../assets/demos/index.html?demo=slider" target="_blank" rel="noopener">Open the demo in a separate tab</a>.

## Basic Usage

```rust
use beverly::components::slider::{Slider, SliderStyle, spawn_slider};
use beverly::prelude::ThemeResource;
use bevy::prelude::*;

fn add_slider(parent: &mut ChildSpawnerCommands, theme: &ThemeResource) {
    spawn_slider(
        parent,
        Slider::new(0.0, 100.0).value(42.0).step(1.0).label("Volume"),
        SliderStyle::default(),
        theme,
    );
}
```

Use `.disabled(true)` to prevent interaction. Provide `.label(...)` as the
accessible name, and keep a visible label beside the control.

## Reading Changes

Read `SliderChanged` with `MessageReader<SliderChanged>` in an update system.
Each message contains the slider `entity`, its numeric `value`, and whether it
is currently `dragging`. The example uses the entity to update only the matching
readout; application state remains owned by the application.