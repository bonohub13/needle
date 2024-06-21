/* Hide console when running a release build under Windows */
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_base;

use app_base::AppBase;
use eframe::egui;

const APP_NAME: &'static str = "needle";

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1920.0, 1080.0]),
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|_cc| Box::new(AppBase::new(APP_NAME))),
    )
}
