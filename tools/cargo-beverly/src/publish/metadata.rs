//! Platform-independent Cargo metadata discovery for the project being
//! published: package name, version, manifest/target directories, the binary
//! target to build, and any Beverly-specific packaging overrides.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

/// Resolved facts about the project to publish, independent of build/target/platform.
#[derive(Debug, Clone)]
pub struct ProjectMetadata {
    pub package_name: String,
    pub version: String,
    pub manifest_dir: PathBuf,
    pub target_dir: PathBuf,
    /// Name of the `[[bin]]` target to build (matches the compiled binary's file name on Unix).
    pub bin_name: String,
    /// Source directory of runtime assets to bundle, if the project has one.
    pub assets_dir: Option<PathBuf>,
    pub display_name: String,
    pub pascal_name: String,
    pub bundle_id: String,
}

/// Optional `[package.metadata.beverly]` overrides in the target project's `Cargo.toml`.
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
struct BeverlyPackageMetadata {
    display_name: Option<String>,
    bundle_id: Option<String>,
    assets_dir: Option<String>,
}

/// Runs `cargo metadata` (optionally rooted at `manifest_path`) and resolves
/// everything the publish pipeline needs from it.
pub fn discover(manifest_path: Option<&Path>) -> Result<ProjectMetadata> {
    let mut cmd = cargo_metadata::MetadataCommand::new();
    cmd.no_deps();
    if let Some(path) = manifest_path {
        cmd.manifest_path(path);
    }
    let metadata = cmd.exec().context("failed to run `cargo metadata`")?;

    let package = metadata
        .root_package()
        .context("no root package found (run `cargo beverly publish` from within a package, not a virtual workspace)")?;

    let bin_name = package
        .targets
        .iter()
        .find(|target| target.kind.contains(&cargo_metadata::TargetKind::Bin))
        .map(|target| target.name.clone())
        .context("failed to locate a release binary target (`[[bin]]`) to publish")?;

    let beverly_meta: BeverlyPackageMetadata = package
        .metadata
        .get("beverly")
        .cloned()
        .map(serde_json::from_value)
        .transpose()
        .context("invalid [package.metadata.beverly] section")?
        .unwrap_or_default();

    let manifest_dir = package
        .manifest_path
        .parent()
        .context("package manifest has no parent directory")?
        .as_std_path()
        .to_path_buf();

    let assets_dir = match beverly_meta.assets_dir {
        Some(dir) => Some(manifest_dir.join(dir)),
        None => {
            let default_dir = manifest_dir.join("assets");
            default_dir.is_dir().then_some(default_dir)
        }
    };

    Ok(ProjectMetadata {
        package_name: package.name.to_string(),
        version: package.version.to_string(),
        display_name: beverly_meta
            .display_name
            .unwrap_or_else(|| display_name(&package.name)),
        pascal_name: pascal_case_name(&package.name),
        bundle_id: beverly_meta
            .bundle_id
            .unwrap_or_else(|| default_bundle_id(&package.name)),
        manifest_dir,
        target_dir: metadata.target_directory.into_std_path_buf(),
        bin_name,
        assets_dir,
    })
}

/// Splits a package name on `-`/`_` word separators.
fn words(package_name: &str) -> impl Iterator<Item = &str> {
    package_name
        .split(['-', '_'])
        .filter(|word| !word.is_empty())
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// `my-app` -> `My App`.
pub fn display_name(package_name: &str) -> String {
    words(package_name)
        .map(capitalize)
        .collect::<Vec<_>>()
        .join(" ")
}

/// `my-app` -> `MyApp`. Used for the `.app` bundle and executable file names.
pub fn pascal_case_name(package_name: &str) -> String {
    words(package_name).map(capitalize).collect()
}

/// `my-app` -> `com.beverlyui.my-app`.
pub fn default_bundle_id(package_name: &str) -> String {
    format!("com.beverlyui.{package_name}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_name_title_cases_words() {
        assert_eq!(display_name("my-app"), "My App");
        assert_eq!(display_name("hello_world"), "Hello World");
        assert_eq!(display_name("showcase"), "Showcase");
    }

    #[test]
    fn pascal_case_name_strips_separators() {
        assert_eq!(pascal_case_name("my-app"), "MyApp");
        assert_eq!(pascal_case_name("hello_world"), "HelloWorld");
        assert_eq!(pascal_case_name("showcase"), "Showcase");
    }

    #[test]
    fn default_bundle_id_uses_reverse_dns_prefix() {
        assert_eq!(default_bundle_id("my-app"), "com.beverlyui.my-app");
    }
}
