use std::sync::{Arc, RwLock};

use eframe::egui::{self, Widget as _};

use crate::{
    l10n::{L10N, Term},
    states::{projects::ProjectPreviewState, settings::Settings},
    views::{
        widgets::project_preview::ProjectPreviewWidget,
        windows::{
            SingletonWindowOpenState,
            project_window::{OpenedProjects, ProjectWindow},
        },
    },
};

pub struct WelcomeWindow {
    l: L10N,
    settings: Arc<RwLock<Settings>>,
}

impl WelcomeWindow {
    pub fn new(l: L10N, settings: Arc<RwLock<Settings>>) -> Self {
        Self { l, settings }
    }
}

impl WelcomeWindow {
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        about_window_open_state: SingletonWindowOpenState,
        settings_window_open_state: SingletonWindowOpenState,
        projects: Arc<RwLock<OpenedProjects>>,
    ) {
        egui::Panel::top("top_bar")
            .frame(egui::Frame::new().inner_margin(4))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    self.language_selector(ui);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(egui_phosphor::regular::GEAR).clicked() {
                            settings_window_open_state.open();
                        }
                        if ui.button(egui_phosphor::regular::INFO).clicked() {
                            about_window_open_state.open();
                        }
                    })
                })
            });

        egui::CentralPanel::default().show(ui, |ui| {
            let project_preview = ProjectPreviewState::extract_from_ui(ui);

            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::TopDown)
                    .with_cross_justify(false),
                |ui| {
                    if let Some(project_preview) = project_preview {
                        if let Some(project) = ProjectWindow::try_from_preview_from_dropping_files(
                            self.l.clone(),
                            &project_preview,
                        ) {
                            projects
                                .write()
                                .expect("Failed to acquire write lock on projects.")
                                .add(project);
                        } else {
                            ProjectPreviewWidget::ui(project_preview, ui, &self.l);
                        }
                    } else {
                        egui::Label::new(self.l.tl(&Term::DropHintText {
                            supported_audio_extensions:
                                crate::states::projects::SUPPORTED_AUDIO_EXTENSIONS,
                        }))
                        .selectable(false)
                        .ui(ui);
                    }
                },
            );
        });
    }

    fn language_selector(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_id_salt("language_selector")
            .selected_text(format!(
                "{} {}",
                egui_phosphor::regular::TRANSLATE,
                self.l.current_language().display_name()
            ))
            .show_ui(ui, |ui| {
                let mut new_language_code = self.l.current_language().code();
                for language_code in L10N::available_language_codes() {
                    if let Some(language) = self.l.get_language(language_code) {
                        ui.selectable_value(
                            &mut new_language_code,
                            language.code(),
                            language.display_name(),
                        );
                    }
                }
                if new_language_code != self.l.current_language().code() {
                    self.settings
                        .write()
                        .expect("Failed to acquire write lock on settings.")
                        .l10n_mut
                        .set_current_language(new_language_code);
                }
            });
    }
}
