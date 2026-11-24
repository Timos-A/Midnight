use eframe::egui::{self, Ui};

#[derive(Default)]
pub struct OutputDeviceSelector {
    selected_device: Option<usize>,
}

impl OutputDeviceSelector {
    pub fn new(selected_device: Option<usize>) -> Self {
        Self { selected_device }
    }

    /// Draws the output selector and returns the selected device index.
    pub fn ui(&mut self, ui: &mut Ui, devices: &[String]) -> Option<usize> {
        if devices.is_empty() {
            ui.label("No audio output devices found.");
            return None;
        }

        if self
            .selected_device
            .is_none_or(|index| index >= devices.len())
        {
            self.selected_device = Some(0);
        }

        let selected_name = self
            .selected_device
            .and_then(|index| devices.get(index))
            .map(String::as_str)
            .unwrap_or("Select an output device");

        egui::ComboBox::from_label("Sound output")
            .selected_text(selected_name)
            .show_ui(ui, |ui| {
                for (index, name) in devices.iter().enumerate() {
                    ui.selectable_value(&mut self.selected_device, Some(index), name);
                }
            });

        self.selected_device
    }
}
