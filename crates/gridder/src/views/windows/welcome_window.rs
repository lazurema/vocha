use std::sync::{Arc, RwLock};

use eframe::egui::{self, Widget as _};

use crate::{
    l10n::{L10N, Term},
    states::{projects::ProjectPreviewState, settings::Settings},
    views::{
        widgets::{language_selector::LanguageSelector, project_preview::ProjectPreviewWidget},
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
                    LanguageSelector::language_selector(self.l.clone(), &self.settings, ui);
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
}
