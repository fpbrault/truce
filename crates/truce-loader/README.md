# truce-loader

Plugin runtime + hot-reload infrastructure for truce.

## Overview

Hot-reload mechanics for truce: dylib loading, ABI canary, vtable
probe, and the shells (`HotShell`, `StaticShell`) that bridge the
user-facing traits onto `truce_core::Plugin` for format wrappers.
Used by every truce plugin, in two modes:

- **Static (default).** `export_static!` embeds the user's struct
  directly into the format wrapper at compile time. Zero runtime
  overhead, single dylib.
- **Hot-reload (opt-in).** `export_plugin!` exports the plugin
  across a `#[no_mangle]` C ABI in a separate dylib; the shell
  loads it via `libloading`, verifies ABI compatibility, and swaps
  the trait object on rebuild without restarting the DAW.
  Preserves audio continuity.

Plugin authors don't reach into this crate directly. They write
a single `impl PluginLogic` (in `truce-gui`) on their plugin
struct -- one trait covering both DSP (`reset`, `process`, …) and
GUI (`layout`, `custom_editor`, …) -- and `truce::plugin!` emits
the right `export_*!` call based on the `shell` Cargo feature.

## Key types and macros

- **`PluginLogic`** -- the user-facing trait crossed across the
  dylib boundary as `Box<dyn PluginLogic>` in shell mode.
- **`HotShell`** -- shell-side dylib loader and hot-swap manager.
- **`StaticShell`** -- shell-side wrapper that embeds the plugin
  at compile time.
- **`export_static!`** -- emits the `__HotShellWrapper` for static
  mode.
- **`export_plugin!`** -- emits the `#[no_mangle]` C ABI symbols
  for shell mode (`truce_create`, `truce_abi_canary`,
  `truce_vtable_probe`).

## Features

| Feature | Description |
|---------|-------------|
| `shell` | Enable dylib loading via `libloading` (turns on `HotShell`) |
| `hot-debug` | Verbose hot-reload diagnostics (load timings, ABI checks) |
| `gpu` | GPU rendering support in the shell |

## Usage

Enable the dynamic shell (hot-reload) during development:

```toml
[dependencies]
truce = { git = "https://github.com/truce-audio/truce", tag = "vX.Y.Z", features = ["shell"] }
```

(Replace `vX.Y.Z` with the latest release tag.)

Part of [truce](https://github.com/truce-audio/truce).
