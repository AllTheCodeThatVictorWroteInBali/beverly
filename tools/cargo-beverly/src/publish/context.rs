//! [`PublishContext`] carries everything platform packagers need, without any
//! of them knowing how the values were produced (Cargo metadata, a release
//! build, or CLI flags).

use std::path::PathBuf;

use super::metadata::ProjectMetadata;

/// Platform-independent inputs to a single packaging run.
pub struct PublishContext {
    pub version: String,
    pub display_name: String,
    pub pascal_name: String,
    pub bundle_id: String,
    pub binary_path: PathBuf,
    pub assets_dir: Option<PathBuf>,
    pub output_dir: PathBuf,
}

impl PublishContext {
    pub fn new(metadata: &ProjectMetadata, binary_path: PathBuf, output_dir: PathBuf) -> Self {
        Self {
            version: metadata.version.clone(),
            display_name: metadata.display_name.clone(),
            pascal_name: metadata.pascal_name.clone(),
            bundle_id: metadata.bundle_id.clone(),
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
            PathBuf::from("/tmp/my-app/target/release/my-app"),
            PathBuf::from("/tmp/my-app/dist"),
        );

        assert_eq!(ctx.pascal_name, "MyApp");
        assert_eq!(ctx.output_dir, PathBuf::from("/tmp/my-app/dist"));
    }
}
