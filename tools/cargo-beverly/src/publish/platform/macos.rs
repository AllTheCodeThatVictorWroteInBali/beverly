//! macOS `.app` bundle packaging: `Contents/{MacOS,Resources}` layout and
//! `Info.plist` generation. Everything macOS-specific lives in this file;
//! nothing outside `platform/` knows about bundle layout.

use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

use super::{PlatformPackager, PublishArtifact};
use crate::publish::assets::collect_assets;
use crate::publish::context::PublishContext;

pub struct MacOSPackager;

impl PlatformPackager for MacOSPackager {
    fn package(&self, ctx: &PublishContext) -> Result<PublishArtifact> {
        let bundle_root = ctx.output_dir.join(format!("{}.app", ctx.pascal_name));
        // The bundle is an artifact Beverly fully owns: clear only this path
        // (never the whole output directory) so stale files don't linger.
        if bundle_root.exists() {
            fs::remove_dir_all(&bundle_root)
                .with_context(|| format!("failed to clear stale bundle at {}", bundle_root.display()))?;
        }

        let contents_dir = bundle_root.join("Contents");
        let macos_dir = contents_dir.join("MacOS");
        let resources_dir = contents_dir.join("Resources");
        fs::create_dir_all(&macos_dir)
            .with_context(|| format!("failed to create {}", macos_dir.display()))?;
        fs::create_dir_all(&resources_dir)
            .with_context(|| format!("failed to create {}", resources_dir.display()))?;

        let executable_path = macos_dir.join(&ctx.pascal_name);
        fs::copy(&ctx.binary_path, &executable_path).with_context(|| {
            format!(
                "failed to copy release binary from {} to {}",
                ctx.binary_path.display(),
                executable_path.display()
            )
        })?;
        // Belt-and-suspenders on top of the `profile.release.strip` build
        // default: guarantees a stripped binary even if the consumer's own
        // Cargo.toml explicitly disables strip.
        strip_binary(&executable_path);

        let assets_included = if let Some(assets_dir) = &ctx.assets_dir {
            collect_assets(assets_dir, &resources_dir.join("assets"))?;
            true
        } else {
            false
        };

        fs::write(contents_dir.join("Info.plist"), info_plist(ctx))
            .context("failed to write Info.plist")?;

        let binary_size_bytes = fs::metadata(&executable_path)
            .with_context(|| format!("failed to stat {}", executable_path.display()))?
            .len();

        Ok(PublishArtifact {
            root: bundle_root,
            assets_included,
            binary_size_bytes,
        })
    }
}

/// Strips debug/symbol info from the packaged executable in place. Best-effort:
/// a missing/failing `strip` (e.g. no Xcode command line tools) just leaves the
/// binary as-is rather than failing the whole publish.
fn strip_binary(path: &Path) {
    match Command::new("strip").arg(path).status() {
        Ok(status) if status.success() => {}
        Ok(status) => {
            eprintln!("warning: `strip` exited with {status}; shipping unstripped binary");
        }
        Err(err) => {
            eprintln!("warning: failed to run `strip` ({err}); shipping unstripped binary");
        }
    }
}

fn info_plist(ctx: &PublishContext) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>{display_name}</string>
    <key>CFBundleDisplayName</key>
    <string>{display_name}</string>
    <key>CFBundleIdentifier</key>
    <string>{bundle_id}</string>
    <key>CFBundleVersion</key>
    <string>{version}</string>
    <key>CFBundleShortVersionString</key>
    <string>{version}</string>
    <key>CFBundleExecutable</key>
    <string>{executable}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
"#,
        display_name = escape_xml(&ctx.display_name),
        bundle_id = escape_xml(&ctx.bundle_id),
        version = escape_xml(&ctx.version),
        executable = escape_xml(&ctx.pascal_name),
    )
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::publish::metadata::ProjectMetadata;
    use crate::publish::platform::Target;
    use std::path::PathBuf;

    fn sample_context() -> PublishContext {
        let metadata = ProjectMetadata {
            package_name: "my-app".into(),
            version: "0.1.0".into(),
            manifest_dir: PathBuf::from("/tmp/my-app"),
            target_dir: PathBuf::from("/tmp/my-app/target"),
            bin_name: "my-app".into(),
            assets_dir: None,
            display_name: "My App".into(),
            pascal_name: "MyApp".into(),
            bundle_id: "com.beverlyui.my-app".into(),
        };
        PublishContext::new(
            &metadata,
            Target::MacosArm64,
            PathBuf::from("/tmp/my-app/target/release/my-app"),
            PathBuf::from("/tmp/my-app/dist"),
        )
    }

    #[test]
    fn info_plist_embeds_metadata() {
        let plist = info_plist(&sample_context());
        assert!(plist.contains("<string>My App</string>"));
        assert!(plist.contains("<string>com.beverlyui.my-app</string>"));
        assert!(plist.contains("<string>0.1.0</string>"));
        assert!(plist.contains("<string>MyApp</string>"));
    }

    #[test]
    fn escape_xml_escapes_reserved_characters() {
        assert_eq!(escape_xml("A & B <C> \"D\""), "A &amp; B &lt;C&gt; &quot;D&quot;");
    }
}
