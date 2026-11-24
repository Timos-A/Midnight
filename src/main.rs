mod app;
mod audio;
mod instruments;
mod project;
mod ui;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    eframe::run_native(
        "Midnight",
        native_options,
        Box::new(|_creation_context| Ok(Box::new(app::App::default()))),
    )
}
