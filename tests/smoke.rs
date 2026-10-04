//! Smoke test verifying Beverly's public API surface is usable from an
//! external crate. `BeverlyPlugin` itself requires a real GPU/render context
//! (like any Bevy rendering plugin), so this exercises plugin construction
//! and core theme/data types rather than a full headless render pass.

use beverly::prelude::*;

#[model]
struct SmokeModel {
    #[setter = set_name]
    name: String,
}

impl SmokeModel {
    fn set_name(&mut self, value: String) -> Result<(), String> {
        let value = value.trim().to_owned();
        if value.is_empty() {
            return Err("name cannot be empty".to_owned());
        }
        self.name = value;
        Ok(())
    }
}

#[test]
fn beverly_plugin_and_core_types_are_constructible() {
    let _plugin = BeverlyPlugin;
    let theme = ThemeResource {
        current: light_theme(),
    };
    let _ = dark_theme();
    assert_eq!(theme.current.mode, ThemeMode::Light);
}

#[test]
fn public_model_macro_and_bound_field_support_custom_setters() {
    let mut app = bevy::prelude::App::new();
    app.insert_resource(SmokeModel {
        name: "Initial".to_owned(),
    });

    let bound = UiElement::input("Name").bind(SmokeModel::name);
    assert!(matches!(bound, UiElement::Bound { .. }));

    let field = SmokeModel::name.erase();
    assert_eq!(
        field
            .write(
                app.world_mut(),
                BindingValue::Text("  Beverly  ".to_owned())
            )
            .unwrap(),
        BindingValue::Text("Beverly".to_owned()),
    );
    assert_eq!(app.world().resource::<SmokeModel>().get_name(), "Beverly");
}
