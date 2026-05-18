//! GPU rendering backend for truce plugins.
//!
//! Uses wgpu (Metal/DX12 backends per the workspace's feature pin) with
//! lyon tessellation and a fontdue glyph atlas. Implements
//! `truce_gui::RenderBackend` so widgets render identically to the CPU
//! path.
//!
//! Platform windowing is provided by baseview.

/// Crate-wide debug-print macro for GPU init / render hot-reload paths.
/// Compiles to nothing unless the `hot-debug` feature is enabled.
/// Defined at crate root so any module under `truce_gpu::*` can reach
/// it without re-importing.
#[macro_export]
macro_rules! hot_debug {
    ($($arg:tt)*) => {
        #[cfg(feature = "hot-debug")]
        eprintln!($($arg)*);
    };
}

mod backend;
// `editor` is the baseview-driven host (macOS/Windows/Linux). iOS
// hosts the editor via `truce-gui::editor_ios` inside a UIView
// attached to the AUv3 view controller, so the baseview-backed
// editor doesn't compile on iOS.
#[cfg(not(target_os = "ios"))]
pub mod editor;
pub mod platform;

pub use backend::WgpuBackend;
#[cfg(not(target_os = "ios"))]
pub use editor::GpuEditor;
