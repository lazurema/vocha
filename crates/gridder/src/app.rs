use std::sync::{Arc, Mutex, RwLock};

use eframe::egui::{self, ViewportBuilder, Widget as _};

use crate::{
    font_loading::load_system_fonts,
    l10n::{L10N, Term},
    projects::{self, Project, ProjectPreview},
};

pub struct GridderApp {
    l10n: L10N,
    is_always_on_top: bool,

    projects: Arc<RwLock<Vec<Arc<(uuid::Uuid, Mutex<Project>)>>>>,
}

impl GridderApp {
    pub fn name() -> &'static str {
        "Gridder"
    }

    pub fn native_options() -> eframe::NativeOptions {
        eframe::NativeOptions {
            viewport: ViewportBuilder::default()
                .with_inner_size((240.0, 240.0))
                .with_resizable(false)
                .with_always_on_top(),
            ..Default::default()
        }
    }

    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = load_system_fonts(egui::FontDefinitions::default());
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        cc.egui_ctx.set_fonts(fonts);

        let l10n = L10N::new();

        Self {
            l10n,
            is_always_on_top: true,
            projects: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl eframe::App for GridderApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("top_bar")
            .frame(egui::Frame::new().inner_margin(4))
            .show_inside(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    self.language_selector(ui);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        self.settings(ui);

                        let repo_link = env!("CARGO_PKG_REPOSITORY");
                        if !repo_link.is_empty() {
                            if ui.button(egui_phosphor::regular::HOUSE_LINE).clicked() {
                                ui.ctx().send_cmd(egui::OutputCommand::OpenUrl(
                                    egui::OpenUrl::new_tab(env!("CARGO_PKG_REPOSITORY")),
                                ));
                            }
                        }
                    })
                })
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            let project_preview = ProjectPreview::extract_from_ui(ui);

            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::TopDown)
                    .with_cross_justify(false),
                |ui| {
                    if let Some(project_preview) = project_preview {
                        if let Some(project) =
                            Project::try_from_preview_from_dropping_files(&project_preview)
                        {
                            self.projects
                                .write()
                                .expect("Failed to acquire write lock on projects.")
                                .push(Arc::new((project.id(), Mutex::new(project))));
                        } else {
                            project_preview.ui(ui, &self.l10n);
                        }
                    } else {
                        egui::Label::new(self.l10n.tl(&Term::DropHintText {
                            supported_audio_extensions: projects::SUPPORTED_AUDIO_EXTENSIONS,
                        }))
                        .selectable(false)
                        .ui(ui);
                    }
                },
            );

            for project in self
                .projects
                .read()
                .expect("Failed to acquire read lock on projects.")
                .iter()
            {
                let projects = self.projects.clone();
                let project = project.clone();
                let l10n = self.l10n.clone();
                ui.ctx().show_viewport_deferred(
                    egui::ViewportId::from_hash_of(project.0),
                    egui::ViewportBuilder::default()
                        .with_title(l10n.tl(&Term::GridderProject { name: None })),
                    move |ui, class| {
                        let mut project = project
                            .1
                            .lock()
                            .expect("Failed to acquire lock on project.");
                        if class == egui::ViewportClass::EmbeddedWindow {
                            // currently, the project UI is based on the
                            // assumption that it has its own viewport.
                            unimplemented!("Embedded viewports are not supported yet.");
                            // project.ui(ui, l10n.clone());
                        } else {
                            egui::CentralPanel::default().show_inside(ui, |ui| {
                                project.ui(ui, l10n.clone());

                                if ui.input(|i| i.viewport().close_requested()) {
                                    projects
                                        .write()
                                        .expect("Failed to acquire write lock on projects.")
                                        .retain(|p| p.0 != project.id());
                                }
                            });
                        }
                    },
                );
            }
        });
    }
}

impl GridderApp {
    fn language_selector(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_id_salt("language_selector")
            .selected_text(format!(
                "{} {}",
                egui_phosphor::regular::TRANSLATE,
                self.l10n.current_language().display_name()
            ))
            .show_ui(ui, |ui| {
                let mut new_language_code = self.l10n.current_language().code();
                for language_code in L10N::available_language_codes() {
                    if let Some(language) = self.l10n.get_language(language_code) {
                        ui.selectable_value(
                            &mut new_language_code,
                            language.code(),
                            language.display_name(),
                        );
                    }
                }
                if new_language_code != self.l10n.current_language().code() {
                    self.l10n.set_current_language(new_language_code);
                }
            });
    }

    fn settings(&mut self, ui: &mut egui::Ui) {
        egui::containers::menu::MenuButton::new(egui_phosphor::regular::GEAR)
            .config(
                egui::containers::menu::MenuConfig::new()
                    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside),
            )
            .ui(ui, |ui| {
                egui::Grid::new("settings_grid").show(ui, |ui| {
                    self.settings_always_on_top_toggle(ui);
                    ui.end_row();
                    self.settings_theme_selector(ui);
                    ui.end_row();
                });
            });
    }

    fn settings_always_on_top_toggle(&mut self, ui: &mut egui::Ui) {
        let old = self.is_always_on_top;
        ui.label(self.l10n.tl(&Term::AlwaysOnTopToggleLabel));
        egui::Checkbox::without_text(&mut self.is_always_on_top).ui(ui);
        if old != self.is_always_on_top {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                    if self.is_always_on_top {
                        egui::WindowLevel::AlwaysOnTop
                    } else {
                        egui::WindowLevel::Normal
                    },
                ));
        }
    }

    fn settings_theme_selector(&mut self, ui: &mut egui::Ui) {
        ui.label(self.l10n.tl(&Term::Theme));

        ui.horizontal(|ui| {
            let mut theme = ui.theme();
            let old_theme = theme.clone();

            ui.scope(|ui| {
                ui.style_mut().spacing.item_spacing.x = 0.0;
                ui.selectable_value(&mut theme, egui::Theme::Dark, egui_phosphor::regular::MOON);
                ui.selectable_value(&mut theme, egui::Theme::Light, egui_phosphor::regular::SUN);
            });

            if theme != old_theme {
                ui.set_theme(theme);
            }
        });
    }
}
