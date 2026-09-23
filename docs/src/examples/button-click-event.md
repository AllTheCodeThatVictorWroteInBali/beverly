# 15.2 Button Click Event

This example extends 15.1 by adding a button, firing an app message when it is pressed, and listening for that message to log the click.

```rust
use bevy::prelude::*;
use beverly::prelude::{light_theme, BeverlyPlugin, ThemeResource};

#[derive(Component)]
struct ClickMeButton;

#[derive(Message)]
struct HelloButtonClicked;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_plugins(BeverlyPlugin)
		.insert_resource(ThemeResource {
			current: light_theme(),
		})
		.add_message::<HelloButtonClicked>()
		.add_systems(Startup, setup)
		.add_systems(Update, (emit_click_event, log_click_event))
		.run();
}

fn setup(mut commands: Commands) {
	commands.spawn(Camera2d);

	commands
		.spawn(Node {
			width: percent(100),
			height: percent(100),
			justify_content: JustifyContent::Center,
			align_items: AlignItems::Center,
			..default()
		})
		.with_children(|parent| {
			parent
				.spawn((
					bevy::ui::widget::Button,
					Node {
						width: px(220),
						height: px(56),
						justify_content: JustifyContent::Center,
						align_items: AlignItems::Center,
						..default()
					},
					BackgroundColor(Color::srgb(0.18, 0.18, 0.22)),
					BorderRadius::all(px(12)),
					ClickMeButton,
				))
				.with_children(|button| {
					button.spawn((
						Text::new("Click me"),
						TextFont {
							font_size: FontSize::Px(20.0),
							..default()
						},
						TextColor(Color::srgb(0.95, 0.96, 0.99)),
					));
				});
		});
}

fn emit_click_event(
	interactions: Query<&Interaction, (Changed<Interaction>, With<ClickMeButton>)>,
	mut writer: MessageWriter<HelloButtonClicked>,
) {
	for interaction in &interactions {
		if *interaction == Interaction::Pressed {
			writer.write(HelloButtonClicked);
		}
	}
}

fn log_click_event(mut reader: MessageReader<HelloButtonClicked>) {
	for _ in reader.read() {
		info!("Hello button clicked");
	}
}
```

When you press the button, this app writes `Hello button clicked` to the console.
