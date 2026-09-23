//! Platform packaging abstraction. [`Target`] is the logical, user-facing
//! platform/architecture selector; [`PlatformPackager`] is the trait each
//! platform implements to turn a built binary + assets into a distributable
//! artifact. Adding Windows/Linux later means adding a `Target` variant and a
//! new module here - the publish pipeline in `super` never changes.

pub mod macos;

use anyhow::{Result, bail};
use clap::ValueEnum;

use super::context::PublishContext;

/// Operating system family a [`Target`] belongs to. Used to select a packager.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    MacOS,
    #[allow(dead_code)] // constructed once Windows packaging exists
    Windows,
    #[allow(dead_code)] // constructed once Linux packaging exists
    Linux,
}

/// Logical publish target, e.g. `--target macos-arm64`. Deliberately separate
/// from a raw Rust target triple so the CLI surface stays stable even as the
/// triple mapping (and eventually cross-compilation) evolves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Target {
    #[value(name = "macos-arm64")]
    MacosArm64,
    #[value(name = "macos-x64")]
    MacosX64,
}

impl Target {
    /// Resolves the logical target matching the machine currently running this CLI.
    pub fn host() -> Result<Self> {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("macos", "aarch64") => Ok(Target::MacosArm64),
            ("macos", "x86_64") => Ok(Target::MacosX64),
            (os, arch) => bail!(
                "unsupported host platform `{os}-{arch}`: only macOS (arm64/x64) is supported today"
            ),
        }
    }

    pub fn os(self) -> Os {
        match self {
            Target::MacosArm64 | Target::MacosX64 => Os::MacOS,
        }
    }

    /// The Rust target triple this logical target maps to. Not passed to
    /// `cargo build` yet (no cross-compilation support), but kept ready for
    /// when it is.
    #[allow(dead_code)] // wired up once cross-compilation is implemented
    pub fn rust_triple(self) -> &'static str {
        match self {
            Target::MacosArm64 => "aarch64-apple-darwin",
            Target::MacosX64 => "x86_64-apple-darwin",
        }
    }

    pub fn label(self) -> &'static str {
        match self.os() {
            Os::MacOS => "macOS",
            Os::Windows => "Windows",
            Os::Linux => "Linux",
        }
    }
}

/// A produced distributable artifact (e.g. a macOS `.app` bundle).
pub struct PublishArtifact {
    /// Root of the produced artifact, e.g. `dist/MyApp.app`.
    pub root: std::path::PathBuf,
    pub assets_included: bool,
}

/// Packages a built binary + assets into a platform-specific distributable.
pub trait PlatformPackager {
    fn package(&self, ctx: &PublishContext) -> Result<PublishArtifact>;
}

/// Selects the packager for a logical [`Target`].
pub fn packager_for(target: Target) -> Result<Box<dyn PlatformPackager>> {
    match target.os() {
        Os::MacOS => Ok(Box::new(macos::MacOSPackager)),
        Os::Windows => bail!("Windows packaging is not implemented yet"),
        Os::Linux => bail!("Linux packaging is not implemented yet"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_triple_maps_logical_targets() {
        assert_eq!(Target::MacosArm64.rust_triple(), "aarch64-apple-darwin");
        assert_eq!(Target::MacosX64.rust_triple(), "x86_64-apple-darwin");
    }

    #[test]
    fn all_current_targets_resolve_to_macos() {
        assert_eq!(Target::MacosArm64.os(), Os::MacOS);
        assert_eq!(Target::MacosX64.os(), Os::MacOS);
    }
}
