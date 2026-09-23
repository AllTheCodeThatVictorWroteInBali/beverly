# cargo-beverly

Developer CLI for Beverly projects, invoked as `cargo beverly <command>`.

## Install locally

From the repository root:

```bash
cargo install --path tools/cargo-beverly
```

This puts `cargo-beverly` on your `$PATH`, after which Cargo exposes it as `cargo beverly`.

During development you can also run it without installing:

```bash
cargo run --manifest-path tools/cargo-beverly/Cargo.toml -- publish
```

## `cargo beverly publish`

Builds a Beverly (Bevy) application in release mode and packages it into a runnable,
self-contained application bundle in `dist/`.

```bash
cargo beverly publish
```

What it does:

1. Reads package name, version, and manifest/target directories from `cargo metadata`
   (no `Cargo.toml` parsing, no duplicated config).
2. Runs `cargo build --release` for the project's binary target.
3. Copies the release binary and the project's `assets/` directory (if present) into
   the platform package.
4. Writes the result to `dist/`, next to `Cargo.toml` (recreating just that bundle, not
   the whole `dist/` directory).

### Platform support

| Platform | Status |
| --- | --- |
| macOS (arm64/x64) | Supported |
| Windows | Coming soon |
| Linux | Coming soon |

Running `cargo beverly publish` on macOS produces:

```
dist/
└── MyApp.app/
    └── Contents/
        ├── MacOS/MyApp
        ├── Resources/assets/...
        └── Info.plist
```

`Info.plist` fields (bundle name, identifier, version, executable) are derived from
Cargo package metadata, and can be overridden with an optional
`[package.metadata.beverly]` section in the target project's `Cargo.toml`:

```toml
[package.metadata.beverly]
display-name = "My App"
bundle-id = "com.example.my-app"
assets-dir = "assets"
```

Windows and Linux support is coming soon (not yet implemented) - `--target` does not accept a
`windows-*`/`linux-*` value yet. The publish pipeline (`src/publish/`) is deliberately split
into platform-independent stages (metadata, build, asset collection) and a `PlatformPackager`
trait (`src/publish/platform/`) implemented per platform, so adding Windows/Linux support means
adding a new `Target` variant and a new `platform/{windows,linux}.rs` packager, not rewriting
the pipeline.

### Flags

```bash
cargo beverly publish --manifest-path path/to/Cargo.toml   # defaults to the current directory
cargo beverly publish --target macos-arm64                 # defaults to the host platform
cargo beverly publish --out-dir path/to/output              # defaults to `dist/`
```
