use std::sync::{Arc, atomic::AtomicU8};

use eframe::{egui, epaint};

use crate::{
    app_name,
    definitions::built_info,
    l10n::{L10N, Term},
    views::utils::two_cells_v,
};

pub const ABOUT_WINDOW_OPEN_STATE_CLOSED: u8 = 0;
pub const ABOUT_WINDOW_OPEN_STATE_OPEN: u8 = 1;
pub const ABOUT_WINDOW_OPEN_STATE_OPEN_DESIRING_FOCUS: u8 = 2;

pub struct AboutWindow {
    l: L10N,
}

impl AboutWindow {
    pub fn new(l: L10N) -> Self {
        Self { l }
    }
}

impl AboutWindow {
    pub fn window(&mut self, ui: &mut egui::Ui, open_state: Arc<AtomicU8>) {
        ui.ctx().show_viewport_deferred(
            egui::ViewportId::from_hash_of("AboutWindow"),
            egui::ViewportBuilder::default()
                .with_title(format!("{} {}", self.l.tl(&Term::About), app_name!()))
                .with_inner_size((240.0, 240.0))
                .with_resizable(false)
                .with_always_on_top(),
            move |ui, class| {
                if class == egui::ViewportClass::EmbeddedWindow {
                    unimplemented!("Embedded viewports are not supported yet.");
                } else {
                    egui::CentralPanel::default().show(ui, |ui| {
                        if ui.input(|i| i.viewport().close_requested()) {
                            open_state.store(
                                ABOUT_WINDOW_OPEN_STATE_CLOSED,
                                std::sync::atomic::Ordering::Relaxed,
                            );
                            return;
                        }
                        if open_state.load(std::sync::atomic::Ordering::Relaxed)
                            == ABOUT_WINDOW_OPEN_STATE_OPEN_DESIRING_FOCUS
                        {
                            // FIXME: doesn't seem to work correctly (macOS 15):
                            // The window only gets focused when the mouse
                            // pointer moves into the window.
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Focus);
                            open_state.store(
                                ABOUT_WINDOW_OPEN_STATE_OPEN,
                                std::sync::atomic::Ordering::Relaxed,
                            );
                        }

                        two_cells_v(
                            ui,
                            ui.available_height(),
                            |ui| {
                                ui.vertical_centered_justified(|ui| {
                                    ui.label(egui::RichText::new(app_name!()).heading());
                                });
                            },
                            |ui| {
                                egui::Frame::new()
                                    .stroke(epaint::Stroke::new(1.0, epaint::Color32::GRAY))
                                    .show(ui, |ui| {
                                        egui::ScrollArea::both()
                                    .scroll_bar_visibility(
                                        egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded,
                                    )
                                    .show(ui, |ui| {
                                        Self::information_grid_inner(ui);
                                    });
                                    });
                            },
                        );
                    });
                }
            },
        );
    }

    fn information_grid_inner(ui: &mut egui::Ui) {
        let height = egui::Grid::new("AboutInformation")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                let version = env!("CARGO_PKG_VERSION");
                ui.label("Version");
                ui.label(version);
                ui.end_row();

                if let Some(hash) = built_info::GIT_COMMIT_HASH {
                    ui.label("Commit Hash");
                    ui.label(hash);
                    ui.end_row();
                }

                let repo_link = env!("CARGO_PKG_REPOSITORY");
                if !repo_link.is_empty() {
                    ui.label("Repository");
                    ui.hyperlink(repo_link);
                    ui.end_row();
                }
            })
            .response
            .rect
            .height();
        if height < ui.available_height() {
            ui.allocate_space(egui::vec2(0.0, ui.available_height() - height));
        }
    }
}
