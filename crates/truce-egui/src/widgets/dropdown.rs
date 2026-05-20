//! Parameter dropdown (click-to-open list) bound to a truce parameter.
//!
//! Thin wrapper over egui's stock [`egui::ComboBox`] that pulls the
//! current label + the full option list out of the param's
//! [`truce_params::Params::format_value`] and wires the chosen option
//! back through [`PluginContext::automate`]. Useful for `EnumParam`
//! or `IntParam` parameters with more options than a click-to-cycle
//! `param_selector` reasonably fits.

use truce_core::cast::discrete_norm;
use truce_core::editor::{PluginContext, PluginContextReadF32};
use truce_params::{Params, sample::Float};

/// Render a click-to-open dropdown for the param `id` with `label`
/// underneath. Option labels are derived from the param's range +
/// `Params::format_value` so they match what the host shows in its
/// automation lanes.
///
/// Returns the encompassing [`egui::Response`] so callers can attach
/// tooltips or chain interactions.
pub fn param_dropdown<P: Params + ?Sized>(
    ui: &mut egui::Ui,
    state: &PluginContext<P>,
    id: impl Into<u32>,
    label: &str,
) -> egui::Response {
    let id = id.into();
    let current_text = state.format_param(id);

    // Enumerate option labels via the same path the built-in GUI's
    // `get_options` closure uses (see `truce-gui/src/render_core.rs`).
    let params = state.params();
    let infos = params.param_infos();
    let Some(info) = infos.iter().find(|i| i.id == id).copied() else {
        // Param id not recognised - draw a disabled placeholder so the
        // layout doesn't collapse but no automation can land.
        let resp = ui.add_enabled(false, egui::Label::new(format!("(unknown {id})")));
        ui.label(label);
        return resp;
    };
    let count = info.range.step_count_usize() + 1;

    ui.vertical(|ui| {
        let cur_value: f32 = state.get_param(id);
        let combo_resp = egui::ComboBox::from_id_salt(("truce-egui:param_dropdown", id))
            .selected_text(&current_text)
            .show_ui(ui, |ui| {
                for i in 0..count {
                    let norm = discrete_norm(i, count);
                    let plain = info.range.denormalize(norm);
                    let label_text = params
                        .format_value(id, plain)
                        .unwrap_or_else(|| format!("{plain:.0}"));
                    let norm_f32 = f32::from_f64(norm);
                    let selected = (cur_value - norm_f32).abs() < f32::EPSILON.max(1e-4);
                    if ui.selectable_label(selected, label_text).clicked() {
                        state.automate(id, norm);
                    }
                }
            });
        ui.label(label);
        combo_resp.response
    })
    .inner
}
