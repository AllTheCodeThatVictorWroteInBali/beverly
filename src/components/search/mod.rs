mod component;
mod events;

pub use component::{
    LocalCompletionProvider, Search, SearchChanged, SearchCompletion, SearchCompletionRequested,
    SearchPlugin, SearchProviders, SearchSubmitted, SearchSuggestion, SearchSuggestionSelected,
    spawn_search,
};
pub use events::{focus_search_bar, search_completion_system, search_keyboard_system};
