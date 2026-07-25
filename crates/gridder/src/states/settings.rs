use eframe::egui;

use crate::l10n::L10NMut;

pub struct Settings {
    l10n_mut: L10NMut,

    theme: egui::Theme,
    is_theme_dirty: bool,
}

impl Settings {
    pub fn new(l10n_mut: L10NMut) -> Self {
        Self {
            l10n_mut,
            theme: egui::Theme::Dark,
            is_theme_dirty: true,
        }
    }

    pub fn set_current_language(&mut self, language_code: &'static str) {
        self.l10n_mut.set_current_language(language_code);
    }

    /// FIXME: non-root viewports are not updated immediately.
    pub fn set_theme(&mut self, ui: &egui::Ui, theme: egui::Theme) {
        if self.theme != theme {
            self.theme = theme;
            self.is_theme_dirty = true;
        }
        ui.request_repaint_of(egui::ViewportId::ROOT);
    }
    pub fn pop_dirty_theme(&mut self) -> Option<egui::Theme> {
        if self.is_theme_dirty {
            self.is_theme_dirty = false;
            Some(self.theme)
        } else {
            None
        }
    }
}
