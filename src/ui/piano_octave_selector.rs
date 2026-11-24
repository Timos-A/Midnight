use eframe::egui::{self, Ui};

#[derive(Default)]
pub struct PianoOctaveSelector {
    octave_shift: i8,
}

impl PianoOctaveSelector {
    pub fn ui(&mut self, ui: &mut Ui) {
        let low_octave = 4 + self.octave_shift;
        let selected_range = format!("C{low_octave} - B{}", low_octave + 1);

        ui.horizontal(|ui| {
            ui.label("Piano range");
            egui::ComboBox::from_id_salt("piano_octave_selector")
                .selected_text(selected_range)
                .show_ui(ui, |ui| {
                    for shift in -2..=2 {
                        let low_octave = 4 + shift;
                        ui.selectable_value(
                            &mut self.octave_shift,
                            shift,
                            format!("C{low_octave} - B{}", low_octave + 1),
                        );
                    }
                });
        });
    }

    pub fn octave_shift(&self) -> i8 {
        self.octave_shift
    }
}
