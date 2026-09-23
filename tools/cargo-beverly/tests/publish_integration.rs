//! End-to-end test: builds a throwaway binary crate through the real
//! `cargo-beverly` executable and asserts the resulting `.app` bundle layout.
//! Uses a temp directory so it never depends on Beverly's own project.

use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn publish_produces_a_macos_app_bundle() {
    let project_dir = tempfile::tempdir().expect("failed to create temp project dir");
    write_demo_project(project_dir.path());

    let out_dir = project_dir.path().join("dist");
    let exe = env!("CARGO_BIN_EXE_cargo-beverly");

    let output = Command::new(exe)
        .arg("publish")
        .arg("--manifest-path")
        .arg(project_dir.path().join("Cargo.toml"))
        .arg("--out-dir")
        .arg(&out_dir)
        .output()
        .expect("failed to run cargo-beverly");

    assert!(
        output.status.success(),
        "cargo-beverly publish failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let bundle = out_dir.join("DemoApp.app");
    assert!(bundle.is_dir(), "missing bundle at {}", bundle.display());
    assert!(bundle.join("Contents/MacOS/DemoApp").is_file());
    assert!(bundle.join("Contents/Resources/assets/hello.txt").is_file());
    assert!(bundle.join("Contents/Info.plist").is_file());

    let plist = fs::read_to_string(bundle.join("Contents/Info.plist")).unwrap();
    assert!(plist.contains("com.beverlyui.demo-app"));
    assert!(plist.contains("0.3.1"));
}

fn write_demo_project(root: &Path) {
    fs::write(
        root.join("Cargo.toml"),
        r#"[package]
name = "demo-app"
version = "0.3.1"
edition = "2021"
"#,
    )
    .unwrap();

    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("main.rs"), "fn main() { println!(\"hello\"); }\n").unwrap();

    let assets = root.join("assets");
    fs::create_dir_all(&assets).unwrap();
    fs::write(assets.join("hello.txt"), "hello asset\n").unwrap();
}
