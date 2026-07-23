use eframe::egui;

pub fn two_cells_h(
    ui: &mut egui::Ui,
    full_width: f32,
    left: impl FnOnce(&mut egui::Ui),
    right: impl FnOnce(&mut egui::Ui),
) {
    let left_width = ui
        .horizontal(|ui| {
            left(ui);
        })
        .response
        .rect
        .width();

    // TODO: find out what this magic number is.
    const MAGIC: f32 = 16.0;

    let right_width = full_width - left_width - ui.style().spacing.item_spacing.x - MAGIC;
    egui_extras::StripBuilder::new(ui)
        .size(egui_extras::Size::exact(right_width))
        .horizontal(|mut strip| {
            strip.cell(|ui| right(ui));
        });
}

pub fn two_cells_v(
    ui: &mut egui::Ui,
    full_height: f32,
    top: impl FnOnce(&mut egui::Ui),
    bottom: impl FnOnce(&mut egui::Ui),
) {
    let top_height = ui
        .vertical(|ui| {
            top(ui);
        })
        .response
        .rect
        .height();

    // TODO: find out what this magic number is.
    const MAGIC: f32 = 16.0;

    let bottom_height = full_height - top_height - ui.style().spacing.item_spacing.y - MAGIC;
    egui_extras::StripBuilder::new(ui)
        .size(egui_extras::Size::exact(bottom_height))
        .vertical(|mut strip| {
            strip.cell(|ui| bottom(ui));
        });
}
