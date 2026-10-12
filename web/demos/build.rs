use std::{collections::BTreeSet, env, fs, path::PathBuf};

fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let examples = root.join("../../examples").canonicalize().unwrap();
    println!("cargo:rerun-if-changed=manifest.json");
    println!("cargo:rerun-if-changed={}", examples.display());
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join("manifest.json")).unwrap()).unwrap();
    let mut names = BTreeSet::new();
    let mut modules = String::new();
    let mut arms = String::new();
    for entry in manifest.as_array().unwrap() {
        let name = entry["demo"].as_str().unwrap();
        assert!(
            name.chars()
                .all(|character| character.is_ascii_lowercase() || character == '_')
        );
        assert!(names.insert(name.to_owned()), "Duplicate demo: {name}");
        let path = examples.join(format!("{name}.rs"));
        assert!(path.is_file(), "Missing example: {name}");
        modules.push_str(&format!(
            "#[path = {:?}] mod {name};\n",
            path.to_str().unwrap()
        ));
        arms.push_str(&format!("{name:?} => {name}::main(),\n"));
    }
    let actual: BTreeSet<_> = fs::read_dir(&examples)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
        .map(|path| path.file_stem().unwrap().to_str().unwrap().to_owned())
        .filter(|name| name != "button")
        .collect();
    assert_eq!(
        names, actual,
        "Every example must have a documentation demo"
    );
    modules.push_str(&format!(
        "fn run_selected(name: &str) -> bool {{ match name {{ {arms} _ => return false }} true }}"
    ));
    fs::write(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("demos.rs"),
        modules,
    )
    .unwrap();
}
