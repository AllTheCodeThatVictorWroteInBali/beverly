use bevy::a11y::AccessibilityNode;
use bevy::prelude::*;

use crate::rendering::{Border, Paint, Surface};
use crate::primitives::a11y;
use crate::theme::{ThemeResource, dark_theme};

/// Marker component for the pagination root.
#[derive(Component)]
pub struct Pagination;

/// Stores the current pagination state.
#[derive(Component, Debug, Clone)]
pub struct PaginationState {
    /// Current page, 1-indexed.
    pub current_page: usize,

    /// Number of records displayed per page.
    pub page_size: usize,

    /// Total number of pages.
    pub total_pages: usize,
}

impl PaginationState {
    pub fn new(page_size: usize, total_pages: usize) -> Self {
        Self {
            current_page: 1,
            page_size: page_size.max(1),
            total_pages: total_pages.max(1),
        }
    }

    pub fn has_previous(&self) -> bool {
        self.current_page > 1
    }

    pub fn has_next(&self) -> bool {
        self.current_page < self.total_pages
    }

    pub fn first_page(&mut self) {
        self.current_page = 1;
    }

    pub fn last_page(&mut self) {
        self.current_page = self.total_pages;
    }

    pub fn previous_page(&mut self) {
        if self.has_previous() {
            self.current_page -= 1;
        }
    }

    pub fn next_page(&mut self) {
        if self.has_next() {
            self.current_page += 1;
        }
    }

    pub fn set_page(&mut self, page: usize) {
        self.current_page = page.clamp(1, self.total_pages);
    }

    pub fn set_page_size(&mut self, page_size: usize) {
        self.page_size = page_size.max(1);
    }

    pub fn set_total_pages(&mut self, total_pages: usize) {
        self.total_pages = total_pages.max(1);

        if self.current_page > self.total_pages {
            self.current_page = self.total_pages;
        }
    }
}

/// Identifies the pagination action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaginationAction {
    First,
    Previous,
    Next,
    Last,
    Page(usize),
}

/// Message emitted whenever pagination changes.
#[derive(Message, Debug, Clone, Copy)]
pub struct PaginationEvent {
    /// The pagination component that generated the event.
    pub entity: Entity,

    /// New page number.
    pub page: usize,

    /// Current page size.
    pub page_size: usize,

    /// What caused the change.
    pub action: PaginationAction,
}

/// Internal button role.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PaginationButtonRole {
    First,
    Previous,
    Next,
    Last,
    PageSlot(usize),
}

/// Internal button marker.
#[derive(Component)]
struct PaginationButton {
    owner: Entity,
    role: PaginationButtonRole,
    action: PaginationAction,
}

#[derive(Component)]
struct PaginationButtonDisabled(pub bool);

/// Marker for per-button label text.
#[derive(Component)]
struct PaginationButtonLabel {
    button: Entity,
}

/// Marker for page summary text.
#[derive(Component)]
struct PaginationSummaryText {
    owner: Entity,
}

#[derive(Component, Clone)]
struct PaginationStyleConfig {
    show_first_last: bool,
    show_page_numbers: bool,
    max_page_buttons: usize,
}

/// Plugin for the pagination component.
pub struct PaginationPlugin;

impl Plugin for PaginationPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PaginationEvent>().add_systems(
            Update,
            (pagination_button_interaction, update_pagination_ui),
        );
    }
}

#[derive(Clone)]
pub struct PaginationConfig {
    pub page_size: usize,
    pub total_pages: usize,

    /// Show first/last buttons.
    pub show_first_last: bool,

    /// Show numbered page buttons.
    pub show_page_numbers: bool,

    /// Maximum number of page buttons shown at once.
    pub max_page_buttons: usize,
}

impl Default for PaginationConfig {
    fn default() -> Self {
        Self {
            page_size: 25,
            total_pages: 1,
            show_first_last: true,
            show_page_numbers: true,
            max_page_buttons: 7,
        }
    }
}

pub fn spawn_pagination(parent: &mut ChildSpawnerCommands, config: PaginationConfig) -> Entity {
    let default_colors = dark_theme().colors;

    let mut root_entity = parent.spawn((
        Pagination,
        PaginationState::new(config.page_size, config.total_pages),
        PaginationStyleConfig {
            show_first_last: config.show_first_last,
            show_page_numbers: config.show_page_numbers,
            max_page_buttons: config.max_page_buttons.max(1),
        },
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            padding: UiRect::axes(px(10.0), px(8.0)),
            border: UiRect::all(px(1.0)),
            border_radius: BorderRadius::all(px(14.0)),
            column_gap: Val::Px(6.0),
            ..default()
        },
        Surface::rounded_rect_border(
            14.0,
            default_colors.surface.with_alpha(0.62),
            1.0,
            default_colors.border.with_alpha(0.42),
        ),
    ));

    let root = root_entity.id();

    root_entity.with_children(|parent| {
        if config.show_first_last {
            spawn_pagination_button(
                parent,
                root,
                "<<",
                PaginationButtonRole::First,
                PaginationAction::First,
            );
        }

        spawn_pagination_button(
            parent,
            root,
            "<",
            PaginationButtonRole::Previous,
            PaginationAction::Previous,
        );

        if config.show_page_numbers {
            spawn_page_numbers(parent, root, &config);
        }

        spawn_pagination_button(
            parent,
            root,
            ">",
            PaginationButtonRole::Next,
            PaginationAction::Next,
        );

        if config.show_first_last {
            spawn_pagination_button(
                parent,
                root,
                ">>",
                PaginationButtonRole::Last,
                PaginationAction::Last,
            );
        }

        parent.spawn((
            PaginationSummaryText { owner: root },
            Text::new(format!("Page {} of {}", 1, config.total_pages.max(1))),
            TextFont {
                font_size: FontSize::Px(14.0),
                ..default()
            },
            TextColor(default_colors.text_muted),
            Node {
                margin: UiRect::left(px(6.0)),
                ..default()
            },
        ));
    });

    root
}

fn spawn_pagination_button(
    parent: &mut ChildSpawnerCommands,
    owner: Entity,
    label: &str,
    role: PaginationButtonRole,
    action: PaginationAction,
) {
    let default_colors = dark_theme().colors;
    let accessible_label = match role {
        PaginationButtonRole::First => "First page".to_string(),
        PaginationButtonRole::Previous => "Previous page".to_string(),
        PaginationButtonRole::Next => "Next page".to_string(),
        PaginationButtonRole::Last => "Last page".to_string(),
        PaginationButtonRole::PageSlot(slot) => format!("Page {}", slot + 1),
    };

    let mut button_entity = parent.spawn((
        Button,
        a11y::TabIndex(0),
        a11y::button_node(accessible_label),
        PaginationButton {
            owner,
            role,
            action,
        },
        PaginationButtonDisabled(false),
        Node {
            min_width: Val::Px(34.0),
            min_height: Val::Px(34.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(px(8.0)),
            border: UiRect::all(px(1.0)),
            border_radius: BorderRadius::all(px(10.0)),
            ..default()
        },
        Surface::rounded_rect_border(10.0, default_colors.surface, 1.0, default_colors.border),
    ));

    let button = button_entity.id();
    button_entity.with_children(|parent| {
        parent.spawn((
            PaginationButtonLabel { button },
            Text::new(label),
            TextFont {
                font_size: FontSize::Px(16.0),
                ..default()
            },
            TextColor(default_colors.text),
        ));
    });
}

fn spawn_page_numbers(parent: &mut ChildSpawnerCommands, owner: Entity, config: &PaginationConfig) {
    let count = config.max_page_buttons.max(1);

    for slot in 0..count {
        spawn_pagination_button(
            parent,
            owner,
            &(slot + 1).to_string(),
            PaginationButtonRole::PageSlot(slot),
            PaginationAction::Page(slot + 1),
        );
    }
}

fn pagination_button_interaction(
    mut query: Query<
        (&Interaction, &PaginationButton, &PaginationButtonDisabled),
        (Changed<Interaction>, With<Button>),
    >,

    mut pagination_query: Query<&mut PaginationState>,

    mut events: MessageWriter<PaginationEvent>,
) {
    for (interaction, button, disabled) in &mut query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if disabled.0 {
            continue;
        }

        let Ok(mut state) = pagination_query.get_mut(button.owner) else {
            continue;
        };

        let old_page = state.current_page;

        match button.action {
            PaginationAction::First => {
                state.first_page();
            }

            PaginationAction::Previous => {
                state.previous_page();
            }

            PaginationAction::Next => {
                state.next_page();
            }

            PaginationAction::Last => {
                state.last_page();
            }

            PaginationAction::Page(page) => {
                state.set_page(page);
            }
        }

        // Don't emit an event if the action didn't actually change
        // the current page.
        if state.current_page == old_page {
            continue;
        }

        events.write(PaginationEvent {
            entity: button.owner,
            page: state.current_page,
            page_size: state.page_size,
            action: button.action,
        });
    }
}

fn update_pagination_ui(
    theme: Option<Res<ThemeResource>>,
    pagination_query: Query<(Entity, &PaginationState, &PaginationStyleConfig), With<Pagination>>,
    mut button_query: Query<
        (
            Entity,
            &mut PaginationButton,
            &mut PaginationButtonDisabled,
            &mut Node,
            &mut Surface,
            &Interaction,
            &mut a11y::TabIndex,
            &mut AccessibilityNode,
        ),
        With<Button>,
    >,
    mut label_query: Query<
        (&PaginationButtonLabel, &mut Text, &mut TextColor),
        Without<PaginationSummaryText>,
    >,
    mut summary_query: Query<
        (&PaginationSummaryText, &mut Text, &mut TextColor),
        Without<PaginationButtonLabel>,
    >,
) {
    let colors = theme
        .as_ref()
        .map(|theme| theme.current.colors)
        .unwrap_or_else(|| dark_theme().colors);

    for (pagination_entity, state, style) in &pagination_query {
        let visible_buttons = state.total_pages.min(style.max_page_buttons.max(1));
        let page_window_start =
            page_window_start(state.current_page, state.total_pages, visible_buttons);

        for (
            button_entity,
            mut button,
            mut disabled,
            mut node,
            mut surface,
            interaction,
            mut tab_index,
            mut a11y_node,
        ) in &mut button_query
        {
            if button.owner != pagination_entity {
                continue;
            }

            let mut hidden = false;
            let mut is_active_page = false;
            let mut accessible_label: Option<String> = None;

            match button.role {
                PaginationButtonRole::First => {
                    if !style.show_first_last {
                        hidden = true;
                    }
                    button.action = PaginationAction::First;
                    disabled.0 = !state.has_previous();
                }
                PaginationButtonRole::Previous => {
                    button.action = PaginationAction::Previous;
                    disabled.0 = !state.has_previous();
                }
                PaginationButtonRole::Next => {
                    button.action = PaginationAction::Next;
                    disabled.0 = !state.has_next();
                }
                PaginationButtonRole::Last => {
                    if !style.show_first_last {
                        hidden = true;
                    }
                    button.action = PaginationAction::Last;
                    disabled.0 = !state.has_next();
                }
                PaginationButtonRole::PageSlot(slot) => {
                    if !style.show_page_numbers || slot >= visible_buttons {
                        hidden = true;
                        disabled.0 = true;
                        button.action = PaginationAction::Page(1);
                    } else {
                        let page_number = page_window_start + slot;
                        button.action = PaginationAction::Page(page_number);
                        disabled.0 = false;
                        is_active_page = page_number == state.current_page;
                        accessible_label = Some(format!("Page {page_number}"));
                    }
                }
            }

            node.display = if hidden { Display::None } else { Display::Flex };
            surface.border = Some(Border::new(1.0, Paint::solid(colors.border)));

            tab_index.0 = if hidden || disabled.0 { -1 } else { 0 };
            a11y::set_disabled(&mut a11y_node, disabled.0);
            if let Some(label) = accessible_label {
                a11y_node.0.set_label(label);
            }
            a11y_node.0.set_selected(is_active_page);

            if hidden {
                continue;
            }

            let bg = if disabled.0 {
                colors.surface_elevated.with_alpha(0.82)
            } else if is_active_page {
                colors.primary.with_alpha(0.96)
            } else {
                match *interaction {
                    Interaction::Pressed => colors.primary_active,
                    Interaction::Hovered => colors.secondary.with_alpha(0.88),
                    Interaction::None => colors.surface.with_alpha(0.72),
                }
            };
            surface.fill = Paint::solid(bg);

            for (label_ref, mut text, mut text_color) in &mut label_query {
                if label_ref.button != button_entity {
                    continue;
                }

                if let PaginationAction::Page(page_number) = button.action {
                    if matches!(button.role, PaginationButtonRole::PageSlot(_)) {
                        **text = page_number.to_string();
                    }
                }

                text_color.0 = if disabled.0 {
                    colors.text_disabled
                } else if is_active_page {
                    Color::WHITE
                } else {
                    colors.text
                };
            }
        }

        for (summary, mut text, mut text_color) in &mut summary_query {
            if summary.owner != pagination_entity {
                continue;
            }

            **text = format!("Page {} of {}", state.current_page, state.total_pages);
            text_color.0 = colors.text_muted;
        }
    }
}

fn page_window_start(current_page: usize, total_pages: usize, visible_buttons: usize) -> usize {
    if total_pages <= visible_buttons {
        return 1;
    }

    let half = visible_buttons / 2;
    let mut start = current_page.saturating_sub(half).max(1);
    let max_start = total_pages
        .saturating_sub(visible_buttons)
        .saturating_add(1);

    if start > max_start {
        start = max_start;
    }

    start.max(1)
}
