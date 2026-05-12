# State

Hello-world for `#[derive(State)]` — the per-instance non-numeric
data plumbing.

## What it demonstrates

- The split between **params** (numeric atoms — automatable, host
  knows about them) and **state** (anything else — strings, file
  paths, view modes, nested structs — the framework round-trips
  bytes on `save_state` / `load_state` but the host never sees
  the shape).
- `#[derive(State)]` on a `StateBinding`-bound struct
- A custom egui editor that reads + writes the instance label
- An audio-thread param (`Active: BoolParam`) sitting alongside
  the state-only label

## Parameters

| Name | Type | Description |
|------|------|-------------|
| Active | `BoolParam` | Pass-through enable toggle |

## State

| Field | Type | Description |
|-------|------|-------------|
| `label` | `String` | User-entered plugin instance label |

## Build and test

```bash
cargo build -p truce-example-state
cargo test -p truce-example-state
```
