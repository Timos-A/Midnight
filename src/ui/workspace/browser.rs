use eframe::egui::{self, Ui};

pub fn show(ui: &mut Ui) {
    ui.heading("Browser");
    ui.add_space(8.0);

    egui::CollapsingHeader::new("Samples")
        .default_open(true)
        .show(ui, |ui| {
            ui.weak("No samples in this project yet.");
        });

    egui::CollapsingHeader::new("Plugin Library")
        .default_open(true)
        .show(ui, |ui| {
            ui.weak("No plugins available yet.");
        });
}
