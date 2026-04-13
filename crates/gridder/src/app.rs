use eframe::egui;

use crate::font_loading::load_system_fonts;

pub struct GridderApp {}

impl GridderApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let fonts = load_system_fonts(egui::FontDefinitions::default());
        cc.egui_ctx.set_fonts(fonts);

        Self {}
    }
}

impl eframe::App for GridderApp {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        eframe::egui::CentralPanel::default().show_inside(ui, |ui| {
            gridder_egui_widgets::hello(ui);
        });
    }
}
