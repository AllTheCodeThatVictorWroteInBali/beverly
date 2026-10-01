//! Platform-independent asset collection: copies a project's runtime asset
//! directory into a packaging destination. Knows nothing about *where*
//! inside a platform bundle assets end up.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};

/// Copies `source` (a directory) to `destination`, replacing any existing
/// contents at `destination` first so stale files don't linger between publishes.
pub fn collect_assets(source: &Path, destination: &Path) -> Result<()> {
    let source = fs::canonicalize(source)
        .with_context(|| format!("failed to resolve asset source {}", source.display()))?;
    let destination = canonicalize_with_missing_tail(destination)?;

    if destination.starts_with(&source) || source.starts_with(&destination) {
        bail!(
            "asset source {} and destination {} must not overlap",
            source.display(),
            destination.display()
        );
    }

    if destination.exists() {
        fs::remove_dir_all(&destination).with_context(|| {
            format!("failed to clear stale assets at {}", destination.display())
        })?;
    }
    copy_dir_recursive(&source, &destination).with_context(|| {
        format!(
            "failed to copy assets from {} to {}",
            source.display(),
            destination.display()
        )
    })
}

fn canonicalize_with_missing_tail(path: &Path) -> Result<std::path::PathBuf> {
    let mut ancestor = path;
    let mut missing_components = Vec::new();

    while !ancestor.exists() {
        let name = ancestor
            .file_name()
            .with_context(|| format!("could not resolve destination {}", path.display()))?;
        missing_components.push(name.to_os_string());
        ancestor = ancestor
            .parent()
            .with_context(|| format!("could not resolve destination {}", path.display()))?;
    }

    let mut canonical = fs::canonicalize(ancestor)
        .with_context(|| format!("failed to resolve destination {}", path.display()))?;
    for component in missing_components.into_iter().rev() {
        canonical.push(component);
    }

    Ok(canonical)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dst_path)?;
        } else if file_type.is_file() {
            fs::copy(entry.path(), &dst_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn refuses_destination_inside_asset_source() {
        let temp = tempdir().unwrap();
        let source = temp.path().join("assets");
        fs::create_dir_all(&source).unwrap();
        let original = source.join("image.png");
        fs::write(&original, b"asset").unwrap();

        let destination = source.join("dist/App.app/Contents/Resources/assets");
        let error = collect_assets(&source, &destination).unwrap_err();

        assert!(error.to_string().contains("must not overlap"));
        assert_eq!(fs::read(original).unwrap(), b"asset");
    }

    #[test]
    fn copies_assets_to_separate_destination() {
        let temp = tempdir().unwrap();
        let source = temp.path().join("assets");
        let destination = temp.path().join("bundle/Resources/assets");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("image.png"), b"asset").unwrap();

        collect_assets(&source, &destination).unwrap();

        assert_eq!(fs::read(destination.join("image.png")).unwrap(), b"asset");
    }
}
