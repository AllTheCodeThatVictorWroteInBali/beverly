use std::collections::HashMap;
use std::sync::Arc;

use bevy::prelude::*;

use super::typography::Typography;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

impl Default for FontStyle {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FontWeight(pub u16);

impl Default for FontWeight {
    fn default() -> Self {
        Self::NORMAL
    }
}

impl FontWeight {
    pub const THIN: Self = Self(100);
    pub const EXTRA_LIGHT: Self = Self(200);
    pub const LIGHT: Self = Self(300);
    pub const NORMAL: Self = Self(400);
    pub const MEDIUM: Self = Self(500);
    pub const SEMIBOLD: Self = Self(600);
    pub const BOLD: Self = Self(700);
    pub const EXTRA_BOLD: Self = Self(800);
    pub const BLACK: Self = Self(900);

    fn distance(self, other: Self) -> u16 {
        self.0.abs_diff(other.0)
    }
}

#[derive(Clone, Debug)]
pub struct LoadedFontFace {
    pub family: String,
    pub weight: FontWeight,
    pub style: FontStyle,
    pub asset_path: String,
    pub handle: Handle<Font>,
    pub bytes: Arc<Vec<u8>>,
}

#[derive(Resource, Debug, Default)]
pub struct TypographyFontManager {
    initialized: bool,
    by_family: HashMap<String, Vec<LoadedFontFace>>,
    fallback_chains: HashMap<String, Vec<String>>,
}

impl TypographyFontManager {
    pub fn register_face(
        &mut self,
        family: impl Into<String>,
        weight: FontWeight,
        style: FontStyle,
        asset_path: impl Into<String>,
        handle: Handle<Font>,
        bytes: Arc<Vec<u8>>,
    ) {
        let family = family.into();
        self.by_family
            .entry(family.clone())
            .or_default()
            .push(LoadedFontFace {
                family,
                weight,
                style,
                asset_path: asset_path.into(),
                handle,
                bytes,
            });
    }

    pub fn set_fallback_chain(&mut self, primary_family: impl Into<String>, chain: Vec<String>) {
        self.fallback_chains.insert(primary_family.into(), chain);
    }

    pub fn resolve_face(
        &self,
        family: &str,
        weight: FontWeight,
        style: FontStyle,
    ) -> Option<&LoadedFontFace> {
        let mut families = vec![family.to_string()];
        if let Some(chain) = self.fallback_chains.get(family) {
            families.extend(chain.iter().cloned());
        }

        for candidate_family in families {
            let Some(faces) = self.by_family.get(&candidate_family) else {
                continue;
            };

            if let Some(best_style) = faces
                .iter()
                .filter(|face| face.style == style)
                .min_by_key(|face| face.weight.distance(weight))
            {
                return Some(best_style);
            }

            if let Some(best_any_style) = faces
                .iter()
                .min_by_key(|face| face.weight.distance(weight))
            {
                return Some(best_any_style);
            }
        }

        None
    }

    pub fn face_count(&self) -> usize {
        self.by_family.values().map(|faces| faces.len()).sum()
    }

    pub fn face_by_asset_path(&self, asset_path: &str) -> Option<&LoadedFontFace> {
        self.by_family
            .values()
            .flat_map(|faces| faces.iter())
            .find(|face| face.asset_path == asset_path)
    }
}

#[derive(Component, Clone, Debug, Default)]
pub struct ResolvedFontFace {
    pub family: String,
    pub asset_path: String,
    pub weight: FontWeight,
    pub style: FontStyle,
}

fn initialize_font_manager(
    mut manager: ResMut<TypographyFontManager>,
    asset_server: Res<AssetServer>,
) {
    if manager.initialized {
        return;
    }

    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let asset_root = manifest_dir.join("assets");

    let read_font_bytes = |asset_path: &str| -> Arc<Vec<u8>> {
        let disk_path = asset_root.join(asset_path);
        match std::fs::read(&disk_path) {
            Ok(bytes) => Arc::new(bytes),
            Err(error) => {
                warn!("failed to read font bytes from {}: {error}", disk_path.display());
                Arc::new(Vec::new())
            }
        }
    };

    let sfns = asset_server.load("fonts/SFNS.ttf");
    manager.register_face(
        "SFNS",
        FontWeight::NORMAL,
        FontStyle::Normal,
        "fonts/SFNS.ttf",
        sfns,
        read_font_bytes("fonts/SFNS.ttf"),
    );

    let symbols = asset_server.load("fonts/AppleSymbols.ttf");
    manager.register_face(
        "AppleSymbols",
        FontWeight::NORMAL,
        FontStyle::Normal,
        "fonts/AppleSymbols.ttf",
        symbols,
        read_font_bytes("fonts/AppleSymbols.ttf"),
    );

    manager.set_fallback_chain(
        "SFNS",
        vec!["AppleSymbols".to_string()],
    );

    manager.initialized = true;
}

fn resolve_text_font_faces(
    manager: Res<TypographyFontManager>,
    mut query: Query<(&Typography, &mut TextFont, &mut ResolvedFontFace)>,
) {
    for (typography, mut text_font, mut resolved) in &mut query {
        let family = &typography.family.0;
        let Some(face) = manager.resolve_face(family, typography.weight, typography.style) else {
            continue;
        };

        if resolved.asset_path != face.asset_path
            || resolved.weight != face.weight
            || resolved.style != face.style
            || resolved.family != face.family
        {
            text_font.font = FontSource::Handle(face.handle.clone());
            resolved.family = face.family.clone();
            resolved.asset_path = face.asset_path.clone();
            resolved.weight = face.weight;
            resolved.style = face.style;
        }
    }
}

fn ensure_resolved_face_component(world: &mut World) {
    let entities: Vec<Entity> = {
        let mut query = world.query_filtered::<Entity, (With<Typography>, Without<ResolvedFontFace>)>();
        query.iter(world).collect()
    };

    for entity in entities {
        if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
            entity_mut.insert(ResolvedFontFace::default());
        }
    }
}

pub struct TypographyFontManagerPlugin;

impl Plugin for TypographyFontManagerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TypographyFontManager>()
            .add_systems(Startup, initialize_font_manager)
            .add_systems(
                Update,
                (ensure_resolved_face_component, resolve_text_font_faces).chain(),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_prefers_primary_family_then_fallback() {
        let mut manager = TypographyFontManager::default();

        manager.register_face(
            "Primary",
            FontWeight::NORMAL,
            FontStyle::Normal,
            "fonts/primary.ttf",
            Handle::default(),
            Arc::new(vec![0u8]),
        );
        manager.register_face(
            "Fallback",
            FontWeight::NORMAL,
            FontStyle::Normal,
            "fonts/fallback.ttf",
            Handle::default(),
            Arc::new(vec![0u8]),
        );
        manager.set_fallback_chain("Missing", vec!["Fallback".to_string()]);

        let primary = manager.resolve_face("Primary", FontWeight::NORMAL, FontStyle::Normal);
        assert_eq!(primary.map(|face| face.family.as_str()), Some("Primary"));

        let fallback = manager.resolve_face("Missing", FontWeight::NORMAL, FontStyle::Normal);
        assert_eq!(fallback.map(|face| face.family.as_str()), Some("Fallback"));
    }

    #[test]
    fn resolve_picks_nearest_weight_for_style() {
        let mut manager = TypographyFontManager::default();

        manager.register_face(
            "SFNS",
            FontWeight::LIGHT,
            FontStyle::Normal,
            "fonts/sfns-light.ttf",
            Handle::default(),
            Arc::new(vec![0u8]),
        );
        manager.register_face(
            "SFNS",
            FontWeight::BOLD,
            FontStyle::Normal,
            "fonts/sfns-bold.ttf",
            Handle::default(),
            Arc::new(vec![0u8]),
        );

        let face = manager
            .resolve_face("SFNS", FontWeight::MEDIUM, FontStyle::Normal)
            .expect("expected a resolved face");

        assert_eq!(face.weight, FontWeight::LIGHT);
    }
}
