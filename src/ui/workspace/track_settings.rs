use eframe::egui::{self, Context, Id};

use crate::project::{InstrumentKind, Track, TrackKind};

pub struct TrackSettingsResult {
    pub open: bool,
}

pub fn show(
    ctx: &Context,
    track_index: usize,
    track: &mut Track,
    is_open: &mut bool,
) -> TrackSettingsResult {
    egui::Window::new("Track Settings")
        .id(Id::unique(("track_settings_window", track_index)))
        .open(is_open)
        .collapsible(false)
        .resizable(false)
        .default_width(340.0)
        .show(ctx, |ui| {
            ui.label("Track name");
            ui.text_edit_singleline(&mut track.name);
            ui.add_space(8.0);

            ui.label("Track type");
            egui::ComboBox::from_id_salt(("track_type", track_index))
                .selected_text(track.kind.label())
                .show_ui(ui, |ui| {
                    for kind in TrackKind::ALL {
                        ui.selectable_value(&mut track.kind, kind, kind.label());
                    }
                });

            if track.kind == TrackKind::Instrument {
                track.instrument.get_or_insert(InstrumentKind::Synth);
                ui.add_space(8.0);
                ui.label("Instrument");
                egui::ComboBox::from_id_salt(("track_instrument", track_index))
                    .selected_text(
                        track
                            .instrument
                            .map(InstrumentKind::label)
                            .unwrap_or("Choose an instrument"),
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut track.instrument,
                            Some(InstrumentKind::Synth),
                            InstrumentKind::Synth.label(),
                        );
                    });
            } else {
                track.instrument = None;
            }

            ui.separator();
            ui.checkbox(&mut track.muted, "Mute track");
            match track.kind {
                TrackKind::Blank => {
                    ui.weak("Choose a track type when you are ready to use this track.");
                }
                TrackKind::Midi => {
                    ui.weak("MIDI recording and clips are not connected yet.");
                }
                TrackKind::Audio => {
                    ui.weak("Audio recording and import are not connected yet.");
                }
                TrackKind::Instrument => {}
            }
        });

    TrackSettingsResult { open: *is_open }
}
