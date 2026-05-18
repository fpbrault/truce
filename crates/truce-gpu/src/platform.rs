//! Platform window bridging for baseview.
//!
//! Re-exports the canonical helpers from `truce_gui::platform` so this
//! crate's call sites can keep using `crate::platform::Foo` while the
//! actual rwh-0.5 to rwh-0.6 bridge, the `ParentWindow` newtype, and
//! the wgpu surface constructor live in one place. truce-gui's copy
//! is the canonical one; every GUI backend reads through these
//! re-exports so platform behaviour stays uniform.

pub use truce_gui::platform::{ParentWindow, query_backing_scale};
// `create_wgpu_surface` constructs from a baseview window handle;
// not available on iOS (the iOS path builds its surface directly
// from the CAMetalLayer-backed UIView in `truce-gui::editor_ios`).
#[cfg(not(target_os = "ios"))]
pub use truce_gui::platform::create_wgpu_surface;
