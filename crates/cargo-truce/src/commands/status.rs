//! `cargo truce status` — show installed plugins and AU registration state.
//!
//! macOS-only: every path it scans (`/Library/Audio/Plug-Ins/...`,
//! `~/Library/Audio/Plug-Ins/...`, `auval -a`) is Apple-specific.
//! Linux / Windows are handled with a clean "not supported" message
//! instead of an empty banner that suggests nothing was found.

use crate::Res;

#[cfg(not(target_os = "macos"))]
pub(crate) fn cmd_status() -> Res {
    Err(
        "`cargo truce status` is macOS-only — every directory it scans \
         (`/Library/Audio/Plug-Ins/...`, `auval -a`) is Apple-specific. \
         For Linux / Windows, list bundles directly under your DAW's \
         configured plug-in path."
            .into(),
    )
}

#[cfg(target_os = "macos")]
pub(crate) fn cmd_status() -> Res {
    use crate::{dirs, load_config, run_quiet};
    use std::fs;
    use std::path::Path;

    let config = load_config()?;
    let vendor = &config.vendor.name;
    let home = dirs::require_home_dir()?;

    eprintln!("=== AU v2 Components ===");
    let comp_dir = Path::new("/Library/Audio/Plug-Ins/Components");
    if comp_dir.exists() {
        for entry in fs::read_dir(comp_dir)? {
            let name = entry?.file_name();
            let name = name.to_string_lossy();
            if name.contains(vendor) {
                eprintln!("  {name}");
            }
        }
    }

    eprintln!("\n=== CLAP ===");
    let clap_dir = home.join("Library/Audio/Plug-Ins/CLAP");
    if clap_dir.exists() {
        for entry in fs::read_dir(&clap_dir)? {
            let name = entry?.file_name();
            let name = name.to_string_lossy();
            if name.contains(vendor) {
                eprintln!("  {name}");
            }
        }
    }

    eprintln!("\n=== VST2 ===");
    let vst2_dir = home.join("Library/Audio/Plug-Ins/VST");
    if vst2_dir.exists() {
        for entry in fs::read_dir(&vst2_dir)? {
            let name = entry?.file_name();
            let name = name.to_string_lossy();
            if name.contains(vendor) {
                eprintln!("  {name}");
            }
        }
    }

    eprintln!("\n=== VST3 ===");
    let vst3_dir = Path::new("/Library/Audio/Plug-Ins/VST3");
    if vst3_dir.exists() {
        for entry in fs::read_dir(vst3_dir)? {
            let name = entry?.file_name();
            let name = name.to_string_lossy();
            if name.contains(vendor) {
                eprintln!("  {name}");
            }
        }
    }

    eprintln!("\n=== auval ===");
    if let Ok(output) = run_quiet("auval", &["-a"]) {
        let vendor_lower = vendor.to_lowercase();
        for line in output.lines() {
            if line.to_lowercase().contains(&vendor_lower) {
                eprintln!("  {line}");
            }
        }
    }

    Ok(())
}
