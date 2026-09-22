use crate::components::input::{TextInputConfig, TextInputKind, spawn_text_input};
use crate::rendering::Surface;
use crate::components::search::events::update_search_visuals;
use crate::theme::ThemeResource;
use bevy::prelude::*;
use std::collections::HashMap;

// ============================================================
// SEARCH COMPONENT
// ============================================================

#[derive(Component)]
pub struct Search {
    pub id: String,
    pub placeholder: String,
    pub query: String,

    /// Whether the search currently has keyboard focus.
    pub focused: bool,

    /// Whether completion suggestions are visible.
    pub show_suggestions: bool,

    /// Currently highlighted suggestion.
    pub selected: Option<usize>,

    /// Maximum number of suggestions shown.
    pub max_suggestions: usize,
}

impl Search {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            placeholder: "Search...".into(),
            query: String::new(),
            focused: false,
            show_suggestions: false,
            selected: None,
            max_suggestions: 8,
        }
    }

    pub fn placeholder(mut self, value: impl Into<String>) -> Self {
        self.placeholder = value.into();
        self
    }

    pub fn max_suggestions(mut self, value: usize) -> Self {
        self.max_suggestions = value;
        self
    }
}

#[derive(Component)]
pub struct SearchBar;

#[derive(Component)]
pub struct SearchValueText;

#[derive(Component)]
pub struct SearchPlaceholderText;

pub fn spawn_search(
    parent: &mut ChildSpawnerCommands,
    _font: Handle<Font>,
    theme: &ThemeResource,
) -> Entity {
    let search = Search::new("navbar_search").placeholder("Search");
    let bg = theme.current.colors.surface_elevated;
    let _border = theme.current.colors.border;

    parent
        .spawn((
            SearchBar,
            search,
            Node {
                width: Val::Px(480.0),
                height: Val::Px(36.0),
                border_radius: BorderRadius::all(Val::Px(10.0)),
                overflow: Overflow::visible(),
                ..default()
            },
            Surface::rounded_rect_fill(10.0, bg),
        ))
        .with_children(|search_bar| {
            let _ = spawn_text_input(
                search_bar,
                TextInputConfig::new("Search")
                    .kind(TextInputKind::Search)
                    .max_length(120)
                    .border_radius(10.0),
            );
        })
        .id()
}

// ============================================================
// SUGGESTIONS
// ============================================================

#[derive(Clone, Debug)]
pub struct SearchSuggestion {
    pub value: String,
    pub label: String,

    /// Optional secondary information.
    pub description: Option<String>,

    /// Optional icon identifier.
    pub icon: Option<String>,
}

impl SearchSuggestion {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            description: None,
            icon: None,
        }
    }

    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    pub fn icon(mut self, value: impl Into<String>) -> Self {
        self.icon = Some(value.into());
        self
    }
}

// ============================================================
// COMPLETION STATE
// ============================================================

#[derive(Component, Default)]
pub struct SearchCompletion {
    pub suggestions: Vec<SearchSuggestion>,
}

// ============================================================
// EVENTS
// ============================================================

/// User changed the search text.
#[derive(Message, Debug, Clone)]
pub struct SearchChanged {
    pub search_id: String,
    pub query: String,
}

/// User submitted the search.
#[derive(Message, Debug, Clone)]
pub struct SearchSubmitted {
    pub search_id: String,
    pub query: String,
}

/// User selected a completion.
#[derive(Message, Debug, Clone)]
pub struct SearchSuggestionSelected {
    pub search_id: String,
    pub value: String,
}

/// Request for a completion provider to return suggestions.
#[derive(Message, Debug, Clone)]
pub struct SearchCompletionRequested {
    pub search_id: String,
    pub query: String,
}

// ============================================================
// COMPLETION PROVIDER TRAIT
// ============================================================

pub trait SearchCompletionProvider: Send + Sync + 'static {
    fn complete(&self, query: &str, limit: usize) -> Vec<SearchSuggestion>;
}

// ============================================================
// LOCAL COMPLETION PROVIDER
// ============================================================

pub struct LocalCompletionProvider {
    values: Vec<SearchSuggestion>,
}

impl LocalCompletionProvider {
    pub fn new(values: Vec<SearchSuggestion>) -> Self {
        Self { values }
    }
}

impl SearchCompletionProvider for LocalCompletionProvider {
    fn complete(&self, query: &str, limit: usize) -> Vec<SearchSuggestion> {
        if query.trim().is_empty() {
            return Vec::new();
        }

        let query = query.to_lowercase();

        self.values
            .iter()
            .filter(|item| item.label.to_lowercase().contains(&query))
            .take(limit)
            .cloned()
            .collect()
    }
}

// ============================================================
// PROVIDER RESOURCE
// ============================================================

#[derive(Resource)]
pub struct SearchProviders {
    providers: HashMap<String, Box<dyn SearchCompletionProvider>>,
}

impl SearchProviders {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn register(
        &mut self,
        search_id: impl Into<String>,
        provider: impl SearchCompletionProvider,
    ) {
        self.providers.insert(search_id.into(), Box::new(provider));
    }

    pub fn complete(&self, search_id: &str, query: &str, limit: usize) -> Vec<SearchSuggestion> {
        self.providers
            .get(search_id)
            .map(|provider| provider.complete(query, limit))
            .unwrap_or_default()
    }
}

// ============================================================
// SEARCH PLUGIN
// ============================================================

pub struct SearchPlugin;

impl Plugin for SearchPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SearchChanged>()
            .add_message::<SearchSubmitted>()
            .add_message::<SearchSuggestionSelected>()
            .add_message::<SearchCompletionRequested>()
            .insert_resource(SearchProviders::new())
            .add_systems(Update, update_search_visuals);
    }
}
