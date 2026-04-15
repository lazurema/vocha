use std::sync::{Arc, RwLock};

use eframe::egui::{self, ViewportBuilder, Widget as _};

use crate::{
    font_loading::load_system_fonts,
    l10n::{L10N, Term},
    projects::{self, Project, ProjectPreview},
};

pub struct GridderApp {
    l10n: L10N,
    is_always_on_top: bool,

    projects: Arc<RwLock<Vec<Arc<Project>>>>,
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
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("top_bar")
            .frame(egui::Frame::new().inner_margin(4))
            .show_inside(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.visuals_mut().button_frame = false;
                    self.language_selector(ui);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        self.always_on_top_toggle(ui);
                    })
                })
            });

        eframe::egui::CentralPanel::default().show_inside(ui, |ui| {
            /// Unfortunately, `ui.response().contains_pointer()` always returns
            ///  `false` while files are being dragged.
            /// See: <https://github.com/emilk/egui/issues/4655>
            ///
            /// TODO(umajho): Investigate this in `egui`.
            ///
            /// As of now (2026/04/14), there is no luck: <https://github.com/rust-windowing/winit/issues/720#issuecomment-1290156438>
            ///
            /// A member of `winit` said there: “The cursor position should be
            /// broadcasted during the drag and drop. It could be that a
            /// particular platform isn't doing so which could indicate a bug.”
            /// If that’s the case, then at least both macOS and Windows are
            /// affected.
            fn get_project_preview(ui: &mut egui::Ui) -> Option<ProjectPreview> {
                if true || ui.response().contains_pointer() {
                    ui.input(|input| {
                        ProjectPreview::try_from_dropped_files(input.raw.dropped_files.iter())
                            .or_else(|| {
                                ProjectPreview::try_from_hovered_files(
                                    input.raw.hovered_files.iter(),
                                )
                            })
                    })
                } else {
                    None
                }
            }
            let project_preview = get_project_preview(ui);

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
                                .push(Arc::new(project));
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
                    egui::ViewportId::from_hash_of(project.id()),
                    egui::ViewportBuilder::default(),
                    move |ui, class| {
                        if class == egui::ViewportClass::EmbeddedWindow {
                            project.ui(ui, l10n.clone());
                        } else {
                            egui::CentralPanel::default().show_inside(ui, |ui| {
                                project.ui(ui, l10n.clone());

                                if ui.input(|i| i.viewport().close_requested()) {
                                    projects
                                        .write()
                                        .expect("Failed to acquire write lock on projects.")
                                        .retain(|p| p.id() != project.id());
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
    fn language_selector(&mut self, ui: &mut eframe::egui::Ui) {
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

    fn always_on_top_toggle(&mut self, ui: &mut eframe::egui::Ui) {
        let old = self.is_always_on_top;
        ui.checkbox(
            &mut self.is_always_on_top,
            self.l10n.tl(&Term::AlwaysOnTopToggleLabel),
        );
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
}
