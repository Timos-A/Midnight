use eframe::egui::{self, Ui};

use crate::project::{Project, TrackKind};

pub enum SidebarAction {
    AddTrack(TrackKind),
    SelectTrack(usize),
    SetMuted { index: usize, muted: bool },
}

pub fn show(ui: &mut Ui, project: &Project, selected_track: usize) -> Option<SidebarAction> {
    let mut action = None;
    ui.heading("Tracks");
    ui.horizontal(|ui| {
        for (label, kind) in [
            ("+ Synth", TrackKind::Synth),
            ("+ MIDI", TrackKind::Midi),
            ("+ Audio", TrackKind::Audio),
        ] {
            if ui.button(label).clicked() {
                action = Some(SidebarAction::AddTrack(kind));
            }
        }
    });
    ui.separator();

    for (index, track) in project.tracks.iter().enumerate() {
        ui.horizontal(|ui| {
            if ui
                .selectable_label(
                    selected_track == index,
                    format!("{}  ·  {}", track.name, track.kind.label()),
                )
                .clicked()
            {
                action = Some(SidebarAction::SelectTrack(index));
            }

            let mut muted = track.muted;
            if ui.checkbox(&mut muted, "M").changed() {
                action = Some(SidebarAction::SetMuted { index, muted });
            }
        });
    }

    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        ui.label("Tracks are the building blocks of a session.");
    });
    action
}
