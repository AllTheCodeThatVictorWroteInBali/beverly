# Changelog

All notable changes to Beverly will be documented in this file.

Beverly is currently in early development, so version numbers and release conventions may change as the project matures.

---

## Unreleased

### Added

- Standalone `beverly` Rust crate: `BeverlyPlugin`, ~35 UI components, rendering/shader system,
  theming, animation, and accessibility primitives, migrated from `theblocks_studio` and
  packaged for independent use (`beverly = "0.1"`).
- Compile-time embedded shaders (`bevy::asset::embedded_asset!`) and Feather icon SVGs, so no
  external `assets/` folder is required by consumers.
- `file_dialog`, `http_form`, and `test-support` Cargo features.
- `examples/{showcase,buttons,cards,components}.rs`.
- Public Beverly project repository
- Beverly marketing website
- Custom domain at `beverlyui.com`
- mdBook documentation site
- GitHub Pages deployment via GitHub Actions
- Initial project documentation
- Initial contribution guidelines
- Project roadmap

### In Progress

- Core Beverly UI architecture
- Reusable UI primitives
- Theme and design-token system
- Accessibility foundations
- Animation and interaction system
- GPU-native visual effects
- AI interaction primitives

---

## 0.1.0 — Initial Development

**Status:** Early development

The initial public development phase of Beverly.

### Goals

- Establish a Rust-native UI foundation for Bevy
- Develop reusable UI primitives
- Explore GPU-native rendering and visual effects
- Establish accessibility primitives
- Explore UI patterns for AI-native applications
- Build local-first and self-hosted foundations

### Notes

This release is experimental. APIs and architecture are expected to change as Beverly develops.

---

## Future Releases

Future releases will use semantic versioning once Beverly's public API begins to stabilize.

Release notes will document:

- New components
- API changes
- Accessibility improvements
- Performance improvements
- Rendering changes
- Bug fixes
- Breaking changes
- Documentation updates
