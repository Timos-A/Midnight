use eframe::egui::{self, Ui};

use crate::ui::output_device_selector::OutputDeviceSelector;

pub fn show(
    ui: &mut Ui,
    output_device_selector: &mut OutputDeviceSelector,
    output_devices: &[String],
) -> Option<usize> {
    ui.horizontal(|ui| {
        ui.label("Transport");
        ui.add_enabled(false, egui::Button::new("⏮"));
        ui.add_enabled(false, egui::Button::new("▶"));
        ui.add_enabled(false, egui::Button::new("■"));
        ui.add_enabled(false, egui::Button::new("● Record"));
        ui.separator();
        ui.label("00:00:00");
        ui.separator();
        output_device_selector.ui(ui, output_devices)
    })
    .inner
}
