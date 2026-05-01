//! Standalone host for truce plugins.
//!
//! Runs a plugin cdylib with direct cpal audio I/O and an optional
//! GUI window (via baseview + the plugin's own `Editor`). Zero
//! plugin-library code is required — the runner obtains the editor
//! via `PluginExport::editor()`, the same API every format wrapper
//! uses.
//!
//! # Entry point
//!
//! Plugins supply a `[[bin]] <suffix>-standalone` target with a
//! `src/main.rs` that calls:
//!
//! ```ignore
//! fn main() {
//!     truce_standalone::run::<my_plugin::Plugin>();
//! }
//! ```
//!
//! # Modes
//!
//! - **Windowed** (default, requires `gui` feature): opens a
//!   baseview window hosting the plugin's editor, drives a cpal
//!   stream on the audio thread.
//! - **Headless** (`--headless` flag or the `gui` feature disabled):
//!   audio only. For effects this means audio passes through; for
//!   instruments the plugin emits silence unless a MIDI device is
//!   connected (`--midi-input`; see [`midi`]).
//!
//! See [`cli`] for the full flag surface.

pub mod audio;
pub mod cli;
pub mod in_process;
pub mod keyboard;
pub mod midi;
pub mod transport;

#[cfg(feature = "gui")]
pub mod windowed;

#[cfg(all(target_os = "macos", feature = "gui"))]
pub mod menu_macos;

#[cfg(all(target_os = "windows", feature = "gui"))]
pub mod menu_windows;

pub mod headless;

pub use truce_core::export::PluginExport;

/// Re-export for backward compatibility.
pub use truce_core::export::PluginExport as StandaloneExport;

/// Compile-time-baked launch defaults from `[plugin.standalone]` in
/// the consumer's `truce.toml`. Constructed by the
/// [`baked_defaults!`] macro and threaded through CLI resolution as
/// the lowest tier (above the runtime default, below env / CLI).
///
/// Constructing this directly with `Defaults::default()` is fine —
/// it disables the bake tier entirely.
#[derive(Default, Clone, Copy, Debug)]
pub struct Defaults {
    pub input_enabled: Option<bool>,
    pub output_enabled: Option<bool>,
}

/// Read launch defaults baked by `truce-build` from
/// `[plugin.standalone]` in the consumer's `truce.toml`. The
/// `option_env!` reads must happen in the consumer's compile context
/// (build-script env vars don't reach dependencies), so this is a
/// macro the consumer expands inside its own `main.rs`.
///
/// ```ignore
/// fn main() {
///     truce_standalone::run::<Plugin>(truce_standalone::baked_defaults!());
/// }
/// ```
#[macro_export]
macro_rules! baked_defaults {
    () => {
        $crate::Defaults {
            input_enabled: option_env!("TRUCE_STANDALONE_BAKED_INPUT_ENABLED")
                .and_then(|s| s.parse().ok()),
            output_enabled: option_env!("TRUCE_STANDALONE_BAKED_OUTPUT_ENABLED")
                .and_then(|s| s.parse().ok()),
        }
    };
}

/// Run the plugin standalone.
///
/// Parses CLI flags + env vars on top of the supplied baked defaults
/// (use [`baked_defaults!`] to read them from `truce.toml`, or pass
/// `Defaults::default()` to skip the bake tier). Dispatches to the
/// windowed or headless runner. Returns when the user closes the
/// window or sends SIGINT.
pub fn run<P: PluginExport>(defaults: Defaults)
where
    P::Params: 'static,
{
    let opts = match cli::parse(defaults) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Error: {e}");
            eprintln!("Run with --help for usage.");
            std::process::exit(2);
        }
    };

    if opts.list_devices {
        audio::list_devices();
        return;
    }
    if opts.list_midi {
        midi::list_midi();
        return;
    }

    #[cfg(feature = "gui")]
    {
        if opts.headless {
            headless::run::<P>(&opts);
        } else {
            windowed::run::<P>(&opts);
        }
        return;
    }
    #[cfg(not(feature = "gui"))]
    headless::run::<P>(&opts);
}
