//! `cargo truce log-stream-au` — tail AU v3 appex logs live.
//!
//! macOS-only. Wraps `/usr/bin/log stream` with a predicate matching
//! the AU v3 Swift wrapper's subsystem (`com.truce.au3`). Forward-only
//! — does not surface historical entries; for those use `log show
//! --last <duration>` directly.

use crate::Res;

#[cfg(not(target_os = "macos"))]
pub(crate) fn cmd_log_stream_au() -> Res {
    Err(
        "`cargo truce log-stream-au` is macOS-only — it wraps Apple's \
         `/usr/bin/log stream`, which doesn't exist on Linux or Windows. \
         AU v3 itself is also macOS-only."
            .into(),
    )
}

#[cfg(target_os = "macos")]
pub(crate) fn cmd_log_stream_au() -> Res {
    use std::process::Command;

    eprintln!("Streaming AU v3 appex logs (Ctrl-C to stop)...\n");
    let status = Command::new("/usr/bin/log")
        .args([
            "stream",
            "--predicate",
            "subsystem == \"com.truce.au3\"",
            "--style",
            "compact",
            "--level",
            "debug",
        ])
        .status()?;
    if !status.success() {
        return Err("log stream exited with error".into());
    }
    Ok(())
}
