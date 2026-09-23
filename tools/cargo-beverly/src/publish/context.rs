//! [`PublishContext`] carries everything platform packagers need, without any
//! of them knowing how the values were produced (Cargo metadata, a release
//! build, or CLI flags).

use std::path::PathBuf;

use super::metadata::ProjectMetadata;
use super::platform::Target;

/// Platform-independent inputs to a single packaging run.
pub struct PublishContext {
    #[allow(dead_code)] // available to packagers that need it (e.g. bundle metadata)
    pub package_name: String,
    pub version: String,
    pub display_name: String,
    pub pascal_name: String,
    pub bundle_id: String,
    #[allow(dead_code)] // always "release" today; kept for future build-profile support
    pub profile: &'static str,
    #[allow(dead_code)] // only one packager exists today; will branch on this once more targets land
    pub target: Target,
    pub binary_path: PathBuf,
    pub assets_dir: Option<PathBuf>,
    pub output_dir: PathBuf,
}

impl PublishContext {
    pub fn new(
        metadata: &ProjectMetadata,
        target: Target,
        binary_path: PathBuf,
        output_dir: PathBuf,
    ) -> Self {
        Self {
            package_name: metadata.package_name.clone(),
            version: metadata.version.clone(),
            display_name: metadata.display_name.clone(),
            pascal_name: metadata.pascal_name.clone(),
            bundle_id: metadata.bundle_id.clone(),
            profile: "release",
            target,
            binary_path,
            assets_dir: metadata.assets_dir.clone(),
            output_dir,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_metadata() -> ProjectMetadata {
        ProjectMetadata {
            package_name: "my-app".into(),
            version: "0.1.0".into(),
            manifest_dir: PathBuf::from("/tmp/my-app"),
            target_dir: PathBuf::from("/tmp/my-app/target"),
            bin_name: "my-app".into(),
            assets_dir: None,
            display_name: "My App".into(),
            pascal_name: "MyApp".into(),
            bundle_id: "com.beverlyui.my-app".into(),
        }
    }

    #[test]
    fn context_carries_metadata_through_unchanged() {
        let metadata = sample_metadata();
        let ctx = PublishContext::new(
            &metadata,
            Target::MacosArm64,
            PathBuf::from("/tmp/my-app/target/release/my-app"),
            PathBuf::from("/tmp/my-app/dist"),
        );

        assert_eq!(ctx.package_name, "my-app");
        assert_eq!(ctx.pascal_name, "MyApp");
        assert_eq!(ctx.profile, "release");
        assert_eq!(ctx.output_dir, PathBuf::from("/tmp/my-app/dist"));
    }
}
