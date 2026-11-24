use std::ops::RangeInclusive;

use eframe::egui::{Color32, Sense, Stroke, Ui, Vec2};

#[derive(Clone, Copy)]
pub enum Scale {
    Linear,
    Logarithmic,
    Squared,
}

pub fn show(
    ui: &mut Ui,
    label: &str,
    value: &mut f32,
    default: f32,
    range: RangeInclusive<f32>,
    scale: Scale,
    value_text: impl FnOnce(f32) -> String,
) {
    let min = *range.start();
    let max = *range.end();

    ui.allocate_ui_with_layout(
        Vec2::new(68.0, 100.0),
        eframe::egui::Layout::top_down(eframe::egui::Align::Center),
        |ui| {
            ui.label(label);
            let (rect, response) = ui.allocate_exact_size(Vec2::new(62.0, 54.0), Sense::drag());
            if response.dragged() {
                let position = to_normalized(*value, min, max, scale);
                let next_position = (position - response.drag_motion().y / 150.0).clamp(0.0, 1.0);
                *value = from_normalized(next_position, min, max, scale);
            }
            if response.double_clicked() {
                *value = default;
            }
            response.on_hover_text("Drag vertically to adjust; double-click to reset");

            let painter = ui.painter();
            let center = rect.center();
            let radius = 21.0;
            painter.circle_filled(center, radius, Color32::from_rgb(46, 49, 56));
            painter.circle_stroke(center, radius, Stroke::new(2.0, Color32::GRAY));

            for tick in 0..=10 {
                let position = tick as f32 / 10.0;
                let angle =
                    -std::f32::consts::FRAC_PI_2 * 1.5 + position * std::f32::consts::PI * 1.5;
                let inner = center + Vec2::new(angle.cos(), angle.sin()) * (radius - 3.0);
                let outer = center + Vec2::new(angle.cos(), angle.sin()) * (radius + 1.0);
                painter.line_segment(
                    [inner, outer],
                    Stroke::new(
                        1.5,
                        if position <= to_normalized(*value, min, max, scale) {
                            Color32::from_rgb(70, 150, 255)
                        } else {
                            Color32::DARK_GRAY
                        },
                    ),
                );
            }

            let angle = -std::f32::consts::FRAC_PI_2 * 1.5
                + to_normalized(*value, min, max, scale) * std::f32::consts::PI * 1.5;
            let tip = center + Vec2::new(angle.cos(), angle.sin()) * (radius - 7.0);
            painter.line_segment([center, tip], Stroke::new(2.5, Color32::WHITE));
            ui.label(value_text(*value));
        },
    );
}

fn to_normalized(value: f32, min: f32, max: f32, scale: Scale) -> f32 {
    let linear = ((value - min) / (max - min)).clamp(0.0, 1.0);
    match scale {
        Scale::Linear => linear,
        Scale::Logarithmic => (value.max(min).ln() - min.ln()) / (max.ln() - min.ln()),
        Scale::Squared => linear.sqrt(),
    }
}

fn from_normalized(position: f32, min: f32, max: f32, scale: Scale) -> f32 {
    match scale {
        Scale::Linear => min + (max - min) * position,
        Scale::Logarithmic => (min.ln() + (max.ln() - min.ln()) * position).exp(),
        Scale::Squared => min + (max - min) * position * position,
    }
}
