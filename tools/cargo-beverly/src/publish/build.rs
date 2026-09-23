//! Platform-independent release build step: invokes Cargo and locates the
//! resulting binary. Knows nothing about `.app` bundles or any other
//! platform-specific packaging.

use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result, bail};

use super::metadata::ProjectMetadata;

/// Runs `cargo build --release` for the target project's binary and returns
/// the path to the resulting executable.
pub fn build_release(metadata: &ProjectMetadata) -> Result<PathBuf> {
    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--manifest-path")
        .arg(metadata.manifest_dir.join("Cargo.toml"))
        .arg("--bin")
        .arg(&metadata.bin_name)
        .status()
        .context("failed to invoke `cargo build`")?;

    if !status.success() {
        bail!("release build failed (see `cargo build` output above)");
    }

    let binary_path = metadata
        .target_dir
        .join("release")
        .join(&metadata.bin_name);

    if !binary_path.is_file() {
        bail!(
            "failed to locate release binary at {}",
            binary_path.display()
        );
    }

    Ok(binary_path)
}
