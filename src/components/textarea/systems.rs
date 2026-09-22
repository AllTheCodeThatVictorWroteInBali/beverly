// src/ui/textarea/systems.rs

use bevy::{
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
};

use super::{
    Textarea, TextareaChanged,
    component::{TextareaPlaceholder, TextareaSurface, TextareaText},
};
use crate::primitives::a11y::{FocusGained, FocusLost};
use crate::primitives::clipboard::Clipboard;
use crate::theme::ThemeResource;
use crate::rendering::Paint;

/// Bridges keyboard (Tab) focus into `Textarea::focused`, so tabbing to a
/// textarea lets you type immediately, the same as clicking into it.
pub fn textarea_focus_gained_system(
    trigger: On<FocusGained>,
    mut textareas: Query<(Entity, &mut Textarea), With<TextareaSurface>>,
) {
    let entity = trigger.entity;
    if !textareas.iter().any(|(candidate, _)| candidate == entity) {
        return;
    }

    for (candidate, mut textarea) in &mut textareas {
        textarea.focused = candidate == entity;
        if textarea.focused {
            textarea.reset_caret();
        }
    }
}

pub fn textarea_focus_lost_system(
    trigger: On<FocusLost>,
    mut textareas: Query<&mut Textarea, With<TextareaSurface>>,
) {
    let entity = trigger.entity;
    if let Ok(mut textarea) = textareas.get_mut(entity) {
        textarea.focused = false;
    }
}

pub fn textarea_focus_system(
    interaction_query: Query<(Entity, &Interaction), (Changed<Interaction>, With<TextareaSurface>)>,
    mut textareas: Query<(Entity, &mut Textarea), With<TextareaSurface>>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    let pressed = interaction_query
        .iter()
        .find_map(|(entity, interaction)| (*interaction == Interaction::Pressed).then_some(entity));

    if let Some(focused_entity) = pressed {
        for (entity, mut textarea) in &mut textareas {
            textarea.focused = entity == focused_entity;

            if textarea.focused {
                textarea.reset_caret();
            }
        }

        return;
    }

    if mouse.just_pressed(MouseButton::Left) {
        for (_, mut textarea) in &mut textareas {
            textarea.focused = false;
        }
    }
}

pub fn textarea_visual_system(
    time: Res<Time>,
    theme: Res<ThemeResource>,
    textareas: Query<(Entity, &Textarea), With<TextareaSurface>>,
    mut text_query: Query<
        (&ChildOf, &mut Text, &mut TextColor),
        (With<TextareaText>, Without<TextareaPlaceholder>),
    >,
    mut placeholder_query: Query<
        (&ChildOf, &mut Text, &mut TextColor, &mut Visibility),
        (With<TextareaPlaceholder>, Without<TextareaText>),
    >,
    mut surface_query: Query<&mut crate::rendering::Surface, With<TextareaSurface>>,
) {
    let blink_on = (time.elapsed_secs() * 2.0).fract() < 0.5;
    let colors = theme.current.colors;

    for (entity, textarea) in &textareas {
        let mut display_value = textarea.value.clone();

        if textarea.focused && blink_on {
            display_value.push('|');
        }

        for (parent, mut text, mut text_color) in &mut text_query {
            if parent.parent() == entity {
                *text = Text::new(display_value.clone());
                text_color.0 = colors.text;
            }
        }

        for (parent, mut placeholder, mut placeholder_color, mut visibility) in
            &mut placeholder_query
        {
            if parent.parent() == entity {
                *placeholder = Text::new(textarea.placeholder.clone());
                placeholder_color.0 = colors.text_muted;
                *visibility = if textarea.value.is_empty() {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            }
        }

        if let Ok(mut surface) = surface_query.get_mut(entity) {
            if textarea.focused {
                surface.fill = Paint::solid(colors.surface);
                if let Some(border) = surface.border.as_mut() {
                    border.paint = Paint::solid(colors.focus);
                }
            } else {
                surface.fill = Paint::solid(colors.surface_elevated);
                if let Some(border) = surface.border.as_mut() {
                    border.paint = Paint::solid(colors.border);
                }
            }
        }
    }
}

pub fn textarea_keyboard_system(
    keyboard: Res<ButtonInput<KeyCode>>,

    mut keyboard_events: MessageReader<KeyboardInput>,

    mut clipboard: ResMut<Clipboard>,

    mut textareas: Query<(Entity, &mut Textarea)>,

    mut changed: MessageWriter<TextareaChanged>,
) {
    let selecting = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    let command = keyboard.pressed(KeyCode::ControlLeft)
        || keyboard.pressed(KeyCode::ControlRight)
        || keyboard.pressed(KeyCode::SuperLeft)
        || keyboard.pressed(KeyCode::SuperRight);

    for (entity, mut textarea) in &mut textareas {
        if !textarea.focused {
            continue;
        }

        /*
         * Select all
         */
        if command && keyboard.just_pressed(KeyCode::KeyA) {
            textarea.select_all();
        }

        /*
         * Copy
         */
        if command && keyboard.just_pressed(KeyCode::KeyC) {
            if textarea.has_selection() {
                clipboard.set(textarea.selected_text());
            }
        }

        /*
         * Cut
         */
        if command && keyboard.just_pressed(KeyCode::KeyX) {
            if textarea.has_selection() {
                clipboard.set(textarea.selected_text());

                if textarea.delete_selection() {
                    changed.write(TextareaChanged::new(entity, &textarea));
                }
            }
        }

        /*
         * Paste
         */
        if command && keyboard.just_pressed(KeyCode::KeyV) {
            let text = clipboard.get().to_owned();

            if !text.is_empty() && textarea.insert_text(&text) {
                changed.write(TextareaChanged::new(entity, &textarea));
            }
        }

        /*
         * Backspace
         */
        if keyboard.just_pressed(KeyCode::Backspace) {
            if textarea.backspace() {
                changed.write(TextareaChanged::new(entity, &textarea));
            }
        }

        /*
         * Delete
         */
        if keyboard.just_pressed(KeyCode::Delete) {
            if textarea.delete_forward() {
                changed.write(TextareaChanged::new(entity, &textarea));
            }
        }

        /*
         * Enter
         */
        if keyboard.just_pressed(KeyCode::Enter) {
            if textarea.insert_text("\n") {
                changed.write(TextareaChanged::new(entity, &textarea));
            }
        }

        /*
         * Character input.
         */
        for event in keyboard_events.read() {
            if !event.state.is_pressed() {
                continue;
            }

            if command {
                continue;
            }

            let inserted = match &event.logical_key {
                Key::Character(text) => {
                    if text.chars().any(|c| c.is_control()) {
                        continue;
                    }

                    text.as_str()
                }
                Key::Space => " ",
                _ => continue,
            };

            if textarea.insert_text(inserted) {
                changed.write(TextareaChanged::new(entity, &textarea));
            }
        }

        if selecting {
            textarea.reset_caret();
        }
    }
}
