use crate::buffer::AudioBuffer;
use crate::bus::BusLayout;
use crate::editor::Editor;
use crate::events::EventList;
use crate::info::PluginInfo;
use crate::process::{ProcessContext, ProcessStatus};

/// The format-facing plugin trait. **Plugin authors do NOT implement
/// this directly.**
///
/// `Plugin` is the surface every format wrapper (CLAP, VST3, VST2,
/// LV2, AU, AAX) consumes. The `truce::plugin!` macro generates an
/// `impl Plugin for __HotShellWrapper` from the user's two
/// implementations — [`PluginLogic`](crate::PluginLogic) (DSP) and
/// `truce_gui::PluginEditor` (GUI) — so the user writes safe Rust
/// against a split surface and the format wrappers see one combined
/// trait against the macro-generated wrapper type.
///
/// What plugin authors implement instead:
///
/// ```ignore
/// impl truce::prelude::PluginLogic for MyPlugin {
///     fn reset(&mut self, sr: f64, bs: usize) { /* ... */ }
///     fn process(&mut self, /* ... */) -> ProcessStatus { /* ... */ }
/// }
///
/// impl truce::prelude::PluginEditor for MyPlugin {
///     fn layout(&self) -> GridLayout { /* ... */ }
/// }
///
/// truce::plugin! { logic: MyPlugin, params: MyPluginParams }
/// ```
///
/// The macro-emitted `impl Plugin` routes each method to the right
/// half: DSP methods (`reset`, `process`, `save_state`, `load_state`,
/// `latency`, `tail`, `bus_layouts`, `supports_in_place`) call into
/// `PluginLogic`; `editor()` calls into `PluginEditor::custom_editor`
/// or builds a `BuiltinEditor` from `PluginEditor::layout`.
///
/// This trait stays in `truce-core` because the format wrappers
/// depend on `truce-core` and need to consume one combined trait;
/// keeping the user-facing surface split (across `truce-core` and
/// `truce-gui`) keeps headless plugins from pulling GUI types into
/// their compile errors.
pub trait Plugin: Send + 'static {
    /// Opt into zero-copy in-place I/O. When this returns `true`,
    /// the format wrapper skips its safety memcpy on host-aliased
    /// buffers and hands the plugin the raw shared memory through
    /// `AudioBuffer::in_out_mut(ch)`. The plugin must check
    /// `AudioBuffer::is_in_place(ch)` per channel before reading
    /// `input(ch)` — for in-place channels `input(ch)` returns an
    /// empty slice, and the data lives only in the shared buffer.
    ///
    /// Default `false`: the wrapper copies aliased inputs into scratch
    /// so `input(ch)` and `output(ch)` are always disjoint. Costs one
    /// memcpy per aliased channel per block (a few hundred KB/sec at
    /// audio rates) and lets plugin code stay format-agnostic.
    ///
    /// `where Self: Sized` so a `dyn Plugin` trait object stays
    /// dyn-compatible — the format wrappers consume `P: Plugin`
    /// generically and call the method statically.
    #[must_use]
    fn supports_in_place() -> bool
    where
        Self: Sized,
    {
        false
    }

    /// Static metadata about the plugin.
    ///
    /// Use `plugin_info!()` for zero-boilerplate (reads from truce.toml
    /// + Cargo.toml at compile time — no `build.rs` required).
    fn info() -> PluginInfo
    where
        Self: Sized;

    /// Supported bus layouts. The host picks one.
    #[must_use]
    fn bus_layouts() -> Vec<BusLayout>
    where
        Self: Sized,
    {
        vec![BusLayout::stereo()]
    }

    /// Called once after construction. Not real-time safe.
    fn init(&mut self) {}

    /// Called when sample rate or max block size changes.
    /// Reset filters, delay lines, etc. Not real-time safe.
    fn reset(&mut self, sample_rate: f64, max_block_size: usize);

    /// Real-time audio processing.
    fn process(
        &mut self,
        buffer: &mut AudioBuffer,
        events: &EventList,
        context: &mut ProcessContext,
    ) -> ProcessStatus;

    /// Save extra state beyond parameter values. Empty `Vec` means
    /// "no extra state" — matches `PluginLogic::save_state`'s shape so
    /// the wrapper bridge is a passthrough rather than an
    /// `Option<Vec<u8>>` ↔ `Vec<u8>` translation.
    fn save_state(&self) -> Vec<u8> {
        Vec::new()
    }

    /// Restore extra state. Mirrors `PluginLogic::load_state`'s
    /// `Result` shape so the wrapper bridge is a passthrough.
    ///
    /// # Errors
    ///
    /// Returns `Err` when the macro-generated impl forwards a
    /// `PluginLogic::load_state` failure (malformed bytes, version
    /// skew between session file and plugin build, etc).
    fn load_state(&mut self, _data: &[u8]) -> Result<(), crate::state::StateLoadError> {
        Ok(())
    }

    /// GUI editor. Return None for headless plugins.
    fn editor(&mut self) -> Option<Box<dyn Editor>> {
        None
    }

    /// Processing latency in samples. Host uses this for delay compensation.
    /// Return 0 if the plugin adds no latency (default).
    fn latency(&self) -> u32 {
        0
    }

    /// Tail time in samples. Return `u32::MAX` for infinite tail.
    /// Return 0 for no tail (default).
    fn tail(&self) -> u32 {
        0
    }

    /// Read a meter value by ID (0.0–1.0). Called by the GUI at ~60fps.
    /// Override to expose level meters, gain reduction, etc.
    fn get_meter(&self, _meter_id: u32) -> f32 {
        0.0
    }
}
