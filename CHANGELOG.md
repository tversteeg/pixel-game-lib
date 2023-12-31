# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.3](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.4.2...pixel-game-lib-v0.4.3) - 2023-12-23

### Fixed
- *(deps)* update rust crate winit to 0.29.5
- *(deps)* update rust crate tokio to 1.35.1
- *(deps)* update rust crate derive-where to 1.2.7

## [0.4.2](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.4.1...pixel-game-lib-v0.4.2) - 2023-12-13

### Added
- *(canvas)* add unoptimized 'draw_line' method

### Fixed
- *(deps)* update rust crate puffin to 0.18.1
- *(deps)* update rust crate tokio to 1.35.0
- *(deps)* update rust crate derive-where to 1.2.6
- *(deps)* update rust-wasm-bindgen monorepo

## [0.4.1](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.4.0...pixel-game-lib-v0.4.1) - 2023-11-26

### Fixed
- *(deps)* update rust crate winit to 0.29.4
- *(deps)* update rust crate puffin to 0.18.0
- *(deps)* update rust crate serde to 1.0.193

### Other
- set MSRV

## [0.4.0](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.3.1...pixel-game-lib-v0.4.0) - 2023-11-14

### Fixed
- *(deps)* update rust crate winit_input_helper to 0.15.1
- *(deps)* update rust crate hecs to 0.10.4
- *(deps)* update rust crate tokio to 1.34.0
- *(deps)* update rust crate serde to 1.0.192
- *(assets)* always embed on web
- *(deps)* update rust-wasm-bindgen monorepo
- *(deps)* update rust crate serde to 1.0.190

### Other
- *(deps)* [**breaking**] update winit to 0.29

## [0.3.1](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.3.0...pixel-game-lib-v0.3.1) - 2023-10-25

### Added
- *(physics)* add XPBD-based physics engine
- *(gui)* add label widget

### Fixed
- *(math)* conditionally implement From<Isometry2> for Iso
- *(canvas)* set_pixel coordinate calculation
- *(gui)* enforce type soundness with a reference type for each widget

### Other
- *(ci)* test every feature instead of all combinations of features
- *(example)* spawn objects on mouse click in physics example
- *(deps)* update swatinem/rust-cache action to v2.7.1
- *(window)* re-export winit_input_helper::WinitInputHelper as window::Input

## [0.3.0](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.2.0...pixel-game-lib-v0.3.0) - 2023-10-20

### Added
- *(gui)* implement layout system
- *(sprite)* metadata in TOML
- *(assets)* add type that owns an asset or uses a ref
- base structure for gui feature
- implement font & sprite asset loading
- assets features
- default-font feature loading image from memory

### Fixed
- *(window)* WASM build
- *(deps)* update rust crate serde to 1.0.189
- *(assets)* feature flags

### Other
- fix window example
- *(lib.rs)* document features
- *(ci)* fix
- merge main
- *(ci)* generate README.md from lib.rs
- [**breaking**] remove all feature flags
- [**breaking**] remove `assets` feature flag
- [**breaking**] overhaul all feature flags
- [**breaking**] re-export less in the crate root

## [0.2.0](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.1.0...pixel-game-lib-v0.2.0) - 2023-10-02

### Added
- *(window)* expose winit input
- [**breaking**] implement WASM window
- *(window)* hide async implementation

### Other
- ignore run-wasm in release-plz
