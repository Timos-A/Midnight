use eframe::egui::{self, Color32, Ui};

use crate::project::{Project, TrackKind};

pub enum ArrangementAction {
    AddTrack,
    OpenTrackSettings(usize),
    SetMuted { index: usize, muted: bool },
    DeleteTrack(usize),
}

pub fn show(
    ui: &mut Ui,
    project: &Project,
    selected_track: Option<usize>,
) -> Option<ArrangementAction> {
    ui.heading("Arrangement");

    let mut action = None;
    ui.horizontal(|ui| {
        if ui.button("+ Add Track").clicked() {
            action = Some(ArrangementAction::AddTrack);
        }
        ui.weak("Create a track, then click its name to configure it.");
    });
    ui.add_space(8.0);

    if project.tracks.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(32.0);
            ui.heading("Your arrangement is empty");
            ui.label("Add a track to start building your session.");
        });
        return action;
    }

    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width().max(700.0));
            ui.horizontal(|ui| {
                ui.add_sized([200.0, 24.0], egui::Label::new("Track / bars"));
                for bar in 1..=9 {
                    ui.add_sized([96.0, 24.0], egui::Label::new(format!("{bar}")));
                }
            });
            ui.separator();

            for (index, track) in project.tracks.iter().enumerate() {
                ui.horizontal(|ui| {
                    let response = ui.add_sized(
                        [200.0, 44.0],
                        egui::Button::new(format!(
                            "{}{}  ·  {}",
                            track.name,
                            if track.muted { " (muted)" } else { "" },
                            track.kind.label()
                        ))
                        .selected(selected_track == Some(index)),
                    );
                    if response.clicked() {
                        action = Some(ArrangementAction::OpenTrackSettings(index));
                    }
                    response.context_menu(|ui| {
                        let mut muted = track.muted;
                        if ui.checkbox(&mut muted, "Mute track").changed() {
                            action = Some(ArrangementAction::SetMuted { index, muted });
                        }
                        ui.separator();
                        if ui.button("Delete Track").clicked() {
                            action = Some(ArrangementAction::DeleteTrack(index));
                        }
                    });

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
                                TrackKind::Blank => "Blank track",
                                TrackKind::Instrument => match track.instrument {
                                    Some(crate::project::InstrumentKind::Synth) => "Built-in Synth",
                                    None => "Choose an instrument in track settings",
                                },
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
    action
}
