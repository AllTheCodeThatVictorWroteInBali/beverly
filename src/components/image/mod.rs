use bevy::asset::io::Reader;
use bevy::asset::{AssetApp, AssetEvent, AssetLoader, LoadContext, LoadState, RenderAssetUsages};
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::collections::HashMap;

use crate::primitives::semantic::{SemanticNode, SemanticRole};

const MAX_ENCODED_IMAGE_BYTES: usize = 16 * 1024 * 1024;
const MAX_DECODED_IMAGE_PIXELS: u64 = 16 * 1024 * 1024;

fn normalize_source_path(path: &str) -> Result<String, String> {
    match url::Url::parse(path) {
        Ok(mut url) if matches!(url.scheme(), "http" | "https") => {
            if url.host_str().is_none() || !url.username().is_empty() || url.password().is_some() {
                return Err(
                    "HTTP(S) image URLs require a host and cannot contain credentials".to_owned(),
                );
            }
            url.set_fragment(None);
            Ok(url.to_string())
        }
        Ok(url) if path.contains("://") && !matches!(url.scheme(), "embedded" | "file") => {
            Err(format!(
                "Unsupported image URL scheme {:?}; supported sources are HTTP(S), embedded assets, and local paths",
                url.scheme()
            ))
        }
        _ => Ok(path.to_owned()),
    }
}

fn detect_web_image_extension(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("png")
    } else if bytes.starts_with(&[255, 216, 255]) {
        Some("jpg")
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some("gif")
    } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
        Some("webp")
    } else if bytes.starts_with(b"BM") {
        Some("bmp")
    } else if bytes.starts_with(&[0, 0, 1, 0]) {
        Some("ico")
    } else {
        None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImageSource {
    Path(String),
    Handle(Handle<Image>),
}

impl From<String> for ImageSource {
    fn from(value: String) -> Self {
        Self::Path(value)
    }
}

impl From<&str> for ImageSource {
    fn from(value: &str) -> Self {
        Self::Path(value.to_owned())
    }
}

impl From<Handle<Image>> for ImageSource {
    fn from(value: Handle<Image>) -> Self {
        Self::Handle(value)
    }
}

#[derive(Component, Clone, Debug, Default, PartialEq)]
pub struct ImageConfig {
    pub source: Option<ImageSource>,
    pub alt: Option<String>,
    pub decorative: bool,
}

impl ImageConfig {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn src(mut self, source: impl Into<ImageSource>) -> Self {
        self.source = Some(source.into());
        self
    }
    pub fn alt(mut self, alt: impl Into<String>) -> Self {
        self.alt = Some(alt.into());
        self
    }
    pub fn decorative(mut self, decorative: bool) -> Self {
        self.decorative = decorative;
        self
    }
}

#[derive(Component, Clone, Debug, PartialEq)]
pub enum ImageStatus {
    Empty,
    Loading,
    Ready,
    Failed(String),
}

pub struct WebImagePlugin;

#[derive(Asset, TypePath)]
struct DecodedWebImage(Option<Image>);

#[derive(Default, TypePath)]
struct WebImageLoader;

impl AssetLoader for WebImageLoader {
    type Asset = DecodedWebImage;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &(),
        _: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        decode_web_image(&bytes)
            .map(|image| DecodedWebImage(Some(image)))
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
    }
}

fn decode_web_image(bytes: &[u8]) -> Result<Image, String> {
    if bytes.len() > MAX_ENCODED_IMAGE_BYTES {
        return Err("Image exceeds the 16 MiB encoded-size limit".to_owned());
    }
    let extension = if let Some(extension) = detect_web_image_extension(bytes) {
        extension
    } else {
        let prefix = String::from_utf8_lossy(&bytes[..bytes.len().min(4096)]).to_ascii_lowercase();
        if prefix
            .trim_start_matches('\u{feff}')
            .trim_start()
            .contains("<svg")
        {
            let mut options = resvg::usvg::Options::default();
            options.image_href_resolver.resolve_string = Box::new(|_, _| None);
            let tree =
                resvg::usvg::Tree::from_data(bytes, &options).map_err(|error| error.to_string())?;
            let size = tree.size().to_int_size();
            if u64::from(size.width()) * u64::from(size.height()) > MAX_DECODED_IMAGE_PIXELS {
                return Err("SVG exceeds the 16 megapixel raster-size limit".to_owned());
            }
            let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width(), size.height())
                .ok_or("Invalid SVG dimensions")?;
            resvg::render(
                &tree,
                resvg::tiny_skia::Transform::identity(),
                &mut pixmap.as_mut(),
            );
            let mut pixels = Vec::with_capacity(pixmap.data().len());
            for pixel in pixmap.pixels() {
                let pixel = pixel.demultiply();
                pixels.extend_from_slice(&[
                    pixel.red(),
                    pixel.green(),
                    pixel.blue(),
                    pixel.alpha(),
                ]);
            }
            return Ok(Image::new(
                Extent3d {
                    width: size.width(),
                    height: size.height(),
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                pixels,
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::default(),
            ));
        }
        return Err("Unsupported web image. Supported formats are PNG, JPEG, GIF, WebP, BMP, ICO, and SVG; AVIF is not supported by Bevy's decoder.".to_owned());
    };
    let image = Image::from_buffer(
        bytes,
        ImageType::Extension(extension),
        CompressedImageFormats::NONE,
        true,
        ImageSampler::Default,
        RenderAssetUsages::default(),
    )
    .map_err(|error| error.to_string())?;
    let size = image.texture_descriptor.size;
    if u64::from(size.width) * u64::from(size.height) > MAX_DECODED_IMAGE_PIXELS {
        return Err("Image exceeds the 16 megapixel decoded-size limit".to_owned());
    }
    Ok(image)
}

#[derive(Component)]
struct PendingImage(Handle<DecodedWebImage>);

#[derive(Resource, Default)]
struct ImageTextures(HashMap<AssetId<DecodedWebImage>, Handle<Image>>);

impl Plugin for WebImagePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<DecodedWebImage>()
            .init_asset::<Image>()
            .register_asset_loader(WebImageLoader)
            .init_resource::<ImageTextures>()
            .add_systems(Update, (sync_images, finish_images).chain());
    }
}

pub fn spawn_image(parent: &mut ChildSpawnerCommands, config: ImageConfig) -> Entity {
    if !config.decorative && config.alt.as_ref().is_none_or(|alt| alt.trim().is_empty()) {
        bevy::log::warn!(
            "Non-decorative image requires descriptive `.alt(...)` text or `.decorative(true)`."
        );
    }
    parent
        .spawn((
            config,
            ImageStatus::Empty,
            ImageNode::default(),
            Node::default(),
            SemanticNode::new(SemanticRole::Image),
        ))
        .id()
}

pub fn image() -> crate::primitives::composition::UiElement {
    crate::primitives::composition::UiElement::Image(ImageConfig::default())
}

fn sync_images(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut images: Query<
        (
            Entity,
            &ImageConfig,
            &mut ImageNode,
            &mut SemanticNode,
            &mut ImageStatus,
        ),
        Changed<ImageConfig>,
    >,
) {
    for (entity, config, mut image, mut semantic, mut status) in &mut images {
        semantic.label = config.alt.clone();
        semantic.decorative = config.decorative;
        semantic.actions.clear();
        image.image = Handle::default();
        commands.entity(entity).remove::<PendingImage>();
        match &config.source {
            Some(ImageSource::Path(path)) if !path.trim().is_empty() => {
                let path = match normalize_source_path(path) {
                    Ok(path) => path,
                    Err(error) => {
                        *status = ImageStatus::Failed(error);
                        semantic.state.busy = false;
                        continue;
                    }
                };
                commands
                    .entity(entity)
                    .insert(PendingImage(assets.load(path)));
                *status = ImageStatus::Loading;
                semantic.state.busy = true;
            }
            Some(ImageSource::Handle(handle)) => {
                image.image = handle.clone();
                *status = ImageStatus::Loading;
                semantic.state.busy = true;
            }
            _ => {
                *status = ImageStatus::Empty;
                semantic.state.busy = false;
            }
        }
    }
}

fn finish_images(
    server: Res<AssetServer>,
    mut decoded: ResMut<Assets<DecodedWebImage>>,
    mut textures: ResMut<Assets<Image>>,
    mut cache: ResMut<ImageTextures>,
    mut events: MessageReader<AssetEvent<DecodedWebImage>>,
    mut images: Query<(
        &mut ImageNode,
        &mut ImageStatus,
        &mut SemanticNode,
        Option<&PendingImage>,
    )>,
) {
    for event in events.read() {
        if let AssetEvent::Removed { id } = event {
            cache.0.remove(id);
        }
    }
    for (mut image, mut status, mut semantic, pending) in &mut images {
        if let Some(pending) = pending {
            if let Some(mut loaded) = decoded.get_mut(&pending.0) {
                if let Some(texture) = loaded.0.take() {
                    let texture = textures.add(texture);
                    cache.0.insert(pending.0.id(), texture);
                }
                if let Some(texture) = cache.0.get(&pending.0.id()) {
                    image.image = texture.clone();
                    *status = ImageStatus::Ready;
                    semantic.state.busy = false;
                }
            } else if let Some(LoadState::Failed(error)) = server.get_load_state(pending.0.id()) {
                *status = ImageStatus::Failed(error.to_string());
                semantic.state.busy = false;
            }
        } else if !matches!(*status, ImageStatus::Empty | ImageStatus::Failed(_)) {
            if textures.contains(image.image.id()) {
                *status = ImageStatus::Ready;
                semantic.state.busy = false;
            } else if let Some(LoadState::Failed(error)) = server.get_load_state(image.image.id()) {
                *status = ImageStatus::Failed(error.to_string());
                semantic.state.busy = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_sources_accept_local_embedded_external_and_generated_assets() {
        for path in [
            "photos/report.png",
            "embedded://example/logo.webp",
            "http://localhost/image.jpg",
            "https://example.com/image.png",
        ] {
            let config = ImageConfig::new().src(path).alt("Preview");
            assert_eq!(config.source, Some(ImageSource::Path(path.to_owned())));
            assert_eq!(config.alt.as_deref(), Some("Preview"));
        }
        let handle = Handle::<Image>::default();
        assert_eq!(
            ImageConfig::new().src(handle.clone()).source,
            Some(ImageSource::Handle(handle))
        );
    }

    #[test]
    fn urls_are_normalized_and_credentials_or_unknown_schemes_are_rejected() {
        assert_eq!(
            normalize_source_path("HTTPS://Example.COM/image.webp?size=small#preview").unwrap(),
            "https://example.com/image.webp?size=small"
        );
        assert_eq!(
            normalize_source_path("embedded://example/assets/logo.svg").unwrap(),
            "embedded://example/assets/logo.svg"
        );
        assert!(normalize_source_path("https://user:pass@example.com/image.png").is_err());
        assert!(normalize_source_path("ftp://example.com/image.png").is_err());
    }

    #[test]
    fn svg_xml_prolog_and_webp_content_signature_are_detected() {
        let svg = br##"<?xml version="1.0"?><svg xmlns="http://www.w3.org/2000/svg" width="2" height="1"><rect width="2" height="1" fill="#ff0000"/></svg>"##;
        let image = decode_web_image(svg).unwrap();
        assert_eq!(
            (
                image.texture_descriptor.size.width,
                image.texture_descriptor.size.height
            ),
            (2, 1)
        );
        assert_eq!(
            detect_web_image_extension(b"RIFF\0\0\0\0WEBPVP8 "),
            Some("webp")
        );
    }

    #[test]
    fn bundled_svg_example_rasterizes_to_its_intrinsic_dimensions() {
        let svg = include_bytes!("../../../assets/examples/analytics.svg");
        let image = decode_web_image(svg).unwrap();
        assert_eq!(
            (
                image.texture_descriptor.size.width,
                image.texture_descriptor.size.height
            ),
            (960, 540)
        );
    }

    #[test]
    fn minimal_png_decodes_and_avif_fails_explicitly() {
        let png = include_bytes!("../../../assets/favicon_io/favicon-16x16.png");
        let image = decode_web_image(png).unwrap();
        assert_eq!(image.texture_descriptor.size.width, 16);
        assert_eq!(image.texture_descriptor.size.height, 16);
        assert!(
            decode_web_image(b"\0\0\0\x1cftypavif\0\0\0\0avifmif1")
                .unwrap_err()
                .contains("AVIF is not supported")
        );
    }
}
