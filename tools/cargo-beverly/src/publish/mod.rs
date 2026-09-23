//! `cargo beverly publish`: a platform-independent pipeline (metadata ->
//! build -> asset collection -> platform packaging) that delegates the
//! platform-specific step to a [`platform::PlatformPackager`].

pub mod assets;
pub mod build;
pub mod context;
pub mod metadata;
pub mod platform;

use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use clap::Args;

use context::PublishContext;
use platform::Target;

#[derive(Args, Debug)]
pub struct PublishArgs {
    /// Path to the Cargo.toml of the project to publish (defaults to the current directory).
    #[arg(long)]
    pub manifest_path: Option<PathBuf>,

    /// Logical target platform to publish for (defaults to the host platform).
    #[arg(long)]
    pub target: Option<Target>,

    /// Override the output directory (defaults to `dist/` next to the manifest).
    #[arg(long = "out-dir")]
    pub out_dir: Option<PathBuf>,
}

pub fn run(args: &PublishArgs) -> Result<()> {
    let project = metadata::discover(args.manifest_path.as_deref())?;
    println!("   Publishing {} v{}", project.package_name, project.version);

    println!("   Building release binary...");
    let binary_path = build::build_release(&project)?;
    println!("    Finished release build");

    let target = match args.target {
        Some(target) => target,
        None => Target::host()?,
    };
    let output_dir = args
        .out_dir
        .clone()
        .unwrap_or_else(|| project.manifest_dir.join("dist"));
    fs::create_dir_all(&output_dir)?;

    let ctx = PublishContext::new(&project, target, binary_path, output_dir);

    println!("\n   Packaging {} application...", target.label());
    let packager = platform::packager_for(target)?;
    let artifact = packager.package(&ctx)?;

    println!("   ✓ Binary");
    if artifact.assets_included {
        println!("   ✓ Assets");
    } else {
        println!("   • Assets (none found, skipped)");
    }
    println!("   ✓ Info.plist");

    println!("\n   Published:");
    println!("   {}", artifact.root.display());

    Ok(())
}
