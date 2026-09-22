// src/ui/clipboard/resource.rs

use bevy::prelude::*;

/// Cross-platform clipboard service.
///
/// The actual platform implementation can be swapped without
/// changing any UI components.
#[derive(Resource, Default)]
pub struct Clipboard {
    contents: String,
}

impl Clipboard {
    pub fn new() -> Self {
        Self {
            contents: String::new(),
        }
    }

    pub fn get(&self) -> &str {
        &self.contents
    }

    pub fn set(&mut self, text: impl Into<String>) {
        self.contents = text.into();
    }

    pub fn clear(&mut self) {
        self.contents.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.contents.is_empty()
    }
}
