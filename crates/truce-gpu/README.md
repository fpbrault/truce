# truce-gpu

GPU rendering backend for truce plugins.

## Overview

Wraps the built-in widget toolkit from `truce-gui` with hardware-accelerated
rendering via wgpu (Metal on macOS, DX12 on Windows, Vulkan on Linux). Uses
lyon for path tessellation and fontdue for glyph atlas generation. Platform
windowing is provided by baseview. Widgets render identically to the CPU path
but with significantly better performance on complex UIs.

User plugins take a direct dep on this crate
(`truce-gpu = { workspace = true }`) when they want a GPU-rendered
editor. Every in-tree example plugin uses this path.

## Key types

- **`GpuEditor`** -- GPU-accelerated `Editor` implementation
- **`WgpuBackend`** -- implements `truce_gui::RenderBackend` using wgpu

## Usage

```toml
[dependencies]
truce     = { version = "0.48", features = ["clap"] }
truce-gpu = { version = "0.48" }
```

(Cargo's caret resolver expands `"0.48"` to `>=0.48.0, <0.49.0`,
so you'll pick up every `0.48.x` patch release without re-editing.
To track an unreleased checkout, swap the lines for
`git = "https://github.com/truce-audio/truce", branch = "main"`.
Or just run `cargo truce new` and let the scaffolder pin for you.)

```rust
fn editor() -> Option<Box<dyn Editor>> {
    Some(Box::new(GpuEditor::new()))
}
```

Part of [truce](https://github.com/truce-audio/truce). [Docs](https://truce.audio/docs/).
