use std::sync::{Arc, LazyLock, RwLock};

use eframe::egui;

use crate::{
    l10n::{L10N, Term},
    states::settings::Settings,
    views::windows::SingletonWindowOpenState,
};

static VIEWPORT_ID: LazyLock<egui::ViewportId> =
    LazyLock::new(|| egui::ViewportId::from_hash_of("SettingsWindow"));

pub struct SettingsWindow {
    l: crate::l10n::L10N,
    settings: Arc<RwLock<Settings>>,
}

impl SettingsWindow {
    pub fn new(l: crate::l10n::L10N, settings: Arc<RwLock<Settings>>) -> Self {
        Self { l, settings }
    }
}

impl SettingsWindow {
    pub fn window(&mut self, ui: &mut egui::Ui, open_state: SingletonWindowOpenState) {
        let widget = SettingsWidget::from_window_ref(self);

        if open_state.pop_is_desiring_focus() {
            ui.send_viewport_cmd_to(*VIEWPORT_ID, egui::ViewportCommand::Focus);
        }

        ui.ctx().show_viewport_deferred(
            *VIEWPORT_ID,
            egui::ViewportBuilder::default()
                .with_title(self.l.tl(&Term::Settings))
                .with_inner_size((240.0, 240.0))
                .with_resizable(false)
                .with_always_on_top(),
            move |ui, class| {
                if ui.input(|i| i.viewport().close_requested()) {
                    open_state.close();
                    return;
                }

                if class == egui::ViewportClass::EmbeddedWindow {
                    unimplemented!("Embedded viewports are not supported yet.");
                } else {
                    egui::CentralPanel::default().show(ui, |ui| {
                        widget.ui(ui);
                    });
                }
            },
        );
    }
}

struct SettingsWidget {
    l: L10N,
    settings: Arc<RwLock<Settings>>,
}

impl SettingsWidget {
    fn from_window_ref(window: &SettingsWindow) -> Self {
        Self {
            l: window.l.clone(),
            settings: window.settings.clone(),
        }
    }
}

impl SettingsWidget {
    fn ui(&self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new(self.l.tl(&Term::Appearance))
            .default_open(true)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.settings_theme_selector(ui);
                });
            });
    }

    fn settings_theme_selector(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(self.l.tl(&Term::Theme));

            ui.horizontal(|ui| {
                let mut theme = ui.theme();
                let old_theme = theme.clone();

                ui.scope(|ui| {
                    ui.style_mut().spacing.item_spacing.x = 0.0;
                    ui.selectable_value(
                        &mut theme,
                        egui::Theme::Dark,
                        egui_phosphor::regular::MOON,
                    );
                    ui.selectable_value(
                        &mut theme,
                        egui::Theme::Light,
                        egui_phosphor::regular::SUN,
                    );
                });

                if theme != old_theme {
                    self.settings
                        .write()
                        .expect("Failed to acquire write lock on settings.")
                        .set_theme(ui, theme);
                }
            });
        });
    }
}
