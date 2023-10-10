//! Utility library for games, not a game engines.
//!
//! # Features
//!
//! ### `default-font`
//!
//! Implements [`Default`] for [`font::Font`] with a font that's embedded into memory.
//!
//! ### `hot-reloading-assets` (default)
//!
//! Hot-reload assets from disk when they are saved.
//! Has no effect on the web target.
//!
//! ### `embedded-assets` (default on web)
//!
//! Bake _all_ assets in the `assets/` folder in the binary.
//! When creating a release binary this feature flag should be enabled.

pub mod font;
pub mod sprite;

pub mod window;
pub use window::window;

pub mod assets;
pub use assets::{asset, asset_owned};

pub mod gui;

/// Re-export taffy types.
pub use taffy;
/// Re-export vek types.
pub use vek;
