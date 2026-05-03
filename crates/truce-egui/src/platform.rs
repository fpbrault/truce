//! Platform window bridging for baseview.
//!
//! Re-exports the canonical helpers from `truce_gui::platform` so this
//! crate's call sites can keep using `crate::platform::Foo` while the
//! actual rwh-0.5 → rwh-0.6 bridge, the `ParentWindow` newtype, the
//! wgpu surface constructor, and the per-OS scale-factor query live
//! in one place.
//!
//! Note: `query_backing_scale` here used to fall back to `1.0` on
//! every non-macOS platform; the `truce_gui` canonical version walks
//! Win32 `GetDpiForWindow` / `GetDpiForSystem` and the Linux
//! cached-from-baseview value, so re-exporting fixes Windows + Linux
//! egui editors that previously rendered at 1.0× regardless of the
//! host's DPI.

pub use truce_gui::platform::{ParentWindow, create_wgpu_surface, query_backing_scale};
