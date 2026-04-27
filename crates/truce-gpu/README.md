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
truce = { git = "https://github.com/truce-audio/truce", features = ["clap"] }
truce-gpu = { git = "https://github.com/truce-audio/truce" }
```

```rust
fn editor() -> Option<Box<dyn Editor>> {
    Some(Box::new(GpuEditor::new()))
}
```

Part of [truce](https://github.com/truce-audio/truce).
