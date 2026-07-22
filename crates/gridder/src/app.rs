use std::sync::{Arc, RwLock};

use eframe::egui::{self, ViewportBuilder};

use crate::{
    l10n::L10N,
    states::settings::Settings,
    utils::font_loading::load_system_fonts,
    views::windows::{
        project_window::{OpenedProjects, ProjectWindow},
        welcome_window::WelcomeWindow,
    },
};

pub struct GridderApp {
    l: L10N,
    #[expect(dead_code)]
    settings: Arc<RwLock<Settings>>,

    projects: Arc<RwLock<OpenedProjects>>,

    welcome_window: WelcomeWindow,
}

impl GridderApp {
    pub fn name() -> &'static str {
        "Voĉa Gridder @ Lazurema"
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
        let mut fonts = egui::FontDefinitions::default();
        load_system_fonts(&mut fonts);
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        cc.egui_ctx.set_fonts(fonts);

        let l10n = L10N::new();
        let settings = Arc::new(RwLock::new(Settings::new(l10n.clone())));

        Self {
            l: l10n.clone(),
            settings: settings.clone(),
            projects: Arc::new(RwLock::new(OpenedProjects::new())),
            welcome_window: WelcomeWindow::new(l10n.clone(), settings.clone()),
        }
    }
}

impl eframe::App for GridderApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.welcome_window.ui(ui, self.projects.clone());

        for project in self
            .projects
            .read()
            .expect("Failed to acquire read lock on projects.")
            .iter()
        {
            let projects = self.projects.clone();
            let project = project.clone();
            ProjectWindow::window(ui, self.l.clone(), project, projects);
        }
    }
}
