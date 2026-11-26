use eframe::egui::{self, Color32, Ui};

use crate::project::{Project, TrackKind};

pub fn show(ui: &mut Ui, project: &Project, selected_track: usize) -> Option<usize> {
    ui.heading("Arrangement");
    ui.label("Timeline workspace");
    ui.add_space(8.0);

    let mut newly_selected = None;
    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width().max(700.0));
            ui.horizontal(|ui| {
                ui.add_sized([180.0, 24.0], egui::Label::new("Track / bars"));
                for bar in 1..=9 {
                    ui.add_sized([96.0, 24.0], egui::Label::new(format!("{bar}")));
                }
            });
            ui.separator();

            for (index, track) in project.tracks.iter().enumerate() {
                ui.horizontal(|ui| {
                    let track_button = egui::Button::new(format!(
                        "{}{}",
                        track.name,
                        if track.muted { " (muted)" } else { "" }
                    ))
                    .selected(selected_track == index);
                    if ui.add_sized([180.0, 44.0], track_button).clicked() {
                        newly_selected = Some(index);
                    }

                    ui.allocate_ui_with_layout(
                        egui::vec2(96.0 * 8.0, 44.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.painter().rect_stroke(
                                ui.max_rect(),
                                2.0,
                                egui::Stroke::new(1.0, Color32::from_gray(65)),
                                egui::StrokeKind::Inside,
                            );
                            let message = match track.kind {
                                TrackKind::Synth => "Play this instrument below",
                                TrackKind::Midi => {
                                    "MIDI clip lane · recording and editing coming next"
                                }
                                TrackKind::Audio => {
                                    "Audio lane · recording and editing coming next"
                                }
                            };
                            ui.centered_and_justified(|ui| {
                                ui.weak(message);
                            });
                        },
                    );
                });
            }
        });
    ui.separator();
    newly_selected
}
