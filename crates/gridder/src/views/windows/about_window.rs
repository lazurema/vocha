use std::sync::LazyLock;

use eframe::{egui, epaint};

use crate::{
    app_name,
    definitions::built_info,
    l10n::{L10N, Term},
    views::{
        utils::{two_cells_v, with_min_height},
        windows::SingletonWindowOpenState,
    },
};

static VIEWPORT_ID: LazyLock<egui::ViewportId> =
    LazyLock::new(|| egui::ViewportId::from_hash_of("AboutWindow"));

pub struct AboutWindow {
    l: L10N,
}

impl AboutWindow {
    pub fn new(l: L10N) -> Self {
        Self { l }
    }
}

impl AboutWindow {
    pub fn window(&mut self, ui: &mut egui::Ui, open_state: SingletonWindowOpenState) {
        let widget = AboutWidget::from_window_ref(self);

        if open_state.pop_is_desiring_focus() {
            ui.send_viewport_cmd_to(*VIEWPORT_ID, egui::ViewportCommand::Focus);
        }

        ui.ctx().show_viewport_deferred(
            *VIEWPORT_ID,
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
                            open_state.close();
                            return;
                        }
                        widget.ui(ui);
                    });
                }
            },
        );
    }
}

struct AboutWidget {
    #[expect(dead_code)]
    l: L10N,
}

impl AboutWidget {
    fn from_window_ref(window: &AboutWindow) -> Self {
        Self {
            l: window.l.clone(),
        }
    }
}

impl AboutWidget {
    fn ui(&self, ui: &mut egui::Ui) {
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
                                self.information_grid_inner(ui);
                            });
                    });
            },
        );
    }

    fn information_grid_inner(&self, ui: &mut egui::Ui) {
        with_min_height(ui, ui.available_height(), |ui| {
            egui::Grid::new("AboutInformation")
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
        });
    }
}
