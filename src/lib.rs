//! Beverly is a Rust-native UI system for building high-performance,
//! accessible interfaces with [Bevy](https://bevyengine.org/).
//!
//! ```no_run
//! use bevy::prelude::*;
//! use beverly::prelude::*;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(BeverlyPlugin)
//!     .run();
//! ```

pub mod animation;
pub mod components;
pub mod icons;
mod plugin;
pub mod prelude;
pub mod primitives;
pub mod rendering;
pub mod theme;

pub use plugin::BeverlyPlugin;
