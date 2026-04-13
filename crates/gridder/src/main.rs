#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod font_loading;

use snafu::prelude::*;

use app::GridderApp;

#[snafu::report]
fn main() -> Result<(), snafu::Whatever> {
    tracing_subscriber::fmt::init();

    tracing::info!("Starting Gridder…");

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Gridder",
        native_options,
        Box::new(|cc| Ok(Box::new(GridderApp::new(cc)))),
    )
    .whatever_context("Failed to run the eframe App.")?;

    Ok(())
}
