//! Platform-independent release build step: invokes Cargo and locates the
//! resulting binary. Knows nothing about `.app` bundles or any other
//! platform-specific packaging.

use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result, bail};

use super::metadata::ProjectMetadata;

/// Sensible release-profile defaults for a *published* binary: fat LTO +
/// single codegen unit for better cross-crate inlining/dead-code elimination,
/// symbol stripping, and abort-on-panic to drop unwind tables. Passed via
/// `--config` (not written into the consumer's `Cargo.toml`) so any profile
/// field the consumer's own manifest already sets explicitly still wins -
/// these are only used to fill in gaps.
const RELEASE_PROFILE_DEFAULTS: &[(&str, &str)] = &[
    ("profile.release.lto", "\"fat\""),
    ("profile.release.codegen-units", "1"),
    ("profile.release.strip", "true"),
    ("profile.release.panic", "\"abort\""),
];

/// Runs `cargo build --release` for the target project's binary and returns
/// the path to the resulting executable.
pub fn build_release(metadata: &ProjectMetadata) -> Result<PathBuf> {
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--release")
        .arg("--manifest-path")
        .arg(metadata.manifest_dir.join("Cargo.toml"))
        .arg("--bin")
        .arg(&metadata.bin_name);
    for (key, value) in RELEASE_PROFILE_DEFAULTS {
        cmd.arg("--config").arg(format!("{key}={value}"));
    }
    let status = cmd
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
