use std::sync::{Arc, RwLock};

use eframe::egui::{self, ViewportBuilder};

use crate::{
    app_title,
    l10n::L10N,
    states::settings::Settings,
    utils::font_loading::load_system_fonts,
    views::windows::{
        about_window::{ABOUT_WINDOW_OPEN_STATE_CLOSED, AboutWindow},
        project_window::{OpenedProjects, ProjectWindow},
        welcome_window::WelcomeWindow,
    },
};

pub struct GridderApp {
    l: L10N,
    #[expect(dead_code)]
    settings: Arc<RwLock<Settings>>,

    welcome_window: WelcomeWindow,
    about_window: AboutWindow,
    about_window_open_state: Arc<std::sync::atomic::AtomicU8>,
    projects: Arc<RwLock<OpenedProjects>>,
}

impl GridderApp {
    pub fn name() -> &'static str {
        app_title!()
    }

    pub fn native_options() -> eframe::NativeOptions {
        eframe::NativeOptions {
            viewport: ViewportBuilder::default()
                .with_inner_size((320.0, 320.0))
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
            welcome_window: WelcomeWindow::new(l10n.clone(), settings.clone()),
            about_window: AboutWindow::new(l10n.clone()),
            about_window_open_state: Arc::new(std::sync::atomic::AtomicU8::new(
                ABOUT_WINDOW_OPEN_STATE_CLOSED,
            )),
            projects: Arc::new(RwLock::new(OpenedProjects::new())),
        }
    }
}

impl eframe::App for GridderApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.welcome_window.ui(
            ui,
            self.about_window_open_state.clone(),
            self.projects.clone(),
        );

        if self
            .about_window_open_state
            .load(std::sync::atomic::Ordering::Relaxed)
            != ABOUT_WINDOW_OPEN_STATE_CLOSED
        {
            self.about_window
                .window(ui, self.about_window_open_state.clone());
        }

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
