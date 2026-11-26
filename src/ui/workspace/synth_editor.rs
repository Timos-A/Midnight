use eframe::egui::{self, Context, Id};

use crate::instruments::synth::controls::SynthControls;

pub fn show(ctx: &Context, track_index: usize, controls: &mut SynthControls) -> bool {
    let mut close_requested = false;
    let response = egui::Modal::new(Id::unique(("synth_plugin_ui", track_index))).show(ctx, |ui| {
        ui.set_min_width(760.0);
        ui.set_max_width(960.0);
        controls.ui(ui);
        ui.separator();
        if ui.button("Close Synth UI").clicked() {
            close_requested = true;
        }
    });

    !close_requested && !response.backdrop_response.clicked()
}
