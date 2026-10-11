# Noise and Texture

`Noise` adds procedural grain to a `Surface`. It varies the surface's brightness across its area, which breaks up flat fills and gradients and adds tactile character. The shader generates it, so no texture asset is needed. `BeverlyPlugin` installs the renderer used by the examples below.

## Basic Usage

```rust
use beverly::rendering::{GradientStop, LinearGradient, Noise, Paint, Surface};
use bevy::prelude::*;

fn add_grainy_surface(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Node {
            width: Val::Px(300.0),
            height: Val::Px(170.0),
            ..default()
        },
        Surface::rounded_rect_fill(
            8.0,
            Paint::linear(LinearGradient::angle_degrees(45.0, vec![
                GradientStop::new(0.0, Color::srgb(0.98, 0.76, 0.18)),
                GradientStop::new(1.0, Color::srgb(0.94, 0.26, 0.32)),
            ])),
        )
        .with_noise(Noise::grain(8.0, 0.12)),
    ));
}
```

`Noise::grain(scale, strength)` takes the grain size in pixels and a strength from `0.0` to `1.0`. A default `Noise` has zero strength and is invisible, so always set a strength. Use `surface.clear_noise()` to remove it.

## How It Works

The shader samples smooth value noise on a grid spaced `scale` pixels apart, producing a value between -1 and 1 for each pixel. It then multiplies the pixel's brightness by `1 + noise * strength`.

- All color channels are multiplied by the same factor, so noise darkens and lightens a color without shifting its hue.
- **Alpha is not changed.** Noise never makes a surface more or less transparent.
- Brightness is scaled, not offset, so a **pure black fill stays black**. A white fill can only get darker, because its channels are already at the maximum. Mid-tones show the most grain. The demo includes light, mid, and black swatches so you can see this.
- The pattern is defined in the surface's local coordinates, so it moves with the surface rather than staying fixed on screen.

## Parameters

| Setter | Default | Valid range | Effect |
| --- | --- | --- | --- |
| `Noise::grain(scale, strength)` | 24 px, 0 | scale 0.001 to 4096, strength 0 to 1 | Grain size and amplitude |
| `with_seed(seed)` | 0 | finite values | Picks a different but repeatable pattern |
| `with_animated(bool)` | `false` | | Scrolls the noise over time |
| `with_speed(speed)` | 0 | 0 to 16 | Scroll speed; needs `animated` and a speed above zero to move |
| `with_target(target)` | `Surface` | see below | Which part of the surface is textured |

Out-of-range values are clamped and non-finite values fall back to their defaults, so extreme input cannot break the shader. `NoiseKind` currently has one value, `Grain`, and `NoiseSpace` has one value, `Local`; both are set for you.

### Grain size

Small values (1 to 3 px) look like fine film grain. Larger values give soft, cloudy mottling. The grain size is in **physical pixels** and is not scaled by the display's density, so the same value looks finer on a high-density display than on a standard one. Check the result on the displays you target.

### Seed

Two surfaces with the same size and settings produce identical grain, because the pattern comes from local coordinates. Give each its own seed when they sit side by side and the repetition would be noticeable.

## Targets

`NoiseTarget` chooses which layer is textured:

| Target | Textures |
| --- | --- |
| `Surface` (default) | The fill, the border, and the [blurred backdrop](blur.md) together |
| `Fill` | The fill only |
| `Border` | The border only |

```rust
use beverly::rendering::{Noise, NoiseTarget, Paint, Surface};
use bevy::prelude::*;

let surface = Surface::rounded_rect_fill(8.0, Paint::solid(Color::srgb(0.5, 0.5, 0.5)))
    .uniform_border(6.0, Paint::solid(Color::srgb(0.18, 0.55, 0.94)))
    .with_noise(Noise::grain(4.0, 0.2).with_target(NoiseTarget::Border));
```

With the `Surface` target, a frosted-glass surface gets grain over its blurred backdrop, a common way to add a frosted texture. Shadows, glows, inner shadows, and focus rings are not textured.

## Animated Noise

```rust
use beverly::rendering::Noise;

let noise = Noise::grain(8.0, 0.1).with_animated(true).with_speed(2.0);
```

Animation scrolls the same noise field diagonally at the given speed. It does not reshuffle the pattern.

Animated noise updates its time value every frame and prevents that surface from sharing a material with identical ones, so use it on a few prominent surfaces rather than many.

Reduced-effects and reduced-motion behavior:

- Setting `UI_REDUCED_EFFECTS=1` turns noise **off entirely**, animated or not.
- The accessibility policy's `reduced_effects` and `reduced_motion` flags do **not** change noise. They affect other animated paints, such as shimmer and spinning gradients. Check `AccessibilityVisualPolicyResource` yourself and pass `false` to `with_animated` when `reduced_motion` is set, as the demo does:

```rust
use beverly::rendering::Noise;
use beverly::theme::AccessibilityVisualPolicyResource;
use bevy::prelude::*;

fn grain(policy: &AccessibilityVisualPolicyResource) -> Noise {
    Noise::grain(8.0, 0.1)
        .with_animated(!policy.current.reduced_motion)
        .with_speed(2.0)
}
```

## Updating at Runtime

Assign a new value to `surface.noise`:

```rust
use beverly::rendering::{Noise, Surface};

fn set_grain(surface: &mut Surface, strength: f32) {
    surface.noise = Some(Noise::grain(8.0, strength));
}
```

Avoid writing an identical value each frame, which causes needless material updates. Compare first and assign only on a change.

## Design Guidance

- Keep strength low. Values from about `0.03` to `0.15` read as texture; higher values read as dirt or static.
- Check contrast. Noise changes the brightness text sits on, so verify readability at the lightest and darkest parts of the grain.
- Use texture sparingly and consistently, for example on hero surfaces, frosted glass, or a deliberately tactile theme, and not on every card.
- Do not use noise as the only way to communicate state.

## Debugging

Set `UI_RENDER_DEBUG` to inspect noise while tuning it:

```sh
UI_RENDER_DEBUG=noise cargo run --example noise
```

Related values are `noise_coords`, `noise_strength`, and `noise_modulation`. See [Shaders](shaders.md#debugging) for the full list.

## Demo

Run the interactive demo from the repository root:

```sh
cargo run --example noise
cargo run --example noise -- dark
```

The demo has sliders for grain size (1 to 64 px), strength (0 to 100 percent), seed, and animation speed. Buttons choose the target (Surface, Fill, or Border) and switch between Still and Animated.

Previews show a gradient with a thick border, flat light, mid, and black swatches, and a frosted-glass window over colored stripes. All of them share the same settings. Animation is disabled when the reduced-motion preference is on. The top-right toggle switches the page between light and dark mode.
