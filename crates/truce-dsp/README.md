# truce-dsp

Realtime-safe DSP utilities for truce plugins.

## Overview

A grab-bag of lock-free, allocation-free primitives that user plugins
opt into directly. Nothing here depends on the wider truce framework
— the crate stands alone, so plugins that don't need it pay no cost.

The primary inhabitant today is `audio_tap`, a lock-free SPSC ring
for handing audio-derived data from the DSP thread to the editor /
UI thread (oscilloscopes, spectrum analyzers, waveform history,
visualizers in general). The producer side is `realtime-safe` (no
locks, no allocation, no syscalls); the consumer side runs on the UI
thread and reads as fast as it can.

Future additions (smoothers, envelope followers, etc.) can live
alongside it.

## Key types

- **`AudioTapProducer`** -- realtime-safe writer (DSP thread)
- **`AudioTapConsumer`** -- reader (UI thread)
- **`audio_tap()`** -- factory that pairs a producer + consumer

## Usage

Add it to your plugin's `Cargo.toml`:

```toml
[dependencies]
truce-dsp = { workspace = true }   # or crates.io / git, depending on setup
```

```rust
use truce_dsp::{audio_tap, AudioTapConsumer, AudioTapProducer};

let (tap_tx, tap_rx) = audio_tap(/* capacity */ 4096);
// store tap_tx on the DSP side, tap_rx on the editor side
```

Part of [truce](https://github.com/truce-audio/truce).
