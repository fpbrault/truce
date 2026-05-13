# Fundsp Reverb

Stereo plate reverb wired through a [`fundsp`](https://github.com/SamiPerttu/fundsp) audio graph. The point of this example is the integration shape — how to hold a fundsp graph inside a truce plugin and keep it alloc-free on the audio thread.

```text
in (L,R) ──► high-pass (low cut)  ──► low-pass (high cut)  ──► reverb_stereo ──┐
                                                                                │
in (L,R) ─────────────────────────────────────────────────────────────────► dry ┤──► out
                                                                                │
                                                              mix ──────────────┘
```

| Param    | Range                  |
|----------|------------------------|
| Low Cut  | 20 Hz → 2 kHz (log)    |
| High Cut | 500 Hz → 18 kHz (log)  |
| Time     | 0.2 s → 10 s (log)     |
| Mix      | 0 → 1                  |

## Integration patterns

- **Graph built in `reset()`.** fundsp's `allocate()` reserves all delay lines and FIR weights up front; `process()` never allocates.
- **Params reach the graph through `fundsp::Shared` atomics.** `var(&shared)` reads them per sample; the closure inside `for_each_frame` writes the smoothed truce-side value into the cell on the same tick (sample-accurate automation).
- **`Box<dyn AudioUnit>`** for the field type. The concrete `An<…>` is hundreds of chars of nested generics; the vtable cost is one indirection per block.
- **`AudioBuffer::for_each_frame::<2, _>`** transposes truce's per-channel layout into stack-allocated frames so fundsp's `tick(in, out)` callback can be called directly. No scratch field.
- **Reverb time rebuilds the graph** when the param drifts ≥ 5% — `reverb_stereo`'s `time` argument is baked at construction. Rebuilds allocate, so the hysteresis keeps it rare.

## Gotchas

- **Filter input order is positional and unchecked.** `highpass()` / `lowpass()` take `(signal, cutoff, Q)`. Every connection is `f32`, so `(cutoff | Q | signal) >> highpass()` compiles fine and silently feeds the filter cutoff in as audio — the resulting filter blows up the reverb FDN to peak ~7000 within a second. Test against constant input + `assert_peak_below`.
- **Type-level channels.** `dry * mix` fails to compile when `dry` is stereo and `mix` is a 1-channel `Shared` read; broadcast the mix to stereo manually with `var(&mix) | var(&mix)`. fundsp's payoff (graph composition with `>>`/`|`/`&`) costs this kind of explicit plumbing.

## Build

```sh
cargo build -p truce-example-fundsp-reverb
cargo test  -p truce-example-fundsp-reverb --release
cargo truce install -p truce-example-fundsp-reverb
cargo truce run     -p truce-example-fundsp-reverb
```
