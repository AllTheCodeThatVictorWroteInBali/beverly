pub mod atlas;
pub mod component;
pub mod debug;
pub mod font;
pub mod hit_test;
pub mod layout;
pub mod render;
pub mod shaping;
pub mod typography;

#[allow(unused_imports)]
pub use component::{HighlightableText, TextRole, ThemedText, ThemedTextPlugin};
pub use debug::{
	TypographyDebugPlugin,
	TypographyDebugSettings,
};
pub use atlas::{
	GlyphAtlasCache,
	GlyphAtlasKey,
	GlyphAtlasRegion,
	GlyphRasterizationMode,
	TypographyGlyphAtlasPlugin,
};
pub use font::{
	FontStyle,
	FontWeight,
	LoadedFontFace,
	TypographyFontManager,
	TypographyFontManagerPlugin,
};
pub use layout::{
	GlyphPlacement,
	GlyphRun,
	TextAlignment,
	TextDirection,
	TextLayoutBlock,
	TextLayoutCache,
	TextLayoutKey,
	TextLine,
	TextMetrics,
	TextWrapping,
	TypographyLayoutPlugin,
};
pub use render::{
	GlyphInstance,
	TextRenderItem,
	TextRenderItemPlugin,
};
pub use shaping::{
	GlyphCluster,
	ShapedText,
	TextShapingCache,
	TextShapingKey,
	TypographyShapingPlugin,
};
pub use typography::{
	FontFamily,
	LineHeight,
	Typography,
	TypographyPlugin,
};
