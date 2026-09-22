//! Smoke test verifying Beverly's public API surface is usable from an
//! external crate. `BeverlyPlugin` itself requires a real GPU/render context
//! (like any Bevy rendering plugin), so this exercises plugin construction
//! and core theme/data types rather than a full headless render pass.

use beverly::prelude::*;

#[test]
fn beverly_plugin_and_core_types_are_constructible() {
    let _plugin = BeverlyPlugin;
    let theme = ThemeResource {
        current: light_theme(),
    };
    let _ = dark_theme();
    assert_eq!(theme.current.mode, ThemeMode::Light);
}
