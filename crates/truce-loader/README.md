# truce-loader

Plugin runtime + hot-reload infrastructure for truce.

## Overview

Provides the runtime that wires a plugin's `PluginLogic` to its host-side
shell. Used by every truce plugin, in two modes:

- **Static (default).** `export_static!` embeds the logic directly into
  the format wrapper at compile time. Zero runtime overhead, single dylib.
- **Hot-reload (opt-in).** `export_plugin!` exports the logic across a
  `#[no_mangle]` C ABI in a separate dylib; the shell loads it via
  `libloading`, verifies ABI compatibility, and swaps the trait object
  on rebuild without restarting the DAW. Preserves audio continuity.

Developers implement the safe `PluginLogic` trait. The `truce::plugin!`
macro emits the right `export_*!` call based on the `hot-reload`
Cargo feature on the `truce` facade.

## Key types and macros

- **`PluginLogic`** -- safe trait every plugin implements
- **`HotShell`** -- shell-side dylib loader and hot-swap manager
- **`StaticShell`** -- shell-side wrapper that embeds the logic at compile time
- **`export_static!`** -- emits the `__HotShellWrapper` for static mode
- **`export_plugin!`** -- emits the `#[no_mangle]` C ABI for hot-reload mode

## Features

| Feature | Description |
|---------|-------------|
| `shell` | Enable dylib loading via `libloading` (turns on `HotShell`) |
| `hot-debug` | Verbose hot-reload diagnostics (load timings, ABI checks) |
| `gpu` | GPU rendering support in the shell |

## Usage

Enable hot-reload during development:

```toml
[dependencies]
truce = { git = "https://github.com/truce-audio/truce", features = ["hot-reload"] }
```

Part of [truce](https://github.com/truce-audio/truce).
