//! `--state <path>` load helper, shared by `windowed`, `headless`,
//! and `offline` runners. Wraps `truce_core::state::restore_plugin`
//! with a uniform diagnostic surface so users see the same failure
//! messages no matter which mode they launched in.

use std::path::Path;

use truce_core::export::PluginExport;

/// Read `path` and apply it to `plugin` via the canonical state
/// envelope. Logs a single line on success, a single line on each
/// failure mode (read error vs envelope mismatch). Never panics —
/// state load is a convenience, not a hard prereq for processing.
pub fn load_into<P: PluginExport>(plugin: &mut P, path: &Path) {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!(
                "[truce-standalone] failed to read state {}: {e}",
                path.display()
            );
            return;
        }
    };
    match truce_core::state::restore_plugin(plugin, &bytes) {
        Ok(()) => eprintln!("[truce-standalone] loaded state from {}", path.display()),
        Err(truce_core::state::RestoreError::Invalid) => eprintln!(
            "[truce-standalone] {} doesn't look like a state file for {} \
             (wrong magic / version / plugin ID)",
            path.display(),
            P::info().name,
        ),
    }
}
