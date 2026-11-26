use eframe::egui::{self, Ui};

pub fn show(ui: &mut Ui, project_name: &str) {
    ui.horizontal(|ui| {
        ui.heading("Midnight");
        ui.separator();
        ui.label(project_name);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_enabled(false, egui::Button::new("Save"))
                .on_hover_text("Project saving is not available yet");
            ui.add_enabled(false, egui::Button::new("Export WAV"))
                .on_hover_text("Audio export is not available yet");
        });
    });
}
