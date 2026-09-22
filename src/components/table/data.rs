use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::component::Table;

/// Backend-facing JSON contract for table data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableResponse {
    pub id: String,
    pub columns: Vec<TableColumnData>,
    pub rows: Vec<TableRowData>,
    #[serde(default = "default_true")]
    pub show_header: bool,
    #[serde(default)]
    pub selectable_rows: bool,
    #[serde(default)]
    pub striped: bool,
    #[serde(default)]
    pub pagination: Option<TablePaginationData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumnData {
    pub id: String,
    pub label: String,
    /// Optional percentage width for this column (for example 30 means 30%).
    #[serde(default)]
    pub width: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRowData {
    pub id: String,
    /// Key = column ID
    pub cells: HashMap<String, TableCellData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TableCellData {
    Text(String),
    Number(f64),
    Boolean(bool),
    Null(Option<()>),
}

impl TableCellData {
    pub fn to_text(&self) -> String {
        match self {
            Self::Text(value) => value.clone(),
            Self::Number(value) => {
                if value.fract().abs() < f64::EPSILON {
                    format!("{value:.0}")
                } else {
                    format!("{value}")
                }
            }
            Self::Boolean(value) => {
                if *value {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            Self::Null(_) => String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TablePaginationData {
    pub page: u32,
    pub page_size: u32,
    pub total_items: u64,
    pub total_pages: u32,
}

fn default_true() -> bool {
    true
}

impl TableResponse {
    /// Deserializes from JSON without coupling the table component to a specific backend.
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }

    /// Convenience spawn that forwards data to the Bevy table renderer.
    pub fn spawn(&self, commands: &mut Commands) -> Entity {
        Table::spawn_from_data(commands, self)
    }
}
