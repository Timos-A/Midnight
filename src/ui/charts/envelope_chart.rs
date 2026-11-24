use eframe::egui::{self, Align2, Color32, Sense, Stroke, StrokeKind, Ui, Vec2};

use crate::instruments::synth::engine::SynthSettings;

pub fn show(ui: &mut Ui, settings: SynthSettings) {
    // Respect the parent panel's width so the chart can shrink with the window.
    let width = ui.available_width().clamp(0.0, 500.0);
    let height = 150.0;
    let (area, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let painter = ui.painter_at(area);
    let background = Color32::from_rgb(30, 34, 38);
    let grid = Color32::from_rgb(60, 67, 72);
    let envelope = Color32::from_rgb(66, 226, 217);

    painter.rect_filled(area, 6.0, background);
    painter.rect_stroke(area, 6.0, Stroke::new(1.0, grid), StrokeKind::Inside);

    let horizontal_padding = 14.0_f32.min(width * 0.05);
    let left = area.left() + horizontal_padding;
    let right = area.right() - horizontal_padding;
    let top = area.top() + 12.0;
    let bottom = area.bottom() - 24.0;
    let plot_width = right - left;
    let plot_height = bottom - top;

    for division in 0..=4 {
        let fraction = division as f32 / 4.0;
        let y = bottom - plot_height * fraction;
        painter.line_segment(
            [egui::pos2(left, y), egui::pos2(right, y)],
            Stroke::new(1.0, grid),
        );
    }
    for division in 0..=6 {
        let fraction = division as f32 / 6.0;
        let x = left + plot_width * fraction;
        painter.line_segment(
            [egui::pos2(x, top), egui::pos2(x, bottom)],
            Stroke::new(1.0, grid),
        );
    }

    // Give zero-length stages a small visible width; longer stages remain proportional.
    let durations = [
        settings.delay,
        settings.attack,
        settings.hold,
        settings.decay,
        0.35, // A short visible sustain segment; sustain continues while the key is held.
        settings.release,
    ];
    let total_duration: f32 = durations.iter().sum();
    let timed_stages = total_duration - 0.35;
    let minimum_segment = 8.0;
    let flexible_width = (plot_width - minimum_segment * durations.len() as f32).max(0.0);
    let mut x_positions = [left; 7];
    for (index, duration) in durations.iter().enumerate() {
        let proportional_width = if total_duration > 0.0 {
            flexible_width * duration.max(0.0) / total_duration
        } else {
            flexible_width / durations.len() as f32
        };
        x_positions[index + 1] = x_positions[index] + minimum_segment + proportional_width;
    }
    x_positions[6] = right;

    let y_for_level = |level: f32| bottom - plot_height * level.clamp(0.0, 1.0);
    let levels = [0.0, 0.0, 1.0, 1.0, settings.sustain, settings.sustain, 0.0];
    let mut curve = vec![egui::pos2(x_positions[0], y_for_level(levels[0]))];

    for segment in 0..6 {
        for step in 1..=12 {
            let t = step as f32 / 12.0;
            let level = match segment {
                1 => 1.0 - (1.0 - t).powi(2), // attack
                3 => settings.sustain + (1.0 - settings.sustain) * (1.0 - t).powi(2), // decay
                5 => settings.sustain * (1.0 - t).powi(2), // release
                _ => levels[segment] + (levels[segment + 1] - levels[segment]) * t,
            };
            let x = x_positions[segment] + (x_positions[segment + 1] - x_positions[segment]) * t;
            curve.push(egui::pos2(x, y_for_level(level)));
        }
    }

    painter.add(egui::Shape::line(curve, Stroke::new(2.5, envelope)));
    for point_index in [1, 2, 3, 4, 5, 6] {
        painter.circle_filled(
            egui::pos2(x_positions[point_index], y_for_level(levels[point_index])),
            3.2,
            envelope,
        );
    }
    painter.text(
        egui::pos2(left, area.bottom() - 5.0),
        Align2::LEFT_BOTTOM,
        "0s",
        egui::FontId::proportional(10.0),
        Color32::LIGHT_GRAY,
    );
    painter.text(
        egui::pos2(right, area.bottom() - 5.0),
        Align2::RIGHT_BOTTOM,
        format!("{timed_stages:.1}s + held"),
        egui::FontId::proportional(10.0),
        Color32::LIGHT_GRAY,
    );
}
