pub mod component;
mod embedded;

pub use component::{FeatherIconsPlugin, Icon, IconCommands, IconNode, IconProxyOcclusionRoot};
pub(crate) use embedded::embed_feather_icons;
