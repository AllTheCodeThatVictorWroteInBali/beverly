// src/ui/textarea/component.rs

use bevy::prelude::*;

use crate::primitives::a11y;
use crate::components::text::{TextRole, ThemedText};
use crate::rendering::{Paint, Surface};

#[derive(Component)]
pub struct Textarea {
    pub value: String,
    pub placeholder: String,

    pub focused: bool,

    /// Character index of the cursor.
    pub cursor: usize,

    /// Selection anchor.
    pub selection_anchor: Option<usize>,

    pub max_length: Option<usize>,

    /// Vertical scroll offset.
    pub scroll_y: f32,

    pub line_height: f32,

    /// Caret state.
    pub caret_visible: bool,
    pub caret_timer: Timer,

    /// True while mouse/touch selection is being dragged.
    pub dragging_selection: bool,
}

#[derive(Component)]
pub struct TextareaSurface;

#[derive(Component)]
pub struct TextareaText;

#[derive(Component)]
pub struct TextareaPlaceholder;

#[derive(Clone)]
pub struct TextareaConfig {
    pub placeholder: String,
    pub max_length: Option<usize>,
    pub initial_value: String,
    pub height: f32,
}

impl Textarea {
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            value: String::new(),
            placeholder: placeholder.into(),

            focused: false,

            cursor: 0,
            selection_anchor: None,

            max_length: None,

            scroll_y: 0.0,
            line_height: 22.0,

            caret_visible: true,
            caret_timer: Timer::from_seconds(0.5, TimerMode::Repeating),

            dragging_selection: false,
        }
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self.cursor = self.value.chars().count();
        self
    }

    pub fn max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();

        if let Some(max) = self.max_length {
            self.value = self.value.chars().take(max).collect();
        }

        self.cursor = self.value.chars().count();
        self.clear_selection();
        self.reset_caret();
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
        self.clear_selection();
        self.reset_caret();
    }

    pub fn has_selection(&self) -> bool {
        matches!(
            self.selection_range(),
            Some((start, end)) if start != end
        )
    }

    pub fn selection_range(&self) -> Option<(usize, usize)> {
        let anchor = self.selection_anchor?;

        if anchor < self.cursor {
            Some((anchor, self.cursor))
        } else {
            Some((self.cursor, anchor))
        }
    }

    pub fn selected_text(&self) -> String {
        let Some((start, end)) = self.selection_range() else {
            return String::new();
        };

        self.value.chars().skip(start).take(end - start).collect()
    }

    pub fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    pub fn select_all(&mut self) {
        self.selection_anchor = Some(0);
        self.cursor = self.value.chars().count();
        self.reset_caret();
    }

    pub fn reset_caret(&mut self) {
        self.caret_visible = true;
        self.caret_timer.reset();
    }

    pub fn move_cursor(&mut self, position: usize, selecting: bool) {
        let position = position.min(self.value.chars().count());

        if selecting {
            if self.selection_anchor.is_none() {
                self.selection_anchor = Some(self.cursor);
            }
        } else {
            self.clear_selection();
        }

        self.cursor = position;
        self.reset_caret();
    }

    pub fn delete_selection(&mut self) -> bool {
        let Some((start, end)) = self.selection_range() else {
            return false;
        };

        if start == end {
            return false;
        }

        self.value = self
            .value
            .chars()
            .enumerate()
            .filter_map(|(i, c)| if i < start || i >= end { Some(c) } else { None })
            .collect();

        self.cursor = start;
        self.clear_selection();
        self.reset_caret();

        true
    }

    pub fn insert_text(&mut self, text: &str) -> bool {
        self.delete_selection();

        let current_length = self.value.chars().count();

        let remaining = match self.max_length {
            Some(max) => max.saturating_sub(current_length),
            None => text.chars().count(),
        };

        if remaining == 0 {
            return false;
        }

        let text: String = text.chars().take(remaining).collect();

        if text.is_empty() {
            return false;
        }

        let chars: Vec<char> = self.value.chars().collect();

        let mut result = String::with_capacity(self.value.len() + text.len());

        for (i, c) in chars.iter().enumerate() {
            if i == self.cursor {
                result.push_str(&text);
            }

            result.push(*c);
        }

        if self.cursor == chars.len() {
            result.push_str(&text);
        }

        self.value = result;
        self.cursor += text.chars().count();

        self.reset_caret();

        true
    }

    pub fn backspace(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }

        if self.cursor == 0 {
            return false;
        }

        let chars: Vec<char> = self.value.chars().collect();

        self.value = chars
            .into_iter()
            .enumerate()
            .filter_map(|(i, c)| if i == self.cursor - 1 { None } else { Some(c) })
            .collect();

        self.cursor -= 1;
        self.reset_caret();

        true
    }

    pub fn delete_forward(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }

        let len = self.value.chars().count();

        if self.cursor >= len {
            return false;
        }

        self.value = self
            .value
            .chars()
            .enumerate()
            .filter_map(|(i, c)| if i == self.cursor { None } else { Some(c) })
            .collect();

        self.reset_caret();

        true
    }
}

impl TextareaConfig {
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            max_length: None,
            initial_value: String::new(),
            height: 140.0,
        }
    }

    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    pub fn initial_value(mut self, initial_value: impl Into<String>) -> Self {
        self.initial_value = initial_value.into();
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

pub fn spawn_textarea(parent: &mut ChildSpawnerCommands, config: TextareaConfig) -> Entity {
    let mut textarea = Textarea::new(config.placeholder.clone()).with_value(config.initial_value);
    let accessible_label = config.placeholder.clone();

    if let Some(max_length) = config.max_length {
        textarea = textarea.max_length(max_length);
    }

    parent
        .spawn((
            Button,
            a11y::TabIndex(0),
            a11y::multiline_text_input_node(accessible_label),
            textarea,
            Node {
                width: percent(100),
                min_height: px(config.height),
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::FlexStart,
                padding: UiRect::all(px(14.0)),
                border: UiRect::all(px(1.0)),
                border_radius: BorderRadius::all(px(0.0)),
                ..default()
            },
                BackgroundColor(Color::NONE),
                BorderColor::all(Color::NONE),
                Surface::rounded_rect_fill(
                    0.0,
                    Paint::solid(Color::srgb(0.96, 0.96, 0.96)),
                )
                .uniform_border(1.0, Paint::solid(Color::srgb(0.83, 0.83, 0.81))),
            TextareaSurface,
        ))
        .with_children(|textarea_node| {
            textarea_node.spawn((
                ThemedText::new(TextRole::Body),
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(Color::srgb(0.12, 0.12, 0.14)),
                TextareaText,
                Node { ..default() },
            ));

            textarea_node.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new(config.placeholder),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(Color::srgb(0.62, 0.62, 0.60)),
                TextareaPlaceholder,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(14.0),
                    top: px(14.0),
                    ..default()
                },
            ));
        })
        .id()
}
