//! Platform-independent asset collection: copies a project's runtime asset
//! directory into a packaging destination. Knows nothing about *where*
//! inside a platform bundle assets end up.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

/// Copies `source` (a directory) to `destination`, replacing any existing
/// contents at `destination` first so stale files don't linger between publishes.
pub fn collect_assets(source: &Path, destination: &Path) -> Result<()> {
    if destination.exists() {
        fs::remove_dir_all(destination).with_context(|| {
            format!("failed to clear stale assets at {}", destination.display())
        })?;
    }
    copy_dir_recursive(source, destination).with_context(|| {
        format!(
            "failed to copy assets from {} to {}",
            source.display(),
            destination.display()
        )
    })
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
