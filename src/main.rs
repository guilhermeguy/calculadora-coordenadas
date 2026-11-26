// Hide the console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod geo;

use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Calculadora de Coordenadas")
            .with_inner_size([app::WIDTH, 260.0])
            .with_resizable(false)
            .with_maximize_button(false),
        ..Default::default()
    };
    eframe::run_native(
        "Calculadora de Coordenadas",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
